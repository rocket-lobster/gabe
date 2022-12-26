use core::panic;
use std::io::{Read, Seek, Write};

use super::super::mmu::Memory;
use super::{Cartridge, CartridgeError};

// Maximum can support 2 MB worth of ROM banks, which is 0x7F = 128 16-Kb banks
const MAX_ROM_SIZE: u32 = 0x1FFFFF;

/// MBC1 cartridges can support up to 2 MB of ROM banks and/or 32 KB of RAM banks
/// Requires to be provided the ROM and RAM size to calculate the number of
/// ROM/RAM banks to support
pub struct Mbc1 {
    rom: Box<[u8]>,
    ram: Box<[u8]>,
    rom_bank: u8,
    rom_bank_count: u8,
    ram_bank: u8,
    ram_bank_count: u8,
    ram_enabled: bool,
    has_battery: bool,
    mode1_enabled: bool,
}

impl Mbc1 {
    pub fn power_on(rom: Vec<u8>, rom_size: u8, ram_size: u8, has_battery: bool) -> Self {
        assert!(rom.len() <= MAX_ROM_SIZE as usize);
        let rom_bank_count: u8 = match rom_size {
            0x0 => 0x02, // 32 KB
            0x1 => 0x04, // 64 KB
            0x2 => 0x08, // 128 KB
            0x3 => 0x10, // 256 KB
            0x4 => 0x20, // 512 KB
            0x5 => 0x40, // 1 MB
            0x6 => 0x80, // 2 MB
            _ => panic!("Provided ROM Size unsupported for MBC1."),
        };
        let ram_bank_count: u8 = match ram_size {
            0x0 | 0x1 => 0x0, // 0 KB
            0x2 => 0x01,      // 8 KB
            0x3 => 0x04,      // 32 KB
            _ => panic!("Provided RAM Size unsupported for MBC1."),
        };
        let ram: Vec<u8> = vec![0; (0x2000u32 * ram_bank_count as u32) as usize];
        Mbc1 {
            rom: rom.into_boxed_slice(),
            ram: ram.into_boxed_slice(),
            rom_bank: 1,
            ram_bank: 0,
            rom_bank_count,
            ram_bank_count,
            ram_enabled: false,
            has_battery,
            mode1_enabled: false,
        }
    }
}

impl Memory for Mbc1 {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            // Always gets the lower bank 0, no translation of addr
            0x0000..=0x3FFF => {
                if self.mode1_enabled {
                    // Using Mode 1, so bits 5 and 6 are used to select the location of the lower bank
                    // e.g. if we are using bank 0x3A = 0b011_1010, mask bits 4-0 off and use the resulting
                    // value to find the bank for 0x0000-0x3FFF, which would be 0b011_1010 & 0b110_0000 = 0b010_0000 = bank 0x20
                    self.rom[(addr as u32 + (0x4000u32 * (self.rom_bank & 0x60) as u32)) as usize]
                } else {
                    self.rom[addr as usize]
                }
            }
            // Offset the addr to be relative to the bank, then add the offset based of the rom_bank
            // Allows this range to technically be a cloned area of bank 0 in some edge cases where rom_bank is 0
            0x4000..=0x7FFF => {
                self.rom[((addr - 0x4000) as u32 + (0x4000u32 * self.rom_bank as u32)) as usize]
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    if self.mode1_enabled {
                        self.ram
                            [((addr - 0xA000) as u32 + (0x2000u32 * self.ram_bank as u32)) as usize]
                    } else {
                        // Without Mode 1, RAM always uses bank 0.
                        self.ram[(addr - 0xA000) as usize]
                    }
                } else {
                    0xFF
                }
            }
            _ => {
                error!("Invalid cartridge read address {}", addr);
                0
            }
        }
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x1FFF => {
                if ((val & 0xF) == 0x0A) && self.ram_bank_count != 0 {
                    self.ram_enabled = true;
                } else {
                    self.ram_enabled = false;
                }
            }
            0x2000..=0x3FFF => {
                if (val & 0x1F) == 0x0 {
                    self.rom_bank = 1;
                } else {
                    // Mask into ROM bank after check, so that you can technically select rom_bank 0
                    match self.rom_bank_count {
                        0x02 => self.rom_bank = val & 0x01,
                        0x04 => self.rom_bank = val & 0x03,
                        0x08 => self.rom_bank = val & 0x07,
                        0x10 => self.rom_bank = val & 0x0F,
                        0x20 => self.rom_bank = val & 0x1F,
                        _ => panic!("MBC1 ROM Bank selection logic failure."),
                    }
                }
            }
            0x4000..=0x5FFF => {
                if self.rom_bank_count >= 0x40 {
                    // Using a >1 MB ROM, need additional bits to select ROM bank
                    self.rom_bank += (val & 0x3) << 5;
                } else if self.ram_bank_count == 0x4 {
                    // Using 32 KB of ram, select the RAM bank
                    self.ram_bank = val & 0x3;
                }
            }
            0x6000..=0x7FFF => {
                // Mode selection only matters if using RAM or large ROM sizes
                if self.rom_bank_count >= 0x40 || self.ram_bank_count == 0x4 {
                    self.mode1_enabled = (val & 0x1) == 0x1;
                }
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    if self.mode1_enabled {
                        self.ram[((addr - 0xA000) as u32 + (0x2000u32 * self.ram_bank as u32))
                            as usize] = val;
                    } else {
                        // Without Mode 1, RAM always uses bank 0.
                        self.ram[(addr - 0xA000) as usize] = val;
                    }
                }
            }
            _ => error!("Invalid cartridge write address {}", addr),
        }
    }
}

impl Cartridge for Mbc1 {
    fn read_save_file(&mut self, file: &mut std::fs::File) -> Result<(), CartridgeError> {
        if self.has_battery && self.ram_bank_count >= 0x1 {
            // We have battery-backed RAM available to read from a file
            // If we hit a read error, just propagate up, otherwise we succeed.
            if let Err(e) = file.rewind() {
                Err(CartridgeError::Io(e))
            } else if let Err(e) = file.read(&mut self.ram) {
                Err(CartridgeError::Io(e))
            } else {
                Ok(())
            }
        } else {
            Err(CartridgeError::Unsupported(
                "Game doesn't support save files via battery-backed RAM.".to_string(),
            ))
        }
    }

    fn write_save_file(&self, file: &mut std::fs::File) -> Result<(), CartridgeError> {
        if self.has_battery && self.ram_bank_count >= 0x1 {
            // We have battery-backed RAM available to write to a file
            // If we hit a write error, just propagate up, otherwise we succeed.
            if let Err(e) = file.rewind() {
                Err(CartridgeError::Io(e))
            } else if let Err(e) = file.write_all(&self.ram) {
                Err(CartridgeError::Io(e))
            } else {
                Ok(())
            }
        } else {
            Err(CartridgeError::Unsupported(
                "Game doesn't support save files via battery-backed RAM.".to_string(),
            ))
        }
    }
}
