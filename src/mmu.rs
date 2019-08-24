use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::mbc0::Mbc0;
use super::memory::Memory;
use super::vram::Vram;
use super::wram::Wram;

/// The state of all Gameboy memory, both internal memory and external cartridge memory
///
/// This structure is used whenever the CPU needs to write into or read from memory,
/// and Mmu then handles any appropriate bank switching, depending on the provided game.
/// Also handles game saves when appropriate.
pub struct Mmu {
    cart: Box<dyn Memory>,
    vram: Vram,
    wram: Wram,
    oam: [u8; 0xA0],
    io: [u8; 0x80],
    hram: [u8; 0x7F],
    ie: bool,
}

impl Mmu {
    pub fn power_on(path: impl AsRef<Path>) -> Self {
        let mut f = File::open(path.as_ref()).unwrap();
        let mut rom_data = Vec::new();
        f.read_to_end(&mut rom_data).unwrap();
        println!("ROM size: {}", rom_data.len());
        let cart: Box<dyn Memory> = match rom_data[0x147] {
            0x00 => Box::new(Mbc0::power_on(rom_data)),
            _ => unimplemented!(),
        };

        Mmu {
            cart,
            vram: Vram::power_on(),
            wram: Wram::power_on(),
            oam: [0; 0xA0],
            io: [0; 0x80],
            hram: [0; 0x7F],
            ie: false,
        }
    }
}

impl Memory for Mmu {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.cart.read_byte(addr),
            0x8000..=0x9FFF => self.vram.read_byte(addr),
            0xA000..=0xBFFF => self.cart.read_byte(addr),
            0xC000..=0xFDFF => self.wram.read_byte(addr),
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize],
            0xFF00..=0xFF7F => self.io[(addr - 0xFF00) as usize],
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize],
            0xFFFF => self.ie as u8,
            _ => unimplemented!(),
        }
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x7FFF => self.cart.write_byte(addr, val),
            0x8000..=0x9FFF => self.vram.write_byte(addr, val),
            0xA000..=0xBFFF => self.cart.write_byte(addr, val),
            0xC000..=0xFDFF => self.wram.write_byte(addr, val),
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize] = val,
            0xFF00..=0xFF7F => self.io[(addr - 0xFF00) as usize] = val,
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize] = val,
            0xFFFF => match val {
                0 => self.ie = false,
                _ => self.ie = true,
            },
            _ => unimplemented!(),
        };
    }
}
