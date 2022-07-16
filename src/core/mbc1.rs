use core::panic;

use super::memory::Memory;

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
}

impl Mbc1 {
    pub fn power_on(rom: Vec<u8>, rom_size: u8, ram_size: u8) -> Self {
        assert!(rom.len() <= MAX_ROM_SIZE as usize);
        let rom_bank_count: u8 = match rom_size {
            0x0 => 0x02,
            0x1 => 0x04,
            0x2 => 0x08,
            0x3 => 0x10,
            0x4 => 0x20,
            0x5 => 0x40,
            0x6 => 0x80,
            _ => panic!("Provided ROM Size unsupported for MBC1."),
        };
        let ram_bank_count: u8 = match ram_size {
            0x0 | 0x1 => 0x0,
            0x2 => 0x01,
            0x3 => 0x04,
            _ => panic!("Provided RAM Size unsupported for MBC1."),
        };
        let ram: Vec<u8> = vec![0; (0x2000 * rom_bank_count as u16) as usize];
        Mbc1 {
            rom: rom.into_boxed_slice(),
            ram: ram.into_boxed_slice(),
            rom_bank: 1,
            ram_bank: 0,
            rom_bank_count,
            ram_bank_count,
        }
    }
}

impl Memory for Mbc1 {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x3FFF => self.rom[(addr as usize)],
            0x4000..=0x7FFF => self.rom[(addr as usize)],
            0xA000..=0xBFFF => self.ram[(addr as usize)],
            _ => {
                error!("Invalid cartridge read address {}", addr);
                0
            }
        }
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x3FFF => self.rom[(addr as usize)] = val,
            0x4000..=0x7FFF => self.rom[(addr as usize)] = val,
            0xA000..=0xBFFF => self.ram[(addr as usize)] = val,
            _ => error!("Invalid cartridge write address {}", addr)
        }
    }
}
