use super::apu::AudioBuffer;
use super::cpu;
use super::mmu;
use super::mmu::Memory;
use super::vram::FrameData;

use std::io;
use std::path::Path;
use std::time::Duration;

pub struct Gameboy {
    cpu: cpu::Cpu,
    mmu: mmu::Mmu,
    extra_cycles: usize,
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
                extra_cycles: 0,
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

        // Update memory
        self.mmu.update(cycles, keys_pressed)
    }

    /// Runs the emulator for the provided number of seconds elapsed, provided as a Duration.
    /// Converts the Duration into the number of clock cycles elapsed in the seconds provided
    /// If the provided duration is long enough that multiple video frames are generated, only the
    /// latest frame will be returned, otherwise it will return None.
    pub fn step_seconds(
        &mut self,
        seconds_elapsed: Duration,
        keys_pressed: Option<&[GbKeys]>,
    ) -> Option<FrameData> {
        // Multiply the provided duration by the clock rate in T-cycles
        // Flooring to create an integer number of cycles
        // Minor timing loss from flooring
        let mut cycles: usize = (seconds_elapsed.as_secs_f64() * 4_194_304f64).floor() as usize;

        // Check if previous steps resulted in more leftover cycles than we need to run
        // Do nothing if we aren't running enough cycles
        if cycles <= self.extra_cycles {
            self.extra_cycles -=  cycles;
            None
        } else {
            // Remove the leftover cycles from a previous step from our current cycles to run
            cycles -= self.extra_cycles;
            let mut ret = None;

            while cycles > 0 {
                let cpu_cycles = self.cpu.tick(&mut self.mmu);

                // Update memory
                if let Some(f) = self.mmu.update(cpu_cycles, keys_pressed) {
                    ret = Some(f);
                }

                // If the number of cycles we ran is more than we needed to run, 
                // account for that by tracking the extra cycles, and return 0
                cycles = if let Some(c) = cycles.checked_sub(cpu_cycles) {
                    c 
                } else {
                    self.extra_cycles = cpu_cycles - cycles;
                    0
                };
            }
            ret
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
    pub fn get_memory_range(&self, range: std::ops::Range<usize>) -> Box<[u8]> {
        self.mmu.get_memory_range(range).into_boxed_slice()
    }
}
