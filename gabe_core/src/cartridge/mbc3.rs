use core::cmp::Ordering;

use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::*;

use super::super::mmu::Memory;
use super::{Cartridge, CartridgeError};

// Maximum can support 2 MB worth of ROM banks, which is 0x7F = 128 16-Kb banks
const MAX_ROM_SIZE: u32 = 0x20_0000;

/// MBC3 cartridges can support up to 2 MB of ROM banks and/or 32 KB of RAM banks
/// Requires to be provided the ROM and RAM size to calculate the number of
/// ROM/RAM banks to support
/// Also supports a Real-Time Clock
pub struct Mbc3 {
    rom: Box<[u8]>,
    ram: Box<[u8]>,
    rom_bank: u8,
    _rom_bank_count: u8,
    ram_bank: u8,
    ram_bank_count: u8,
    ram_enabled: bool,
    has_battery: bool,
    _has_rtc: bool,
    rtc_enabled: bool,
}

impl Mbc3 {
    pub fn power_on(
        rom: Box<[u8]>,
        rom_size: u8,
        ram_size: u8,
        has_battery: bool,
        has_rtc: bool,
    ) -> Self {
        assert!(rom.len() <= MAX_ROM_SIZE as usize);
        let rom_bank_count: u8 = match rom_size {
            0x0 => 0x02, // 32 KB
            0x1 => 0x04, // 64 KB
            0x2 => 0x08, // 128 KB
            0x3 => 0x10, // 256 KB
            0x4 => 0x20, // 512 KB
            0x5 => 0x40, // 1 MB
            0x6 => 0x80, // 2 MB
            _ => panic!("Provided ROM Size unsupported for MBC3."),
        };
        let ram_bank_count: u8 = match ram_size {
            0x0 | 0x1 => 0x0, // 0 KB
            0x2 => 0x01,      // 8 KB
            0x3 => 0x04,      // 32 KB
            _ => panic!("Provided RAM Size unsupported for MBC3."),
        };
        let ram: Vec<u8> = vec![0; (0x2000u32 * ram_bank_count as u32) as usize];
        if has_rtc {
            error!("MBC3 RTC not implemented, clock info will not be provided.");
        }
        Mbc3 {
            rom,
            ram: ram.into_boxed_slice(),
            rom_bank: 1,
            ram_bank: 0,
            _rom_bank_count: rom_bank_count,
            ram_bank_count,
            ram_enabled: false,
            has_battery,
            _has_rtc: has_rtc,
            rtc_enabled: false,
        }
    }
}

impl Memory for Mbc3 {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            // Always gets the lower bank 0, no translation of addr
            0x0000..=0x3FFF => self.rom[addr as usize],
            // Offset the addr to be relative to the bank, then add the offset based of the rom_bank
            // Allows this range to technically be a cloned area of bank 0 in some edge cases where rom_bank is 0
            0x4000..=0x7FFF => {
                self.rom[((addr - 0x4000) as u32 + (0x4000u32 * self.rom_bank as u32)) as usize]
            }
            0xA000..=0xBFFF => {
                if self.rtc_enabled {
                    // TODO: Read RTC regs
                    0x00
                } else if self.ram_enabled {
                    self.ram[((addr - 0xA000) as u32 + (0x2000u32 * self.ram_bank as u32)) as usize]
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
                if (val & 0x7F) == 0x0 {
                    self.rom_bank = 1;
                } else {
                    self.rom_bank = val & 0x7F;
                }
            }
            0x4000..=0x5FFF => {
                if self.ram_bank_count == 0x4 {
                    // Using 32 KB of ram, select the RAM bank
                    self.ram_bank = val & 0x3;
                }
            }
            0xA000..=0xBFFF => {
                if self.rtc_enabled {
                    // TODO: RTC registers
                } else if self.ram_enabled {
                    self.ram
                        [((addr - 0xA000) as u32 + (0x2000u32 * self.ram_bank as u32)) as usize] =
                        val;
                }
            }
            _ => error!("Invalid cartridge write address {}", addr),
        }
    }
}

impl Cartridge for Mbc3 {
    fn read_save_data(&mut self, data: Box<[u8]>) -> Result<(), CartridgeError> {
        if self.has_battery {
            // We have battery-backed RAM available to read from a file
            // If we hit a read error, just propagate up, otherwise we succeed.
            match data.len().cmp(&self.ram.len()) {
                Ordering::Equal => {
                    self.ram.copy_from_slice(data.as_ref());
                    Ok(())
                }
                Ordering::Greater => {
                    // Fill RAM with data until full
                    for (i, v) in self.ram.iter_mut().enumerate() {
                        *v = data[i];
                    }
                    Ok(())
                }
                Ordering::Less => {
                    // Fill RAM with data until out of data
                    for (i, v) in data.iter().enumerate() {
                        self.ram[i] = *v;
                    }
                    Ok(())
                }
            }
        } else {
            Err(CartridgeError::Unsupported(
                "Game doesn't support save files via battery-backed RAM.".to_string(),
            ))
        }
    }

    fn write_save_data(&self) -> Result<Box<[u8]>, CartridgeError> {
        if self.has_battery {
            // We have battery-backed RAM available to maintain save data
            // Provide cloned RAM data as a pointer
            Ok(self.ram.clone())
        } else {
            Err(CartridgeError::Unsupported(
                "Game doesn't support save files via battery-backed RAM.".to_string(),
            ))
        }
    }
}
