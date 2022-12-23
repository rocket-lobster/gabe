#[macro_use]
extern crate log;

pub mod gb;
pub mod disassemble;
pub mod sink;
mod apu;
mod cartridge;
mod cpu;
mod joypad;
mod mmu;
mod serial;
mod timer;
mod util;
mod vram;
mod wram;

pub const CLOCK_RATE: u32 = 4_194_304;
pub const CGB_CLOCK_RATE: u32 = CLOCK_RATE * 2;
pub const SAMPLE_RATE: u32 = CLOCK_RATE / 16; // 262.144 KHz sample rate