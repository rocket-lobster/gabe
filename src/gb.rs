use super::cpu;
use super::mmu;

use std::io;
use std::path::Path;

pub struct Gameboy {
    cpu: cpu::Cpu,
    mmu: mmu::Mmu,
    frame_cycles: usize,
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
        const CYCLES_PER_FRAME: usize = 4194304 / 60;

        while self.frame_cycles < CYCLES_PER_FRAME {
            // Will run until reaching a broken opcode,
            // no timing for now
            let cycles = self.cpu.tick(&mut self.mmu);

            // update_gpu(cycles)
            // update_apu(cycles)
            // update_timers(cycles)
            self.frame_cycles += cycles;
        }

        // Frame complete, setup for next frame
        self.frame_cycles -= CYCLES_PER_FRAME;
        trace!("Frame complete");
    }
}
