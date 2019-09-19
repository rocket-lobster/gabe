use super::interrupt::Interrupt;
use super::memory::Memory;

pub struct Timer {
    /// 0xFF04: Divider Register
    /// Increments at 16384 Hz, and wraps around. Resets to 0x00 when written to.
    div: u8,
    /// 0xFF05: Timer Counter
    /// Incremented at rate indicated by TAC register. When overflowed, it resets to
    /// the value of the TMA register and a Timer Interrupt is requested.
    tima: u8,
    /// 0xFF06: Timer Modulo
    /// TIMA is set to this value when the timer overflows
    tma: u8,
    /// 0xFF07: Timer Control
    /// Bit 2: 0 means stop the timer, 1 means start the timer
    /// Bit 1-0: Selects timer frequency
    ///
    ///     - 00: 4096 Hz
    ///     - 01: 262144 Hz
    ///     - 10: 65536 Hz
    ///     - 11: 16384 Hz
    tac: u8,
    /// Tracks the current cycles before incrementing DIV, increments at 256 cycles
    div_cycles: usize,
    /// Tracks the current cycles before incrementing TIMA, depends on TAC frequency
    tima_cycles: usize,
}

impl Timer {
    pub fn power_on() -> Self {
        Timer {
            div: 0x0,
            tima: 0x0,
            tma: 0x0,
            tac: 0x0,
            div_cycles: 0,
            tima_cycles: 0,
        }
    }

    /// Updates all the timer registers up to the same cycles as the CPU.
    /// Returns an Option with an Interrupt::Timer if the timer overflowed.
    pub fn update(&mut self, cycles: usize) -> Option<Interrupt> {
        // Update DIV timer
        self.div_cycles += cycles;
        if self.div_cycles >= 256 {
            self.div = self.div.wrapping_add(1);
            self.div_cycles -= 256;
        }
        // Update TIMA timer
        if !self.timer_stopped() {
            self.tima_cycles += cycles;
            if self.tima_cycles >= self.get_tima_freq() {
                self.tima = self.tima.wrapping_add(1);
                self.tima_cycles -= self.get_tima_freq();
                if self.tima == 0x0 {
                    self.tima = self.tma;
                    return Some(Interrupt::Timer);
                }
            }
        }
        None
    }

    /// Reads the value of the TAC register and returns the number of
    /// CPU cycles needed before incrementing the TIMA register
    fn get_tima_freq(&self) -> usize {
        match self.tac & 0b11 {
            0b00 => 1024,
            0b01 => 16,
            0b10 => 64,
            0b11 => 256,
            _ => panic!(""),
        }
    }

    fn timer_stopped(&self) -> bool {
        ((self.tac >> 2) & 0b1) != 0b1
    }
}

impl Memory for Timer {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF04 => self.div,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,
            _ => panic!("0x{:X}: Improper Timer Address", addr),
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0xFF04 => self.div = 0x0,
            0xFF05 => self.tima = val,
            0xFF06 => self.tma = val,
            0xFF07 => self.tac = val,
            _ => panic!("0x{:X}: Improper Timer Address", addr),
        }
    }
}
