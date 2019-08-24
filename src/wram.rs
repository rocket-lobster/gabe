use super::memory::Memory;

pub struct Wram {
    memory: Vec<u8>,
}

impl Wram {
    pub fn power_on() -> Self {
        Wram {
            memory: vec![0; 0x2000]
        }
    }
}

impl Memory for Wram {
    fn read_byte(&self, addr: u16) -> u8 {
        assert!(addr >= 0xC000 && addr <= 0xDFFF);
        self.memory[(addr - 0xC000) as usize]
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        assert!(addr >= 0xC000 && addr <= 0xDFFF);
        self.memory[(addr - 0xC000) as usize] = val;
    }
}