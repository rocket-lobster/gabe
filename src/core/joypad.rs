use super::interrupt::InterruptKind;
use super::memory::Memory;
use super::gb::GbKeys;

pub struct Joypad {
    memory: u8,
}

impl Joypad {
    pub fn power_on() -> Self {
        Joypad {
            memory: 0xFF,
        }
    }

    pub fn update(&mut self, keys_pressed: Option<&[GbKeys]>) -> Option<InterruptKind> {
        let old_state = self.memory;
        if let Some(keys) = keys_pressed {
            // Some keys were pressed
            if !(self.memory | 0b1101_1111) != 0b0 {
                // Action Select set
                // Clear state of buttons
                self.memory &= 0b1111;
                keys.iter().for_each(|k| {
                    match k {
                        GbKeys::A => self.memory &= 0b1110,
                        GbKeys::B => self.memory &= 0b1101,
                        GbKeys::Select => self.memory &= 0b1011,
                        GbKeys::Start => self.memory &= 0b0111,
                        _ => ()
                    }
                });
                // Check if any button was pressed only
                if old_state & 0b1111 > self.memory & 0b1111 {
                    Some(InterruptKind::Joypad)
                } else {
                    None
                }
            } else if !(self.memory | 0b1110_1111) != 0b0 {
                // Direction Select set
                // Clear state of buttons
                self.memory &= 0b1111;
                keys.iter().for_each(|k| {
                    match k {
                        GbKeys::Right => self.memory &= 0b1110,
                        GbKeys::Left => self.memory &= 0b1101,
                        GbKeys::Up => self.memory &= 0b1011,
                        GbKeys::Down => self.memory &= 0b0111,
                        _ => ()
                    }
                });
                // Check if any button was pressed only
                if old_state & 0b1111 > self.memory & 0b1111 {
                    Some(InterruptKind::Joypad)
                } else {
                    None
                }
            } else {
                // Neither slot is selected, change nothing
                None
            }
        } else {
            // No keys pressed
            None
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
        // Only write the upper nibble into the register, mask everything else off
        match (val >> 4) & 0b11 {
            0b00 => self.memory &= 0b1100_1111,
            0b01 => {
                self.memory &= 0b1101_1111;
                self.memory |= 0b0001_0000;
            }
            0b10 => {
                self.memory &= 0b1110_1111;
                self.memory |= 0b0010_0000;
            }
            0b11 => self.memory |= 0b0011_0000,
            _ => panic!("Logic error.")
        } 
    }
}
