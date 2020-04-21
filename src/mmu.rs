use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

use super::interrupt::Interrupt;
use super::mbc0::Mbc0;
use super::memory::Memory;
use super::timer::Timer;
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
    timer: Timer,
    oam: [u8; 0xA0],
    io: [u8; 0x80],
    hram: [u8; 0x7F],
    ie: bool,
}

impl Mmu {
    /// Initializes the MMU with the given ROM path.
    /// Opens the given file and reads cartridge header information to find
    /// the MBC type.
    pub fn power_on(path: impl AsRef<Path>) -> io::Result<Self> {
        let mut f = File::open(path.as_ref())?;
        let mut rom_data = Vec::new();
        f.read_to_end(&mut rom_data)?;
        debug!("ROM size: {}", rom_data.len());
        let cart: Box<dyn Memory> = match rom_data[0x147] {
            0x00 => Box::new(Mbc0::power_on(rom_data)),
            _ => unimplemented!("MBC given not supported!"),
        };
        let mmu = Mmu {
            cart,
            vram: Vram::power_on(),
            wram: Wram::power_on(),
            timer: Timer::power_on(),
            oam: [0; 0xA0],
            io: [0; 0x80],
            hram: [0; 0x7F],
            ie: false,
        };

        Ok(mmu)
    }

    /// Updates all memory components to align with the number of cycles
    /// run by the CPU, given by `cycles`
    pub fn update(&mut self, cycles: usize) {
        // Update APU
        // Update VRAM
        // Update Joypad
        if let Some(i) = self.timer.update(cycles) {
            self.request_interrupt(i);
        }
    }

    /// Takes the given Interrupt enum value, and sets the corresponding bit
    /// in the IF register
    pub fn request_interrupt(&mut self, int: Interrupt) {
        // Grab the IF register of current interrupt requests
        let mut int_flag = self.read_byte(0xFF0F);
        int_flag |= int as u8;
        self.write_byte(0xFF0F, int_flag);
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
            0xFF04..=0xFF07 => self.timer.read_byte(addr),
            0xFF40..=0xFF45 => self.vram.read_byte(addr),
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
            0xFF04..=0xFF07 => self.timer.write_byte(addr, val),
            0xFF40..=0xFF45 => self.vram.write_byte(addr, val),
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize] = val,
            0xFFFF => match val {
                0 => self.ie = false,
                _ => self.ie = true,
            },
            _ => unimplemented!(),
        };
    }
}

#[cfg(test)]
mod mmu_tests {
    use super::*;
    #[test]
    fn interrupt_requests() {}
}
