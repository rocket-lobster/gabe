#![allow(dead_code)]

use super::mmu::Memory;

pub struct Serial {
    /// Serial transfer data: 8 Bits of data to be read/written
    sb: u8,
    /// Bit 7 - Transfer Start Flag (0=No Transfer, 1=Start)
    /// Bit 1 - Clock Speed (0=Normal, 1=Fast) ** CGB Mode Only **
    /// Bit 0 - Shift Clock (0=External Clock, 1=Internal Clock)
    sc: u8,
}

impl Serial {
    pub fn power_on() -> Self {
        Serial {
            sb: 0,
            sc: 0,
        }
    }

    pub fn update(&mut self) {
        // TODO
    }
}

impl Memory for Serial {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF01 => self.sb,
            0xFF02 => self.sc,
            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF01 => self.sb = val,
            0xFF02 => self.sc = val,
            _ => unreachable!(),
        }
    }
}
