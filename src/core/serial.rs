use super::mmu::Memory;

struct SerialControl {
    /// Bit 7 - Transfer Start Flag (0=No Transfer, 1=Start)
    start_flag: bool,
    /// Bit 1 - Clock Speed (0=Normal, 1=Fast) ** CGB Mode Only **
    clock_speed: bool,
    /// Bit 0 - Shift Clock (0=External Clock, 1=Internal Clock)
    shift_clock: bool,
}

impl Memory for SerialControl {
    fn read_byte(&self, addr: u16) -> u8 {
        assert!(addr == 0xFF02);
        let mut v = 0;
        v |= (self.start_flag as u8) << 7;
        v |= (self.clock_speed as u8) << 1;
        v |= self.shift_clock as u8;
        v
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        assert!(addr == 0xFF02);
        self.start_flag = (val & 0x80) != 0x0;
        self.clock_speed = (val & 0x02) != 0x0;
        self.shift_clock = (val & 0x01) != 0x0;
    }
}

pub struct Serial {
    /// Serial transfer data: 8 Bits of data to be read/written
    sb: u8,
    ///
    sc: SerialControl,
}

impl Serial {
    pub fn power_on() -> Self {
        Serial {
            sb: 0,
            sc: SerialControl {
                start_flag: false,
                clock_speed: false,
                shift_clock: false,
            },
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
            0xFF02 => self.sc.read_byte(addr),
            _ => panic!("0x{:X}: Improper Serial Address", addr),
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF01 => self.sb = val,
            0xFF02 => self.sc.write_byte(addr, val),
            _ => panic!("0x{:X}: Improper Serial Address", addr),
        }
    }
}
