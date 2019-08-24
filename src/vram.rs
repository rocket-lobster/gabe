use super::memory::Memory;

pub struct Vram {
    memory: Vec<u8>,
}

impl Vram {
    pub fn power_on() -> Self {
        Vram {
            memory: vec![0; 0x2000]
        }
    }
}

impl Memory for Vram {
    fn read_byte(&self, addr: u16) -> u8 {
        assert!(addr >= 0x8000 && addr <= 0x9FFF);
        self.memory[(addr - 0x8000) as usize]
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        assert!(addr >= 0x8000 && addr <= 0x9FFF);
        self.memory[(addr - 0x8000) as usize] = val;
    }
}