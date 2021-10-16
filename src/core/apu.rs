use super::memory::Memory;

pub struct Apu {
    memory: Vec<u8>,
}

impl Apu {
    pub fn power_on() -> Self {
        Apu {
            memory: vec![0; 0x20],
        }
    }
}

impl Memory for Apu {
    fn read_byte(&self, addr: u16) -> u8 {
        assert!(addr >= 0xFF10 && addr <= 0xFF2F);
        self.memory[(addr - 0xFF10) as usize]
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        assert!(addr >= 0xFF10 && addr <= 0xFF2F);
        self.memory[(addr - 0xFF10) as usize] = val;
    }
}
