pub mod gb;

/// Type alias for the rendered screen data
pub type FrameData = [[[u8; 3]; 160]; 144];

mod apu;
mod cpu;
mod interrupt;
mod joypad;
mod mbc0;
mod memory;
mod mmu;
mod timer;
mod vram;
mod wram;