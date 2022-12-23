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
    keys_pressed: [bool; 8],
}

impl Joypad {
    pub fn power_on() -> Self {
        Joypad {
            state: 0xFF,
            using_directions: false,
            keys_pressed: [false; 8],
        }
    }

    pub fn update(&mut self) -> Option<InterruptKind> {
        let old_state = self.state;
        // Reset values
        self.state |= 0xFFu8;

        for (i, b) in self.keys_pressed.iter().enumerate() {
            if *b {
                self.state &= !(0b1 << i);
            }
        }
        // Get which bits changed states
        let cmp = old_state ^ self.state;

        // AND with previous state, shows if any bits went high to low
        if old_state & cmp != 0 {
            Some(InterruptKind::Joypad)
        } else {
            None
        }
    }

    pub fn set_key_pressed(&mut self, key: GbKeys, pressed: bool) {
        self.keys_pressed[key as usize] = pressed;
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
    use crate::mmu::Memory;

    use super::GbKeys;
    use super::Joypad;

    #[test]
    fn action_buttons() {
        let mut joy = Joypad::power_on();
        joy.write_byte(0xFF00, 0xDF);

        joy.set_key_pressed(GbKeys::A, true);
        assert_eq!(joy.update().is_some(), true);
        assert_eq!(joy.read_byte(0xFF00), 0b1101_1110);

        joy.set_key_pressed(GbKeys::A, false);
        joy.set_key_pressed(GbKeys::B, true);
        joy.set_key_pressed(GbKeys::Start, true);
        assert_eq!(
            joy.update().is_some(),
            true
        );
        assert_eq!(joy.read_byte(0xFF00), 0b1101_0101);

        joy.set_key_pressed(GbKeys::B, false);
        joy.set_key_pressed(GbKeys::Start, false);
        joy.set_key_pressed(GbKeys::Select, true);
        joy.set_key_pressed(GbKeys::A, true);
        joy.set_key_pressed(GbKeys::Down, true);
        assert_eq!(
            joy.update()
                .is_some(),
            true
        );
        assert_eq!(joy.read_byte(0xFF00), 0b1101_1010);

        assert_eq!(
            joy.update()
                .is_some(),
            false
        );
        assert_eq!(joy.read_byte(0xFF00), 0b1101_1010);

        joy.set_key_pressed(GbKeys::Select, false);
        joy.set_key_pressed(GbKeys::A, false);
        joy.set_key_pressed(GbKeys::Down, false);
        assert_eq!(joy.update().is_some(), false);
        assert_eq!(joy.read_byte(0xFF00), 0b1101_1111);
    }

    #[test]
    fn direction_buttons() {
        let mut joy = Joypad::power_on();
        joy.write_byte(0xFF00, 0xEF);

        joy.set_key_pressed(GbKeys::Up, true);
        assert_eq!(joy.update().is_some(), true);
        assert_eq!(joy.read_byte(0xFF00), 0b1110_1011);

        joy.set_key_pressed(GbKeys::Up, false);
        joy.set_key_pressed(GbKeys::Down, true);
        joy.set_key_pressed(GbKeys::Left, true);
        assert_eq!(
            joy.update().is_some(),
            true
        );
        assert_eq!(joy.read_byte(0xFF00), 0b1110_0101);

        joy.set_key_pressed(GbKeys::Down, false);
        joy.set_key_pressed(GbKeys::Left, false);
        joy.set_key_pressed(GbKeys::Right, true);
        joy.set_key_pressed(GbKeys::A, true);
        assert_eq!(
            joy.update().is_some(),
            true
        );
        assert_eq!(joy.read_byte(0xFF00), 0b1110_1110);

        assert_eq!(
            joy.update().is_some(),
            false
        );
        assert_eq!(joy.read_byte(0xFF00), 0b1110_1110);

        joy.set_key_pressed(GbKeys::Right, false);
        joy.set_key_pressed(GbKeys::A, false);
        assert_eq!(joy.update().is_some(), false);
        assert_eq!(joy.read_byte(0xFF00), 0b1110_1111);
    }

    #[test]
    fn action_direction_transition() {
        let mut joy = Joypad::power_on();
        joy.write_byte(0xFF00, 0xDF);

        joy.set_key_pressed(GbKeys::A, true);
        assert_eq!(joy.update().is_some(), true);
        assert_eq!(joy.read_byte(0xFF00), 0b1101_1110);

        joy.set_key_pressed(GbKeys::A, false);
        joy.set_key_pressed(GbKeys::B, true);
        joy.set_key_pressed(GbKeys::Start, true);
        assert_eq!(
            joy.update().is_some(),
            true
        );
        assert_eq!(joy.read_byte(0xFF00), 0b1101_0101);

        joy.set_key_pressed(GbKeys::B, false);
        joy.set_key_pressed(GbKeys::Start, false);
        joy.set_key_pressed(GbKeys::Select, true);
        joy.set_key_pressed(GbKeys::A, true);
        joy.set_key_pressed(GbKeys::Down, true);
        assert_eq!(
            joy.update()
                .is_some(),
            true
        );
        assert_eq!(joy.read_byte(0xFF00), 0b1101_1010);

        joy.write_byte(0xFF00, 0xEF);

        joy.set_key_pressed(GbKeys::Select, false);
        joy.set_key_pressed(GbKeys::A, false);
        joy.set_key_pressed(GbKeys::Down, false);
        joy.set_key_pressed(GbKeys::Up, true);
        assert_eq!(joy.update().is_some(), true);
        assert_eq!(joy.read_byte(0xFF00), 0b1110_1011);

        joy.set_key_pressed(GbKeys::Up, false);
        joy.set_key_pressed(GbKeys::Down, true);
        joy.set_key_pressed(GbKeys::Left, true);
        assert_eq!(
            joy.update().is_some(),
            true
        );
        assert_eq!(joy.read_byte(0xFF00), 0b1110_0101);

        joy.set_key_pressed(GbKeys::Down, false);
        joy.set_key_pressed(GbKeys::Left, false);
        joy.set_key_pressed(GbKeys::Right, true);
        joy.set_key_pressed(GbKeys::A, true);
        assert_eq!(
            joy.update().is_some(),
            true
        );
        assert_eq!(joy.read_byte(0xFF00), 0b1110_1110);

        joy.set_key_pressed(GbKeys::Right, false);
        joy.set_key_pressed(GbKeys::A, false);
        assert_eq!(joy.update().is_some(), false);
        assert_eq!(joy.read_byte(0xFF00), 0b1110_1111);
    }
}
