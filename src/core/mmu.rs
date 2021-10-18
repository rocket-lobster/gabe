use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

use super::apu::Apu;
use super::interrupt::{self, InterruptKind};
use super::joypad::Joypad;
use super::mbc0::Mbc0;
use super::memory::Memory;
use super::timer::Timer;
use super::vram::Vram;
use super::wram::Wram;

/// The state of all Gameboy memory, both internal memory and external cartridge memory
///
/// This structure is used whenever the CPU needs to write into or read from memory,
/// and then each block provides the services necessary when updated. MMU only handles
/// reading and writing into each block, no logic is performed otherwise.
pub struct Mmu {
    cart: Box<dyn Memory>,
    apu: Apu,
    vram: Vram,
    wram: Wram,
    timer: Timer,
    joypad: Joypad,
    oam: [u8; 0xA0],
    hram: [u8; 0x7F],
    intf: u8,
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
            apu: Apu::power_on(),
            vram: Vram::power_on(),
            wram: Wram::power_on(),
            timer: Timer::power_on(),
            joypad: Joypad::power_on(),
            oam: [0; 0xA0],
            hram: [0; 0x7F],
            intf: 0,
            ie: false,
        };

        Ok(mmu)
    }

    /// Updates all memory components to align with the number of cycles
    /// run by the CPU, given by `cycles`. 
    /// Handles updates in response to Interrupts being returned by each 
    /// block, for the CPU to handle on the next fetch.
    pub fn update(&mut self, cycles: usize) {
        // Update APU
        // Update VRAM
        // Update Joypad
        if let Some(i) = self.timer.update(cycles) {
            self.request_interrupt(i);
        }
        if let Some(i) = self.vram.update(cycles) {
            for interrupt in i {
                self.request_interrupt(interrupt);
            }
        }
    }

    /// Takes the given Interrupt enum value, and sets the corresponding bit
    /// in the IF register. CPU will run interrupt handler on next fetch cycle.
    pub fn request_interrupt(&mut self, int: InterruptKind) {
        // Grab the IF register of current interrupt requests
        let mut int_flag = self.read_byte(0xFF0F);
        int_flag |= int as u8;
        self.write_byte(0xFF0F, int_flag);
    }

    /// Debug function. Returns a simple Vec of the requested range of data. Only returns
    /// data visible to MMU, so any non-selected banks or block-internal data not memory-mapped
    /// will not be returned.
    pub fn get_memory_range(&self, start: u16, end: u16) -> Option<Vec<u8>> {
        if start <= end {
            let mut vec: Vec<u8> = Vec::new();
            for addr in start..=end {
                vec.push(self.read_byte(addr));
            }
            Some(vec)
        } else {
            None
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
            0xFF00 => self.joypad.read_byte(addr),
            0xFF04..=0xFF07 => self.timer.read_byte(addr),
            0xFF0F => self.intf,
            0xFF10..=0xFF2F => self.apu.read_byte(addr),
            0xFF40..=0xFF6F => self.vram.read_byte(addr),
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
            0xFF00 => self.joypad.write_byte(addr, val),
            0xFF04..=0xFF07 => self.timer.write_byte(addr, val),
            0xFF0F => self.intf = val,
            0xFF10..=0xFF2F => self.apu.write_byte(addr, val),
            0xFF40..=0xFF6F => self.vram.write_byte(addr, val),
            0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize] = val,
            0xFFFF => match val {
                0 => self.ie = false,
                _ => self.ie = true,
            },
            _ => unimplemented!("Address: {:4X}", addr),
        };
    }
}

#[cfg(test)]
mod mmu_tests {
    use super::*;
    #[test]
    fn interrupt_requests() {}
}
