use super::gb::GbKeys;
use super::mmu::InterruptKind;
use super::mmu::Memory;

/// The eight Game Boy action/direction buttons are arranged as a 2x4 matrix.
/// Select either action or direction buttons by writing to this register, then read out the bits 0-3.
/// Internally represents all 8 buttons as a single byte, then returns the correct nibble when read.
/// Upper nibble: Action buttons
/// Lower nibble: Directional buttons
pub struct Joypad {
    state: u8,
    using_directions: bool,
}

impl Joypad {
    pub fn power_on() -> Self {
        Joypad {
            state: 0xFF,
            using_directions: false,
        }
    }

    pub fn update(&mut self, keys_pressed: Option<&[GbKeys]>) -> Option<InterruptKind> {
        let old_state = self.state;
        // Reset values
        self.state |= 0xFFu8;
        if let Some(keys) = keys_pressed {
            keys.iter().for_each(|k| match k {
                GbKeys::Start => self.state &= 0b0111_1111,
                GbKeys::Select => self.state &= 0b1011_1111,
                GbKeys::B => self.state &= 0b1101_1111,
                GbKeys::A => self.state &= 0b1110_1111,
                GbKeys::Down => self.state &= 0b1111_0111,
                GbKeys::Up => self.state &= 0b1111_1011,
                GbKeys::Left => self.state &= 0b1111_1101,
                GbKeys::Right => self.state &= 0b1111_1110,
            });
            // Get which bits changed states
            let cmp = old_state ^ self.state;
            // AND with previous state, shows if any bits went high to low
            if old_state & cmp != 0 {
                Some(InterruptKind::Joypad)
            } else {
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
        if self.using_directions {
            // Return directional pad values
            (self.state | 0b1111_0000) & 0b1110_1111
        } else {
            // Return action pad values
            ((self.state >> 4) | 0b1111_0000) & 0b1101_1111
        }
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        assert!(addr == 0xFF00);
        // Only write the bit 4/5 into the register, mask everything else off
        match (val >> 4) & 0b11 {
            0b00 | 0b10 => self.using_directions = true,
            0b01 | 0b11 => self.using_directions = false,
            _ => panic!("Logic error."),
        }
    }
}

#[cfg(test)]
mod joypad_tests {
    use crate::core::mmu::Memory;

    use super::GbKeys;
    use super::Joypad;

    #[test]
    fn action_buttons() {
        let mut joy = Joypad::power_on();
        joy.write_byte(0xFF00, 0xDF);
        assert_eq!(joy.update(Some(&[GbKeys::A])).is_some(), true);
        assert_eq!(joy.read_byte(0xFF00), 0b1101_1110);
        assert_eq!(
            joy.update(Some(&[GbKeys::B, GbKeys::Start])).is_some(),
            true
        );
        assert_eq!(joy.read_byte(0xFF00), 0b1101_0101);
        assert_eq!(
            joy.update(Some(&[GbKeys::Select, GbKeys::A, GbKeys::Down]))
                .is_some(),
            true
        );
        assert_eq!(joy.read_byte(0xFF00), 0b1101_1010);
        assert_eq!(
            joy.update(Some(&[GbKeys::Select, GbKeys::A, GbKeys::Down]))
                .is_some(),
            false
        );
        assert_eq!(joy.read_byte(0xFF00), 0b1101_1010);
        assert_eq!(joy.update(None).is_some(), false);
        assert_eq!(joy.read_byte(0xFF00), 0b1101_1111);
    }

    #[test]
    fn direction_buttons() {
        let mut joy = Joypad::power_on();
        joy.write_byte(0xFF00, 0xEF);
        assert_eq!(joy.update(Some(&[GbKeys::Up])).is_some(), true);
        assert_eq!(joy.read_byte(0xFF00), 0b1110_1011);
        assert_eq!(
            joy.update(Some(&[GbKeys::Down, GbKeys::Left])).is_some(),
            true
        );
        assert_eq!(joy.read_byte(0xFF00), 0b1110_0101);
        assert_eq!(
            joy.update(Some(&[GbKeys::Right, GbKeys::A])).is_some(),
            true
        );
        assert_eq!(joy.read_byte(0xFF00), 0b1110_1110);
        assert_eq!(
            joy.update(Some(&[GbKeys::Right, GbKeys::A])).is_some(),
            false
        );
        assert_eq!(joy.read_byte(0xFF00), 0b1110_1110);
        assert_eq!(joy.update(None).is_some(), false);
        assert_eq!(joy.read_byte(0xFF00), 0b1110_1111);
    }

    #[test]
    fn action_direction_transition() {
        let mut joy = Joypad::power_on();
        joy.write_byte(0xFF00, 0xDF);
        assert_eq!(joy.update(Some(&[GbKeys::A])).is_some(), true);
        assert_eq!(joy.read_byte(0xFF00), 0b1101_1110);
        assert_eq!(
            joy.update(Some(&[GbKeys::B, GbKeys::Start])).is_some(),
            true
        );
        assert_eq!(joy.read_byte(0xFF00), 0b1101_0101);
        assert_eq!(
            joy.update(Some(&[GbKeys::Select, GbKeys::A, GbKeys::Down]))
                .is_some(),
            true
        );
        assert_eq!(joy.read_byte(0xFF00), 0b1101_1010);
        joy.write_byte(0xFF00, 0xEF);
        assert_eq!(joy.update(Some(&[GbKeys::Up])).is_some(), true);
        assert_eq!(joy.read_byte(0xFF00), 0b1110_1011);
        assert_eq!(
            joy.update(Some(&[GbKeys::Down, GbKeys::Left])).is_some(),
            true
        );
        assert_eq!(joy.read_byte(0xFF00), 0b1110_0101);
        assert_eq!(
            joy.update(Some(&[GbKeys::Right, GbKeys::A])).is_some(),
            true
        );
        assert_eq!(joy.read_byte(0xFF00), 0b1110_1110);
        assert_eq!(joy.update(None).is_some(), false);
        assert_eq!(joy.read_byte(0xFF00), 0b1110_1111);
    }
}
