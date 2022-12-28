use super::cpu;
use super::mmu;
use super::mmu::Memory;
use super::sink::*;

use alloc::boxed::*;

pub struct Gameboy {
    cpu: cpu::Cpu,
    mmu: mmu::Mmu,
}

/// The supported input states for the Joypad.
/// User provides a combined mask of these values during each step call
pub enum GbKeys {
    Right = 0,
    Left = 1,
    Up = 2,
    Down = 3,
    A = 4,
    B = 5,
    Select = 6,
    Start = 7,
}

pub struct GbDebug {
    pub cpu_data: cpu::Cpu,
    pub ie_data: u8,
    pub if_data: u8,
    pub vram_lcdc: u8,
    pub vram_stat: u8,
    pub vram_ly: u8,
}

impl Gameboy {
    /// Initializes Gameboy state to begin emulation on provided
    /// binary file
    pub fn power_on(rom_data: Box<[u8]>, save_data: Option<Box<[u8]>>) -> Self {
        let mmu = mmu::Mmu::power_on(rom_data, save_data);
        Gameboy {
            cpu: cpu::Cpu::power_on(),
            mmu,
        }
    }

    /// Executes one CPU instruction and updates the other
    /// subsystems with the appropriate number of cycles
    /// Returns a frame if completed during the tick.
    pub fn step(
        &mut self,
        video_sink: &mut dyn Sink<VideoFrame>,
        audio_sink: &mut dyn Sink<AudioFrame>,
    ) -> u32 {
        let cycles = self.cpu.tick(&mut self.mmu);

        // Update memory
        self.mmu.update(cycles, video_sink, audio_sink);
        cycles
    }

    pub fn update_key_state(&mut self, key: GbKeys, pressed: bool) {
        self.mmu.joypad.set_key_pressed(key, pressed);
    }

    pub fn get_save_data(&self) -> Option<Box<[u8]>> {
        let result = self.mmu.cart.write_save_data();
        match result {
            Ok(d) => Some(d),
            Err(_) => None,
        }
    }

    pub fn poll_serial(&mut self) -> Option<u8> {
        if self.mmu.read_byte(0xFF02) == 0x81 {
            // Output ready
            // Clear SC to ACK
            self.mmu.write_byte(0xFF02, 0x01);
            Some(self.mmu.read_byte(0xFF01))
        } else {
            None
        }
    }

    pub fn get_debug_state(&self) -> GbDebug {
        GbDebug {
            cpu_data: self.cpu.get_debug_data(),
            if_data: self.mmu.read_byte(0xFF0F),
            ie_data: self.mmu.read_byte(0xFFFF),
            vram_lcdc: self.mmu.read_byte(0xFF40),
            vram_stat: self.mmu.read_byte(0xFF41),
            vram_ly: self.mmu.read_byte(0xFF44),
        }
    }

    /// Returns the current program counter of the CPU
    pub fn get_pc(&self) -> u16 {
        self.cpu.reg.pc
    }

    /// Returns a boxed slice of u8 values contained within the given range of usize values.
    /// Only returns values as read via the CPU, so forbidden or fixed reads will not be bypassed
    pub fn get_memory_range(&self, range: core::ops::Range<usize>) -> Box<[u8]> {
        self.mmu.get_memory_range(range).into_boxed_slice()
    }
}
