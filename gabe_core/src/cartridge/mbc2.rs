use core::panic;
use std::fs::File;
use std::io::Write;

use super::super::mmu::Memory;
use super::{Cartridge, CartridgeError};

// Maximum can support 256 KB worth of ROM banks, which is 0x10 = 16 16-KB banks
const MAX_ROM_SIZE: u32 = 0x4_0000;

/// Mbc2 cartridges can support up to 2 MB of ROM banks and/or 32 KB of RAM banks
/// Requires to be provided the ROM and RAM size to calculate the number of
/// ROM/RAM banks to support
pub struct Mbc2 {
    rom: Box<[u8]>,
    ram: Box<[u8]>,
    rom_bank: u8,
    rom_bank_count: u8,
    ram_enabled: bool,
    has_battery: bool,
}

impl Mbc2 {
    pub fn power_on(rom: Vec<u8>, rom_size: u8, has_battery: bool) -> Self {
        assert!(rom.len() <= MAX_ROM_SIZE as usize);
        let rom_bank_count: u8 = match rom_size {
            0x0 => 0x02, // 32 KB
            0x1 => 0x04, // 64 KB
            0x2 => 0x08, // 128 KB
            0x3 => 0x10, // 256 KB
            _ => panic!("Provided ROM Size unsupported for MBC2."),
        };
        let ram: Vec<u8> = vec![0; 512];
        Mbc2 {
            rom: rom.into_boxed_slice(),
            ram: ram.into_boxed_slice(),
            rom_bank: 1,
            rom_bank_count,
            ram_enabled: false,
            has_battery,
        }
    }
}

impl Memory for Mbc2 {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            // Always gets the lower bank 0, no translation of addr
            0x0000..=0x3FFF => {
                self.rom[addr as usize]
            }
            // Offset the addr to be relative to the bank, then add the offset based of the rom_bank
            // Allows this range to technically be a cloned area of bank 0 in some edge cases where rom_bank is 0
            0x4000..=0x7FFF => {
                self.rom[((addr - 0x4000) as u32 + (0x4000u32 * self.rom_bank as u32)) as usize]
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    // RAM is on the internal MBC chip, 512 entries of 4-bit values
                    // Only contained in 0xA000-0xA1FF, but repeats through 0xBFFF,
                    // emulate by masking the lowest 9 bits of the addr
                    self.ram[((addr - 0xA000) & 0x1FF) as usize] & 0xF
                } else {
                    0xFF
                }
            }
            _ => {
                error!("Invalid cartridge read address {:X}", addr);
                0
            }
        }
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x3FFF => {
                // Check bit 8 to see if RAM or ROM is being changed
                if addr & 0x100 != 0x0 {
                    // Select ROM bank
                    if val & 0xF == 0x0 {
                        // If the value is zero, use bank 1
                        self.rom_bank = 1;
                    } else {
                        self.rom_bank = val & 0xF;
                        if self.rom_bank >= self.rom_bank_count {
                            self.rom_bank = self.rom_bank_count - 1;
                        }
                    }
                } else {
                    // Enable/Disable RAM
                    self.ram_enabled = val == 0x0A;
                }
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    // RAM is on the internal MBC chip, 512 entries of 4-bit values
                    // Only contained in 0xA000-0xA1FF, but repeats through 0xBFFF,
                    // emulate by masking the lowest 9 bits of the addr
                    self.ram[((addr - 0xA000) & 0x1FF) as usize] = val & 0xF;
                }
            }
            _ => error!("Invalid cartridge write address {:X}", addr),
        }
        
    }
}

impl Cartridge for Mbc2 {
    fn write_save_file(&self, filename: &str) -> Result<(), CartridgeError> {
        if self.has_battery {
            // We have battery-backed RAM available to write to a file
            match File::open(filename) {
                Ok(mut f) => {
                    // If we hit a write error, just propagate up, otherwise we succeed.
                    if let Err(e) = f.write_all(&self.ram) {
                        Err(CartridgeError::Io(e))
                    } else {
                        Ok(())
                    }
                }
                Err(e) => Err(CartridgeError::Io(e)),
            }
        } else {
            Err(CartridgeError::Unsupported(
                "Game doesn't support save files via battery-backed RAM.".to_string(),
            ))
        }
    }
}
