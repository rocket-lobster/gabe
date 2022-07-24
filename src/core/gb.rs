use super::apu::AudioBuffer;
use super::cpu;
use super::memory::Memory;
use super::mmu;
use super::vram::FrameData;

use std::io;
use std::path::Path;

pub struct Gameboy {
    cpu: cpu::Cpu,
    mmu: mmu::Mmu,
    total_cycles: usize,
}

/// The supported input states for the Joypad.
/// User provides a combined mask of these values during each step call
pub enum GbKeys {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    Start,
    Select,
}

pub struct GbDebug {
    pub cpu_data: cpu::Cpu,
    pub ie_data: u8,
    pub if_data: u8,
    pub vram_lcdc: u8,
    pub vram_stat: u8,
    pub vram_ly: u8,
    pub total_cycles: usize,
}

impl Gameboy {
    /// Initializes Gameboy state to begin emulation on provided
    /// binary file
    pub fn power_on(path: impl AsRef<Path>, sample_rate: u32) -> io::Result<(Self, AudioBuffer)> {
        let (mmu, audio_buffer) = mmu::Mmu::power_on(path, sample_rate)?;
        Ok((
            Gameboy {
                cpu: cpu::Cpu::power_on(),
                mmu,
                total_cycles: 0,
            },
            audio_buffer,
        ))
    }

    /// Advances the Gameboy internal state until a frame is completed.
    pub fn step(&mut self, keys_pressed: Option<&[GbKeys]>) -> FrameData {
        loop {
            if let Some(i) = self.tick(keys_pressed) {
                trace!("Frame complete");
                return i;
            }
        }
    }

    /// Executes one CPU instruction and updates the other
    /// subsystems with the appropriate number of cycles
    /// Returns a frame if completed during the tick.
    pub fn tick(&mut self, keys_pressed: Option<&[GbKeys]>) -> Option<FrameData> {
        let cycles = self.cpu.tick(&mut self.mmu);

        self.total_cycles += cycles;

        // Update memory
        self.mmu.update(cycles, keys_pressed)
    }

    pub fn get_debug_state(&self) -> GbDebug {
        GbDebug {
            cpu_data: self.cpu.get_debug_data(),
            if_data: self.mmu.read_byte(0xFF0F),
            ie_data: self.mmu.read_byte(0xFFFF),
            vram_lcdc: self.mmu.read_byte(0xFF40),
            vram_stat: self.mmu.read_byte(0xFF41),
            vram_ly: self.mmu.read_byte(0xFF44),
            total_cycles: self.total_cycles,
        }
    }

    /// Returns the current program counter of the CPU
    pub fn get_pc(&self) -> u16 {
        self.cpu.reg.pc
    }

    /// Returns a boxed slice of u8 values contained within the given range of usize values.
    /// Only returns values as read via the CPU, so forbidden or fixed reads will not be bypassed
    pub fn get_memory_range(&self, range: std::ops::Range<usize>) -> Box<[u8]> {
        self.mmu.get_memory_range(range).into_boxed_slice()
    }
}
