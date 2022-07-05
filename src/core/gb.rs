use super::cpu;
use super::mmu;
use super::vram::FrameData;

use std::io;
use std::path::Path;

pub struct Gameboy {
    cpu: cpu::Cpu,
    mmu: mmu::Mmu,
}

pub struct GbDebug {
    pub cpu_data: cpu::Cpu,
}

impl Gameboy {
    /// Initializes Gameboy state to begin emulation on provided
    /// binary file
    pub fn power_on(path: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Gameboy {
            cpu: cpu::Cpu::power_on(),
            mmu: mmu::Mmu::power_on(path)?,
        })
    }

    /// Advances the Gameboy internal state until a frame is completed.
    pub fn step(&mut self) -> FrameData {
        loop {
            if let Some(i) = self.tick() {
                trace!("Frame complete");
                return i;
            }
        }
    }

    /// Executes one CPU instruction and updates the other
    /// subsystems with the appropriate number of cycles
    /// Returns a frame if completed during the tick.
    pub fn tick(&mut self) -> Option<FrameData> {
        let cycles = self.cpu.tick(&mut self.mmu);

        // Update memory
        self.mmu.update(cycles)
    }

    pub fn get_debug_state(&self) -> GbDebug {
        GbDebug {
            cpu_data: self.cpu.get_debug_data(),
        }
    }
}
