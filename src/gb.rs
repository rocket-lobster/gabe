use super::cpu;
use super::mmu;

use std::path::Path;

struct Gameboy {
    cpu: cpu::Cpu,
    mmu: mmu::Mmu,
}

impl Gameboy {
    pub fn power_on(path: impl AsRef<Path>) -> Self {
        Gameboy {
            cpu: cpu::Cpu::power_on(),
            mmu: mmu::Mmu::power_on(path)
        }
    }
}