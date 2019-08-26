use super::mmu;
use super::memory::Memory;

/// Represents all the registers in use by the Gameboy CPU.
/// Consists of 16-bit register pairs that can be accessed as 8-bit
/// high and low registers and as combined 16-bit values
/// Paired as follows:
/// - AF
/// - BC
/// - DE
/// - HL
///
/// Also contains two other 16-bit registers:
/// - PC (Program Counter)
/// - SP (Stack Pointer)
#[derive(Clone, Default)]
struct Registers {
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,
    pub sp: u16,
    pub pc: u16,
}

impl Registers {
    /// Initializes the state of the Registers of the CPU
    /// Simulates the state of the CPU post-BIOS and right before running
    /// user code
    fn power_on() -> Self {
        // Default to all zeros
        let mut reg = Self::default();

        // Simulate BIOS procedure that initializes values
        reg.a = 0x01;
        reg.f = 0xB0;
        reg.b = 0x00;
        reg.c = 0x13;
        reg.d = 0x00;
        reg.e = 0xD8;
        reg.h = 0x01;
        reg.l = 0x4D;
        reg.sp = 0xFFFE;

        // Start at memory location 0x0100 after running the BIOS procedure
        // This is where actual ROM game code begins
        reg.pc = 0x0100;
        reg
    }

    /// Returns a 16-bit value where 
    /// A is the hi 8-bits and F is the lo 8-bits
    fn get_af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f as u16)
    }

    /// Returns a 16-bit value where 
    /// B is the hi 8-bits and C is the lo 8-bits
    fn get_bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    /// Returns a 16-bit value where 
    /// D is the hi 8-bits and E is the lo 8-bits
    fn get_de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    /// Returns a 16-bit value where 
    /// H is the hi 8-bits and L is the lo 8-bits
    fn get_hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    /// Sets a 16-bit value where 
    /// A is the hi 8-bits and F is the lo 8-bits
    fn set_af(&mut self, val: u16) {
        // TODO: Probably shouldn't use this logic for
        // setting F?
        self.a = (val >> 8) as u8;
        self.f = (val & 0xFF) as u8;
    }

    /// Sets a 16-bit value where 
    /// B is the hi 8-bits and C is the lo 8-bits
    fn set_bc(&mut self, val: u16) {
        self.b = (val >> 8) as u8;
        self.c = (val & 0xFF) as u8;
    }

    /// Sets a 16-bit value where 
    /// D is the hi 8-bits and E is the lo 8-bits
    fn set_de(&mut self, val: u16) {
        self.d = (val >> 8) as u8;
        self.e = (val & 0xFF) as u8;
    }

    /// Sets a 16-bit value where 
    /// H is the hi 8-bits and L is the lo 8-bits
    fn set_hl(&mut self, val: u16) {
        self.h = (val >> 8) as u8;
        self.l = (val & 0xFF) as u8;
    }
}

/// The CPU contains Register state and is responsible for
/// decoding each opcode at the current PC and updating 
/// the Registers and MMU when appropriate.
pub struct Cpu {
    reg: Registers,
}

impl Cpu {
    /// Initializes CPU internal state and returns a handle to the
    /// initialized Cpu struct.
    pub fn power_on() -> Self {
        Cpu {
            reg: Registers::power_on(),
        }
    }

    /// Fetches a single instruction opcode, decodes the opcode to the 
    /// appropriate function, and executes the functionality.
    /// Returns the number of cycles executed.
    pub fn tick(&mut self, mmu: &mut mmu::Mmu) -> usize {
        let opcode = mmu.read_byte(self.reg.pc);
        self.reg.pc += 1;
        match opcode {
            _ => panic!("Unsupported or unimplemented opcode {}", opcode)
        }
        0
    }
}

#[cfg(test)]
mod cpu_tests {
    use super::Registers;
    #[test]
    fn register_read() {
        let reg = Registers::power_on();

        // Verify power-on values
        assert_eq!(reg.a, 0x01);
        assert_eq!(reg.f, 0xB0);
        assert_eq!(reg.b, 0x00);
        assert_eq!(reg.c, 0x13);
        assert_eq!(reg.d, 0x00);
        assert_eq!(reg.e, 0xD8);
        assert_eq!(reg.h, 0x01);
        assert_eq!(reg.l, 0x4D);
        assert_eq!(reg.sp, 0xFFFE);
        assert_eq!(reg.pc, 0x0100);

        // Use register pair accessors
        assert_eq!(reg.get_af(), 0x01B0);
        assert_eq!(reg.get_bc(), 0x0013);
        assert_eq!(reg.get_de(), 0x00D8);
        assert_eq!(reg.get_hl(), 0x014D);
    }

    #[test]
    fn register_write() {
        let mut reg = Registers::power_on();

        // Set register pair values
        reg.set_af(0x1234);
        reg.set_bc(0x5678);
        reg.set_de(0x9001);
        reg.set_hl(0x2345);
        assert_eq!(reg.a, 0x12);
        assert_eq!(reg.f, 0x34);
        assert_eq!(reg.b, 0x56);
        assert_eq!(reg.c, 0x78);
        assert_eq!(reg.d, 0x90);
        assert_eq!(reg.e, 0x01);
        assert_eq!(reg.h, 0x23);
        assert_eq!(reg.l, 0x45);
    }
}