use super::cpu;
use super::mmu;

use std::path::Path;
use std::io;

struct Gameboy {
    cpu: cpu::Cpu,
    mmu: mmu::Mmu,
}

impl Gameboy {

    /// Initializes Gameboy state to begin emulation on provided
    /// binary file
    pub fn power_on(path: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Gameboy {
            cpu: cpu::Cpu::power_on(),
            mmu: mmu::Mmu::power_on(path)?
        })
    }

    /// Advances the Gameboy internal state by one frame
    /// Enough cycles to equal 1/60th of a second in real time
    pub fn step(&mut self) {
        // Calculate number of CPU cycles in 1/60th of a second
        // Run a CPU tick, get number of cycles ran back
        // Pass the cycles through MMU to GPU, APU, and timers to advance that emulation
        // Once we reach enough cycles, sleep the thread until enough time passes
        // Store the number of "leftover" cycles above the limit to avoid timing issues
        loop {
            // Will run until reaching a broken opcode,
            // no timing for now
            self.cpu.tick(&mut self.mmu);
        }
    }
}