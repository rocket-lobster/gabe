use super::memory::Memory;

pub struct Joypad {
    memory: u8,
}

impl Joypad {
    pub fn power_on() -> Self {
        Joypad {
            memory: 0,
        }
    }
}

impl Memory for Joypad {
    fn read_byte(&self, addr: u16) -> u8 {
        assert!(addr == 0xFF00);
        self.memory
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        assert!(addr == 0xFF00);
        self.memory = val;
    }
}
