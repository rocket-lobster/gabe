use super::cpu;
use super::mmu;

use std::io;
use std::path::Path;

pub struct Gameboy {
    cpu: cpu::Cpu,
    mmu: mmu::Mmu,
    frame_cycles: usize,
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
            frame_cycles: 0,
        })
    }

    /// Advances the Gameboy internal state by one frame
    /// Enough cycles to equal 1/60th of a second in real time
    pub fn step(&mut self) {
        // Calculate the number of cycles in 1/60th of a second
        // CPU Clock rate / 60 = Cycles per 1/60th second, i.e. frame
        const CYCLES_PER_FRAME: usize = 4_194_304 / 60;

        // Run until we reach the number of cycles in one video frame
        while self.frame_cycles < CYCLES_PER_FRAME {
            self.tick();
        }

        // Frame complete, setup for next frame
        self.frame_cycles -= CYCLES_PER_FRAME;

        // Failsafe in case logic gets ahead of frame generation, like when debugging
        // Prevents returning multiple duplicate frames that weren't displayed
        while self.frame_cycles >= CYCLES_PER_FRAME {
            self.frame_cycles -= CYCLES_PER_FRAME;
        }
        trace!("Frame complete");
    }

    /// Executes one CPU instruction and updates the other 
    /// subsystems with the appropriate number of cycles
    pub fn tick(&mut self) {
        let cycles = self.cpu.tick(&mut self.mmu);

        // Update memory
        self.mmu.update(cycles);
        self.frame_cycles += cycles;
    }

    pub fn get_debug_state(&mut self) -> GbDebug {
        GbDebug {
            cpu_data: self.cpu.get_debug_data()
        }
    }
}
