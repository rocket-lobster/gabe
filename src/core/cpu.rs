use crate::core::interrupt::InterruptKind;

use super::memory::Memory;
use super::mmu;
use std::fmt;

/// The register F holds flag information that are set by ALU
/// operations. Conditional operations check these flags afterwards.
enum Flag {
    /// Zero flag is set when operations result in zero values
    Z = 0b1000_0000,
    /// Negative flag is set when a subtraction operation is performed
    N = 0b0100_0000,
    /// Half-carry flag is set when an operation creates a carry bit from bit 3 to 4.
    H = 0b0010_0000,
    /// Carry flag is set when an operation creates a carry bit from bit 7.
    C = 0b0001_0000,
}

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
        (u16::from(self.a) << 8) | u16::from(self.f)
    }

    /// Returns a 16-bit value where
    /// B is the hi 8-bits and C is the lo 8-bits
    fn get_bc(&self) -> u16 {
        (u16::from(self.b) << 8) | u16::from(self.c)
    }

    /// Returns a 16-bit value where
    /// D is the hi 8-bits and E is the lo 8-bits
    fn get_de(&self) -> u16 {
        (u16::from(self.d) << 8) | u16::from(self.e)
    }

    /// Returns a 16-bit value where
    /// H is the hi 8-bits and L is the lo 8-bits
    fn get_hl(&self) -> u16 {
        (u16::from(self.h) << 8) | u16::from(self.l)
    }

    /// Sets a 16-bit value where
    /// A is the hi 8-bits and F is the lo 8-bits
    fn set_af(&mut self, val: u16) {
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

    fn set_flag(&mut self, f: Flag, v: bool) {
        if v {
            self.f |= f as u8;
        } else {
            self.f &= !(f as u8);
        }
    }

    fn get_flag(&self, f: Flag) -> bool {
        (self.f & (f as u8)) != 0
    }
}

/// Tables of opcode cycle counts.
/// Skipped when running rustfmt
#[rustfmt::skip]
const OPCODE_TABLE: [usize; 256] = [
//  0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
    4,12, 8, 8, 4, 4, 8, 4,20, 8, 8, 8, 4, 4, 8, 4, // 0
    4,12, 8, 8, 4, 4, 8, 4,12, 8, 8, 8, 4, 4, 8, 4, // 1
    8,12, 8, 8, 4, 4, 8, 4, 8, 8, 8, 8, 4, 4, 8, 4, // 2
    8,12, 8, 8,12,12,12, 4, 8, 8, 8, 8, 4, 4, 8, 4, // 3
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 4
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 5
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 6
    8, 8, 8, 8, 8, 8, 4, 8, 4, 4, 4, 4, 4, 4, 8, 4, // 7
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 8
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // 9
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // A
    4, 4, 4, 4, 4, 4, 8, 4, 4, 4, 4, 4, 4, 4, 8, 4, // B
    8,12,12,16,12,16, 8,16, 8,16,12, 4,12,24, 8,16, // C
    8,12,12, 0,12,16, 8,16, 8,16,12, 0,12, 0, 8,16, // D
   12,12, 8, 0, 0,16, 8,16,16, 4,16, 0, 0, 0, 8,16, // E
   12,12, 8, 4, 0,16, 8,16,12, 8,16, 4, 0, 0, 8,16, // F
];

/// Tables of opcode cycle counts for extended opcodes.
/// Skipped when running rustfmt
#[rustfmt::skip]
const OPCODE_CB_TABLE: [usize; 256] = [
//  0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
    8, 8, 8, 8, 8, 8,16, 8, 8, 8, 8, 8, 8, 8,16, 8, // 0
    8, 8, 8, 8, 8, 8,16, 8, 8, 8, 8, 8, 8, 8,16, 8, // 1
    8, 8, 8, 8, 8, 8,16, 8, 8, 8, 8, 8, 8, 8,16, 8, // 2
    8, 8, 8, 8, 8, 8,16, 8, 8, 8, 8, 8, 8, 8,16, 8, // 3
    8, 8, 8, 8, 8, 8,16, 8, 8, 8, 8, 8, 8, 8,16, 8, // 4
    8, 8, 8, 8, 8, 8,16, 8, 8, 8, 8, 8, 8, 8,16, 8, // 5
    8, 8, 8, 8, 8, 8,16, 8, 8, 8, 8, 8, 8, 8,16, 8, // 6
    8, 8, 8, 8, 8, 8,16, 8, 8, 8, 8, 8, 8, 8,16, 8, // 7
    8, 8, 8, 8, 8, 8,16, 8, 8, 8, 8, 8, 8, 8,16, 8, // 8
    8, 8, 8, 8, 8, 8,16, 8, 8, 8, 8, 8, 8, 8,16, 8, // 9
    8, 8, 8, 8, 8, 8,16, 8, 8, 8, 8, 8, 8, 8,16, 8, // A
    8, 8, 8, 8, 8, 8,16, 8, 8, 8, 8, 8, 8, 8,16, 8, // B
    8, 8, 8, 8, 8, 8,16, 8, 8, 8, 8, 8, 8, 8,16, 8, // C
    8, 8, 8, 8, 8, 8,16, 8, 8, 8, 8, 8, 8, 8,16, 8, // D
    8, 8, 8, 8, 8, 8,16, 8, 8, 8, 8, 8, 8, 8,16, 8, // E
    8, 8, 8, 8, 8, 8,16, 8, 8, 8, 8, 8, 8, 8,16, 8, // F
];

const OPCODE_STRINGS: [&str; 256] = [
    "NOP",
    "LD BC,d16",
    "LD (BC),A",
    "INC BC",
    "INC B",
    "DEC B",
    "LD B,d8",
    "RLCA",
    "LD (a16),SP",
    "ADD HL,BC",
    "LD A,(BC)",
    "DEC BC",
    "INC C",
    "DEC C",
    "LD C,d8",
    "RRCA",
    "STOP 0",
    "LD DE,d16",
    "LD (DE),A",
    "INC DE",
    "INC D",
    "DEC D",
    "LD D,d8",
    "RLA",
    "JR r8",
    "ADD HL,DE",
    "LD A,(DE)",
    "DEC DE",
    "INC E",
    "DEC E",
    "LD E,d8",
    "RRA",
    "JR NZ,r8",
    "LD HL,d16",
    "LD (HL+),A",
    "INC HL",
    "INC H",
    "DEC H",
    "LD H,d8",
    "DAA",
    "JR Z,r8",
    "ADD HL,HL",
    "LD A,(HL+)",
    "DEC HL",
    "INC L",
    "DEC L",
    "LD L,d8",
    "CPL",
    "JR NC,r8",
    "LD SP,d16",
    "LD (HL-),A",
    "INC SP",
    "INC (HL)",
    "DEC (HL)",
    "LD (HL),d8",
    "SCF",
    "JR C,r8",
    "ADD HL,SP",
    "LD A,(HL-)",
    "DEC SP",
    "INC A",
    "DEC A",
    "LD A,d8",
    "CCF",
    "LD B,B",
    "LD B,C",
    "LD B,D",
    "LD B,E",
    "LD B,H",
    "LD B,L",
    "LD B,(HL)",
    "LD B,A",
    "LD C,B",
    "LD C,C",
    "LD C,D",
    "LD C,E",
    "LD C,H",
    "LD C,L",
    "LD C,(HL)",
    "LD C,A",
    "LD D,B",
    "LD D,C",
    "LD D,D",
    "LD D,E",
    "LD D,H",
    "LD D,L",
    "LD D,(HL)",
    "LD D,A",
    "LD E,B",
    "LD E,C",
    "LD E,D",
    "LD E,E",
    "LD E,H",
    "LD E,L",
    "LD E,(HL)",
    "LD E,A",
    "LD H,B",
    "LD H,C",
    "LD H,D",
    "LD H,E",
    "LD H,H",
    "LD H,L",
    "LD H,(HL)",
    "LD H,A",
    "LD L,B",
    "LD L,C",
    "LD L,D",
    "LD L,E",
    "LD L,H",
    "LD L,L",
    "LD L,(HL)",
    "LD L,A",
    "LD (HL),B",
    "LD (HL),C",
    "LD (HL),D",
    "LD (HL),E",
    "LD (HL),H",
    "LD (HL),L",
    "HALT",
    "LD (HL),A",
    "LD A,B",
    "LD A,C",
    "LD A,D",
    "LD A,E",
    "LD A,H",
    "LD A,L",
    "LD A,(HL)",
    "LD A,A",
    "ADD A,B",
    "ADD A,C",
    "ADD A,D",
    "ADD A,E",
    "ADD A,H",
    "ADD A,L",
    "ADD A,(HL)",
    "ADD A,A",
    "ADC A,B",
    "ADC A,C",
    "ADC A,D",
    "ADC A,E",
    "ADC A,H",
    "ADC A,L",
    "ADC A,(HL)",
    "ADC A,A",
    "SUB A,B",
    "SUB A,C",
    "SUB A,D",
    "SUB A,E",
    "SUB A,H",
    "SUB A,L",
    "SUB A,(HL)",
    "SUB A,A",
    "SBC A,B",
    "SBC A,C",
    "SBC A,D",
    "SBC A,E",
    "SBC A,H",
    "SBC A,L",
    "SBC A,(HL)",
    "SBC A,A",
    "AND B",
    "AND C",
    "AND D",
    "AND E",
    "AND H",
    "AND L",
    "AND (HL)",
    "AND A",
    "XOR B",
    "XOR C",
    "XOR D",
    "XOR E",
    "XOR H",
    "XOR L",
    "XOR (HL)",
    "XOR A",
    "OR B",
    "OR C",
    "OR D",
    "OR E",
    "OR H",
    "OR L",
    "OR (HL)",
    "OR A",
    "CP B",
    "CP C",
    "CP D",
    "CP E",
    "CP H",
    "CP L",
    "CP (HL)",
    "CP A",
    "RET NZ",
    "POP BC",
    "JP NZ,a16",
    "JP a16",
    "CALL NZ,a16",
    "PUSH BC",
    "ADD A,d8",
    "RST 00H",
    "RET Z",
    "RET",
    "JP Z,a16",
    "CB ",
    "CALL Z,a16",
    "CALL a16",
    "ADC A,d8",
    "RST 08H",
    "RET NC",
    "POP DE",
    "JP NC,a16",
    "NULL",
    "CALL NC,a16",
    "PUSH DE",
    "SUB d8",
    "RST 10H",
    "RET C",
    "RETI",
    "JP C,a16",
    "NULL",
    "CALL C,a16",
    "NULL",
    "SBC A,d8",
    "RST 18H",
    "LDH (a8),A",
    "POP HL",
    "LD (C),A",
    "NULL",
    "NULL",
    "PUSH HL",
    "AND d8",
    "RST 20H",
    "ADD SP,r8",
    "JP (HL)",
    "JP (a16),A",
    "NULL",
    "NULL",
    "NULL",
    "XOR d8",
    "RST 28H",
    "LDH A,(a8)",
    "POP AF",
    "LD A,(C)",
    "DI",
    "NULL",
    "PUSH AF",
    "OR d8",
    "RST 30H",
    "LD HL,SP+r8",
    "LD SP,HL",
    "JP A,(a16)",
    "EI",
    "NULL",
    "NULL",
    "CP d8",
    "RST 38H",
];

/// The CPU contains Register state and is responsible for
/// decoding each opcode at the current PC and updating
/// the Registers and MMU when appropriate.
#[derive(Clone)]
pub struct Cpu {
    reg: Registers,
    ime: bool,
    halted: bool,
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Implement printing for use in TUI
        writeln!(
            f,
            "A:    {:02X}    AF:     {:04X}",
            self.reg.a,
            self.reg.get_af()
        )?;
        writeln!(
            f,
            "B:    {:02X}    BC:     {:04X}",
            self.reg.b,
            self.reg.get_bc()
        )?;
        writeln!(
            f,
            "C:    {:02X}    DE:     {:04X}",
            self.reg.c,
            self.reg.get_de()
        )?;
        writeln!(
            f,
            "D:    {:02X}    HL:     {:04X}",
            self.reg.d,
            self.reg.get_hl()
        )?;
        writeln!(f, "E:    {:02X}", self.reg.e)?;
        writeln!(f, "H:    {:02X}", self.reg.h)?;
        writeln!(f, "L:    {:02X}", self.reg.l)?;
        writeln!(f, "F:    {:02X}", self.reg.f)?;
        writeln!(f, "IME:    {}", self.ime)?;
        writeln!(f, "Flags:")?;
        writeln!(f, "   Z:   {}", self.reg.get_flag(Flag::Z))?;
        writeln!(f, "   N:   {}", self.reg.get_flag(Flag::N))?;
        writeln!(f, "   H:   {}", self.reg.get_flag(Flag::H))?;
        writeln!(f, "   C:   {}", self.reg.get_flag(Flag::C))
    }
}

impl Cpu {
    /// Initializes CPU internal state and returns a handle to the
    /// initialized Cpu struct.
    pub fn power_on() -> Self {
        Cpu {
            reg: Registers::power_on(),
            ime: false,
            halted: false,
        }
    }

    pub fn get_debug_data(&mut self) -> Cpu {
        self.clone()
    }

    fn check_interrupts(&mut self, mmu: &mut mmu::Mmu) -> Option<usize> {
        // Check if any enabled interrupts were requested
        let mut interrupt_reqs = mmu.read_byte(0xFF0F);
        let interrupt_enables = mmu.read_byte(0xFFFF);
        let interrupt_result = interrupt_reqs & interrupt_enables;
        if interrupt_result == 0x0 {
            // No interrupts were both requested and enabled
            None
        } else {
            // If we're halted, exit on an interrupt
            self.halted = false;
            if !self.ime {
                // No longer halted, exit if we cannot handle interrupts
                None
            } else {
                if (interrupt_result & InterruptKind::VBlank as u8) != 0x0 {
                    // V-Blank interrupt
                    // Reset the request flag to the interrupt
                    interrupt_reqs &= !(InterruptKind::VBlank as u8);
                    mmu.write_byte(0xFF0F, interrupt_reqs);

                    // Run CALL on V-Blank procedure
                    self.stack_push(mmu, self.reg.pc);
                    self.reg.pc = 0x40;
                } else if (interrupt_result & InterruptKind::LcdStat as u8) != 0x0 {
                    // LCD STAT Interrupt
                    // Reset the request flag to the interrupt
                    interrupt_reqs &= !(InterruptKind::LcdStat as u8);
                    mmu.write_byte(0xFF0F, interrupt_reqs);

                    // Run CALL on LCD Stat procedure
                    self.stack_push(mmu, self.reg.pc);
                    self.reg.pc = 0x48;
                } else if (interrupt_result & InterruptKind::Timer as u8) != 0x0 {
                    // Timer Interrupt
                    // Reset the request flag to the interrupt
                    interrupt_reqs &= !(InterruptKind::Timer as u8);
                    mmu.write_byte(0xFF0F, interrupt_reqs);

                    // Run CALL on Timer procedure
                    self.stack_push(mmu, self.reg.pc);
                    self.reg.pc = 0x50;
                } else if (interrupt_result & InterruptKind::Serial as u8) != 0x0 {
                    // Serial Interrupt
                    // Reset the request flag to the interrupt
                    interrupt_reqs &= !(InterruptKind::Serial as u8);
                    mmu.write_byte(0xFF0F, interrupt_reqs);

                    // Run CALL on Serial procedure
                    self.stack_push(mmu, self.reg.pc);
                    self.reg.pc = 0x58;
                } else if (interrupt_result & InterruptKind::Joypad as u8) != 0x0 {
                    // Joypad Interrupt
                    // Reset the request flag to the interrupt
                    interrupt_reqs &= !(InterruptKind::Joypad as u8);
                    mmu.write_byte(0xFF0F, interrupt_reqs);

                    // Run CALL on Joypad procedure
                    self.stack_push(mmu, self.reg.pc);
                    self.reg.pc = 0x60;
                }
                // We're executing a interrupt procedure, disable all interrupts and 
                // return cycles for a CALL procedure.
                self.ime = false;
                Some(16)
            }
        }
    }

    /// Fetches a single instruction opcode, decodes the opcode to the
    /// appropriate function, and executes the functionality.
    /// Returns the number of cycles executed.
    pub fn tick(&mut self, mmu: &mut mmu::Mmu) -> usize {

        if self.ime || self.halted {
            // If CPU is halted or IME is enabled, check if there's any interrupts to execute
            if let Some(c) = self.check_interrupts(mmu) {
                // Running interrupt routine, return cycles
                return c;
            }
        }

        if self.halted {
            // Check if still halted after running interrupt checks
            return OPCODE_TABLE[0];
        }
        
        let old_pc = self.reg.pc;
        let mut opcode = self.imm(mmu);
        let mut using_cb: bool = false;
        trace!(
            "0x{:04X}: 0x{:02X} {}",
            old_pc,
            opcode,
            OPCODE_STRINGS[opcode as usize]
        );
        // Use more cycles when following conditional branches,
        // set when conditionals are met.
        let mut cond_cycles: usize = 0;
        match opcode {
            // NOP
            0x00 => (),

            // HALT
            0x76 => self.halted = true,

            // STOP
            0x10 => unimplemented!("STOP not implemented"),

            // IME
            0xF3 => self.ime = false,
            0xFB => self.ime = true,

            // LD r8,d8
            0x06 => self.reg.b = self.imm(mmu),
            0x0E => self.reg.c = self.imm(mmu),
            0x16 => self.reg.d = self.imm(mmu),
            0x1E => self.reg.e = self.imm(mmu),
            0x26 => self.reg.h = self.imm(mmu),
            0x2E => self.reg.l = self.imm(mmu),
            0x36 => {
                let v = self.imm(mmu);
                mmu.write_byte(self.reg.get_hl(), v);
            }
            0x3E => self.reg.a = self.imm(mmu),

            // LD (r16),A
            0x02 => mmu.write_byte(self.reg.get_bc(), self.reg.a),
            0x12 => mmu.write_byte(self.reg.get_de(), self.reg.a),

            // LD A,(r16)
            0x0a => self.reg.a = mmu.read_byte(self.reg.get_bc()),
            0x1a => self.reg.a = mmu.read_byte(self.reg.get_de()),

            // LD (HL+),A
            0x22 => {
                let v = self.reg.get_hl();
                mmu.write_byte(v, self.reg.a);
                self.reg.set_hl(v + 1);
            }

            // LD (HL-),A
            0x32 => {
                let v = self.reg.get_hl();
                mmu.write_byte(v, self.reg.a);
                self.reg.set_hl(v - 1);
            }

            // LD A,(HL+)
            0x2a => {
                let v = self.reg.get_hl();
                self.reg.a = mmu.read_byte(v);
                self.reg.set_hl(v + 1);
            }

            // LD A,(HL-)
            0x3a => {
                let v = self.reg.get_hl();
                self.reg.a = mmu.read_byte(v);
                self.reg.set_hl(v - 1);
            }

            // LDH (a8),A
            0xE0 => {
                let addr = 0xFF00 + u16::from(self.imm(mmu));
                mmu.write_byte(addr, self.reg.a);
            }
            // LDH A,(a8)
            0xF0 => {
                let addr = 0xFF00 + u16::from(self.imm(mmu));
                self.reg.a = mmu.read_byte(addr);
            }

            // LD (C),A
            0xE2 => {
                let addr = 0xFF00 + u16::from(self.reg.c);
                mmu.write_byte(addr, self.reg.a);
            }
            // LD A,(C)
            0xF2 => {
                let addr = 0xFF00 + u16::from(self.reg.c);
                self.reg.a = mmu.read_byte(addr);
            }

            // LD r8,r8
            0x40 => (),
            0x41 => self.reg.b = self.reg.c,
            0x42 => self.reg.b = self.reg.d,
            0x43 => self.reg.b = self.reg.e,
            0x44 => self.reg.b = self.reg.h,
            0x45 => self.reg.b = self.reg.l,
            0x46 => self.reg.b = mmu.read_byte(self.reg.get_hl()),
            0x47 => self.reg.b = self.reg.a,
            0x48 => self.reg.c = self.reg.b,
            0x49 => (),
            0x4A => self.reg.c = self.reg.d,
            0x4B => self.reg.c = self.reg.e,
            0x4C => self.reg.c = self.reg.h,
            0x4D => self.reg.c = self.reg.l,
            0x4E => self.reg.c = mmu.read_byte(self.reg.get_hl()),
            0x4F => self.reg.c = self.reg.a,
            0x50 => self.reg.d = self.reg.b,
            0x51 => self.reg.d = self.reg.c,
            0x52 => (),
            0x53 => self.reg.d = self.reg.e,
            0x54 => self.reg.d = self.reg.h,
            0x55 => self.reg.d = self.reg.l,
            0x56 => self.reg.d = mmu.read_byte(self.reg.get_hl()),
            0x57 => self.reg.d = self.reg.a,
            0x58 => self.reg.e = self.reg.b,
            0x59 => self.reg.e = self.reg.c,
            0x5A => self.reg.e = self.reg.d,
            0x5B => (),
            0x5C => self.reg.e = self.reg.h,
            0x5D => self.reg.e = self.reg.l,
            0x5E => self.reg.e = mmu.read_byte(self.reg.get_hl()),
            0x5F => self.reg.e = self.reg.a,
            0x60 => self.reg.h = self.reg.b,
            0x61 => self.reg.h = self.reg.c,
            0x62 => self.reg.h = self.reg.d,
            0x63 => self.reg.h = self.reg.e,
            0x64 => (),
            0x65 => self.reg.h = self.reg.l,
            0x66 => self.reg.h = mmu.read_byte(self.reg.get_hl()),
            0x67 => self.reg.h = self.reg.a,
            0x68 => self.reg.l = self.reg.b,
            0x69 => self.reg.l = self.reg.c,
            0x6A => self.reg.l = self.reg.d,
            0x6B => self.reg.l = self.reg.e,
            0x6C => self.reg.l = self.reg.h,
            0x6D => (),
            0x6E => self.reg.l = mmu.read_byte(self.reg.get_hl()),
            0x6F => self.reg.l = self.reg.a,
            0x70 => mmu.write_byte(self.reg.get_hl(), self.reg.b),
            0x71 => mmu.write_byte(self.reg.get_hl(), self.reg.c),
            0x72 => mmu.write_byte(self.reg.get_hl(), self.reg.d),
            0x73 => mmu.write_byte(self.reg.get_hl(), self.reg.e),
            0x74 => mmu.write_byte(self.reg.get_hl(), self.reg.h),
            0x75 => mmu.write_byte(self.reg.get_hl(), self.reg.l),
            0x77 => mmu.write_byte(self.reg.get_hl(), self.reg.a),
            0x78 => self.reg.a = self.reg.b,
            0x79 => self.reg.a = self.reg.c,
            0x7A => self.reg.a = self.reg.d,
            0x7B => self.reg.a = self.reg.e,
            0x7C => self.reg.a = self.reg.h,
            0x7D => self.reg.a = self.reg.l,
            0x7E => self.reg.a = mmu.read_byte(self.reg.get_hl()),
            0x7F => (),

            // LD r16,d16
            0x01 => {
                let v = self.imm_word(mmu);
                self.reg.set_bc(v);
            }
            0x11 => {
                let v = self.imm_word(mmu);
                self.reg.set_de(v);
            }
            0x21 => {
                let v = self.imm_word(mmu);
                self.reg.set_hl(v);
            }
            0x31 => {
                let v = self.imm_word(mmu);
                self.reg.sp = v;
            }

            // LD (a16),A
            0xEA => {
                let v = self.imm_word(mmu);
                mmu.write_byte(v, self.reg.a);
            }

            // LD A,(a16)
            0xFA => {
                let v = self.imm_word(mmu);
                self.reg.a = mmu.read_byte(v);
            }

            // LD (a16),SP
            0x08 => {
                let v = self.imm_word(mmu);
                mmu.write_word(v, self.reg.sp);
            }

            // LD SP,HL
            0xF9 => self.reg.sp = self.reg.get_hl(),

            // ADD A,r8
            0x80 => self.add(self.reg.b),
            0x81 => self.add(self.reg.c),
            0x82 => self.add(self.reg.d),
            0x83 => self.add(self.reg.e),
            0x84 => self.add(self.reg.h),
            0x85 => self.add(self.reg.l),
            0x86 => self.add(mmu.read_byte(self.reg.get_hl())),
            0x87 => self.add(self.reg.a),

            // ADD A,d8
            0xC6 => {
                let v = self.imm(mmu);
                self.add(v);
            }

            // ADC A,r8
            0x88 => self.adc(self.reg.b),
            0x89 => self.adc(self.reg.c),
            0x8A => self.adc(self.reg.d),
            0x8B => self.adc(self.reg.e),
            0x8C => self.adc(self.reg.h),
            0x8D => self.adc(self.reg.l),
            0x8E => self.adc(mmu.read_byte(self.reg.get_hl())),
            0x8F => self.adc(self.reg.a),

            // ADC A,d8
            0xCE => {
                let v = self.imm(mmu);
                self.adc(v);
            }

            // ADD SP,r8
            0xE8 => self.add_sp(mmu),

            // ADD HL,r16
            0x09 => self.add_hl(self.reg.get_bc()),
            0x19 => self.add_hl(self.reg.get_de()),
            0x29 => self.add_hl(self.reg.get_hl()),
            0x39 => self.add_hl(self.reg.sp),

            // SUB r8
            0x90 => self.sub(self.reg.b),
            0x91 => self.sub(self.reg.c),
            0x92 => self.sub(self.reg.d),
            0x93 => self.sub(self.reg.e),
            0x94 => self.sub(self.reg.h),
            0x95 => self.sub(self.reg.l),
            0x96 => self.sub(mmu.read_byte(self.reg.get_hl())),
            0x97 => self.sub(self.reg.a),

            // SUB d8
            0xD6 => {
                let v = self.imm(mmu);
                self.sub(v);
            }

            // SBC r8,r8
            0x98 => self.sbc(self.reg.b),
            0x99 => self.sbc(self.reg.c),
            0x9A => self.sbc(self.reg.d),
            0x9B => self.sbc(self.reg.e),
            0x9C => self.sbc(self.reg.h),
            0x9D => self.sbc(self.reg.l),
            0x9E => self.sbc(mmu.read_byte(self.reg.get_hl())),
            0x9F => self.sbc(self.reg.a),

            // SBC d8
            0xDE => {
                let v = self.imm(mmu);
                self.sbc(v);
            }

            // AND r8
            0xA0 => self.and(self.reg.b),
            0xA1 => self.and(self.reg.c),
            0xA2 => self.and(self.reg.d),
            0xA3 => self.and(self.reg.e),
            0xA4 => self.and(self.reg.h),
            0xA5 => self.and(self.reg.l),
            0xA6 => self.and(mmu.read_byte(self.reg.get_hl())),
            0xA7 => self.and(self.reg.a),

            // AND d8
            0xE6 => {
                let v = self.imm(mmu);
                self.and(v);
            }

            // XOR r8
            0xA8 => self.xor(self.reg.b),
            0xA9 => self.xor(self.reg.c),
            0xAA => self.xor(self.reg.d),
            0xAB => self.xor(self.reg.e),
            0xAC => self.xor(self.reg.h),
            0xAD => self.xor(self.reg.l),
            0xAE => self.xor(mmu.read_byte(self.reg.get_hl())),
            0xAF => self.xor(self.reg.a),

            // XOR d8
            0xEE => {
                let v = self.imm(mmu);
                self.xor(v);
            }

            // OR r8
            0xB0 => self.or(self.reg.b),
            0xB1 => self.or(self.reg.c),
            0xB2 => self.or(self.reg.d),
            0xB3 => self.or(self.reg.e),
            0xB4 => self.or(self.reg.h),
            0xB5 => self.or(self.reg.l),
            0xB6 => self.or(mmu.read_byte(self.reg.get_hl())),
            0xB7 => self.or(self.reg.a),

            // OR d8
            0xF6 => {
                let v = self.imm(mmu);
                self.or(v);
            }

            // CP r8
            0xB8 => self.cp(self.reg.b),
            0xB9 => self.cp(self.reg.c),
            0xBA => self.cp(self.reg.d),
            0xBB => self.cp(self.reg.e),
            0xBC => self.cp(self.reg.h),
            0xBD => self.cp(self.reg.l),
            0xBE => self.cp(mmu.read_byte(self.reg.get_hl())),
            0xBF => self.cp(self.reg.a),

            // CP d8
            0xFE => {
                let v = self.imm(mmu);
                self.cp(v);
            }

            // INC r8
            0x04 => self.reg.b = self.inc(self.reg.b),
            0x0C => self.reg.c = self.inc(self.reg.c),
            0x14 => self.reg.d = self.inc(self.reg.d),
            0x1C => self.reg.e = self.inc(self.reg.e),
            0x24 => self.reg.h = self.inc(self.reg.h),
            0x2C => self.reg.l = self.inc(self.reg.l),
            0x34 => {
                let v = self.inc(mmu.read_byte(self.reg.get_hl()));
                mmu.write_byte(self.reg.get_hl(), v);
            }
            0x3C => self.reg.a = self.inc(self.reg.a),

            // DEC r8
            0x05 => self.reg.b = self.dec(self.reg.b),
            0x0D => self.reg.c = self.dec(self.reg.c),
            0x15 => self.reg.d = self.dec(self.reg.d),
            0x1D => self.reg.e = self.dec(self.reg.e),
            0x25 => self.reg.h = self.dec(self.reg.h),
            0x2D => self.reg.l = self.dec(self.reg.l),
            0x35 => {
                let v = self.dec(mmu.read_byte(self.reg.get_hl()));
                mmu.write_byte(self.reg.get_hl(), v);
            }
            0x3D => self.reg.a = self.dec(self.reg.a),

            // INC r16
            0x03 => self.reg.set_bc(self.reg.get_bc().wrapping_add(1)),
            0x13 => self.reg.set_de(self.reg.get_de().wrapping_add(1)),
            0x23 => self.reg.set_hl(self.reg.get_hl().wrapping_add(1)),
            0x33 => self.reg.sp = self.reg.sp.wrapping_add(1),

            // DEC r16
            0x0B => self.reg.set_bc(self.reg.get_bc().wrapping_sub(1)),
            0x1B => self.reg.set_de(self.reg.get_de().wrapping_sub(1)),
            0x2B => self.reg.set_hl(self.reg.get_hl().wrapping_sub(1)),
            0x3B => self.reg.sp = self.reg.sp.wrapping_sub(1),

            // POP r16
            0xC1 => {
                let v = self.stack_pop(mmu);
                self.reg.set_bc(v);
            }
            0xD1 => {
                let v = self.stack_pop(mmu);
                self.reg.set_de(v);
            }
            0xE1 => {
                let v = self.stack_pop(mmu);
                self.reg.set_hl(v);
            }
            0xF1 => {
                let v = self.stack_pop(mmu);
                self.reg.set_af(v);
            }

            // PUSH r16
            0xC5 => {
                let v = self.reg.get_bc();
                self.stack_push(mmu, v);
            }
            0xD5 => {
                let v = self.reg.get_de();
                self.stack_push(mmu, v);
            }
            0xE5 => {
                let v = self.reg.get_hl();
                self.stack_push(mmu, v);
            }
            0xF5 => {
                let v = self.reg.get_af();
                self.stack_push(mmu, v);
            }

            // JP
            0xC3 => {
                let a = self.imm_word(mmu);
                self.reg.pc = a;
            }
            0xE9 => {
                let a = self.reg.get_hl();
                self.reg.pc = a;
            }
            0xC2 => {
                let a = self.imm_word(mmu);
                if !self.reg.get_flag(Flag::Z) {
                    self.reg.pc = a;
                    cond_cycles = 4;
                }
            }
            0xD2 => {
                let a = self.imm_word(mmu);
                if !self.reg.get_flag(Flag::C) {
                    self.reg.pc = a;
                    cond_cycles = 4;
                }
            }
            0xCA => {
                let a = self.imm_word(mmu);
                if self.reg.get_flag(Flag::Z) {
                    self.reg.pc = a;
                    cond_cycles = 4;
                }
            }
            0xDA => {
                let a = self.imm_word(mmu);
                if self.reg.get_flag(Flag::C) {
                    self.reg.pc = a;
                    cond_cycles = 4;
                }
            }

            // JR
            0x18 => {
                let a = self.imm(mmu) as i8;
                self.reg.pc = self.reg.pc.wrapping_add(a as u16);
            }
            0x20 => {
                let a = self.imm(mmu) as i8;
                if !self.reg.get_flag(Flag::Z) {
                    self.reg.pc = self.reg.pc.wrapping_add(a as u16);
                    cond_cycles = 4;
                }
            }
            0x30 => {
                let a = self.imm(mmu) as i8;
                if !self.reg.get_flag(Flag::C) {
                    self.reg.pc = self.reg.pc.wrapping_add(a as u16);
                    cond_cycles = 4;
                }
            }
            0x28 => {
                let a = self.imm(mmu) as i8;
                if self.reg.get_flag(Flag::Z) {
                    self.reg.pc = self.reg.pc.wrapping_add(a as u16);
                    cond_cycles = 4;
                }
            }
            0x38 => {
                let a = self.imm(mmu) as i8;
                if self.reg.get_flag(Flag::C) {
                    self.reg.pc = self.reg.pc.wrapping_add(a as u16);
                    cond_cycles = 4;
                }
            }

            // CALL
            0xCD => {
                let a = self.imm_word(mmu);
                self.stack_push(mmu, self.reg.pc);
                self.reg.pc = a;
            }
            0xC4 => {
                let a = self.imm_word(mmu);
                if !self.reg.get_flag(Flag::Z) {
                    self.stack_push(mmu, self.reg.pc);
                    self.reg.pc = a;
                    cond_cycles = 12;
                }
            }
            0xCC => {
                let a = self.imm_word(mmu);
                if self.reg.get_flag(Flag::Z) {
                    self.stack_push(mmu, self.reg.pc);
                    self.reg.pc = a;
                    cond_cycles = 12;
                }
            }
            0xD4 => {
                let a = self.imm_word(mmu);
                if !self.reg.get_flag(Flag::C) {
                    self.stack_push(mmu, self.reg.pc);
                    self.reg.pc = a;
                    cond_cycles = 12;
                }
            }
            0xDC => {
                let a = self.imm_word(mmu);
                if self.reg.get_flag(Flag::C) {
                    self.stack_push(mmu, self.reg.pc);
                    self.reg.pc = a;
                    cond_cycles = 12;
                }
            }

            // RET
            0xC9 => {
                let a = self.stack_pop(mmu);
                self.reg.pc = a;
            }
            0xC0 => {
                if !self.reg.get_flag(Flag::Z) {
                    let a = self.stack_pop(mmu);
                    self.reg.pc = a;
                    cond_cycles = 12;
                }
            }
            0xC8 => {
                if self.reg.get_flag(Flag::Z) {
                    let a = self.stack_pop(mmu);
                    self.reg.pc = a;
                    cond_cycles = 12;
                }
            }
            0xD0 => {
                if !self.reg.get_flag(Flag::C) {
                    let a = self.stack_pop(mmu);
                    self.reg.pc = a;
                    cond_cycles = 12;
                }
            }
            0xD8 => {
                if self.reg.get_flag(Flag::C) {
                    let a = self.stack_pop(mmu);
                    self.reg.pc = a;
                    cond_cycles = 12;
                }
            }

            // RETI
            0xD9 => {
                let a = self.stack_pop(mmu);
                self.reg.pc = a;
                self.ime = true;
            }

            // RST
            0xC7 => {
                self.stack_push(mmu, self.reg.pc);
                self.reg.pc = 0x00;
            }
            0xCF => {
                self.stack_push(mmu, self.reg.pc);
                self.reg.pc = 0x08;
            }
            0xD7 => {
                self.stack_push(mmu, self.reg.pc);
                self.reg.pc = 0x10;
            }
            0xDF => {
                self.stack_push(mmu, self.reg.pc);
                self.reg.pc = 0x18;
            }
            0xE7 => {
                self.stack_push(mmu, self.reg.pc);
                self.reg.pc = 0x20;
            }
            0xEF => {
                self.stack_push(mmu, self.reg.pc);
                self.reg.pc = 0x28;
            }
            0xF7 => {
                self.stack_push(mmu, self.reg.pc);
                self.reg.pc = 0x30;
            }
            0xFF => {
                self.stack_push(mmu, self.reg.pc);
                self.reg.pc = 0x38;
            }

            // CB Prefix
            0xCB => {
                opcode = self.imm(mmu);
                using_cb = true;
                match opcode {
                    0x00 => {
                        let v = self.rlc(self.reg.b);
                        self.reg.b = v;
                    }
                    0x01 => {
                        let v = self.rlc(self.reg.c);
                        self.reg.c = v;
                    }
                    0x02 => {
                        let v = self.rlc(self.reg.d);
                        self.reg.d = v;
                    }
                    0x03 => {
                        let v = self.rlc(self.reg.e);
                        self.reg.e = v;
                    }
                    0x04 => {
                        let v = self.rlc(self.reg.h);
                        self.reg.h = v;
                    }
                    0x05 => {
                        let v = self.rlc(self.reg.l);
                        self.reg.l = v;
                    }
                    0x06 => {
                        let hl = mmu.read_byte(self.reg.get_hl());
                        let v = self.rlc(hl);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0x07 => {
                        let v = self.rlc(self.reg.a);
                        self.reg.a = v;
                    }
                    0x08 => {
                        let v = self.rrc(self.reg.b);
                        self.reg.b = v;
                    }
                    0x09 => {
                        let v = self.rrc(self.reg.c);
                        self.reg.c = v;
                    }
                    0x0A => {
                        let v = self.rrc(self.reg.d);
                        self.reg.d = v;
                    }
                    0x0B => {
                        let v = self.rrc(self.reg.e);
                        self.reg.e = v;
                    }
                    0x0C => {
                        let v = self.rrc(self.reg.h);
                        self.reg.h = v;
                    }
                    0x0D => {
                        let v = self.rrc(self.reg.l);
                        self.reg.l = v;
                    }
                    0x0E => {
                        let hl = mmu.read_byte(self.reg.get_hl());
                        let v = self.rrc(hl);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0x0F => {
                        let v = self.rrc(self.reg.a);
                        self.reg.a = v;
                    }
                    0x10 => {
                        let v = self.rl(self.reg.b);
                        self.reg.b = v;
                    }
                    0x11 => {
                        let v = self.rl(self.reg.c);
                        self.reg.c = v;
                    }
                    0x12 => {
                        let v = self.rl(self.reg.d);
                        self.reg.d = v;
                    }
                    0x13 => {
                        let v = self.rl(self.reg.e);
                        self.reg.e = v;
                    }
                    0x14 => {
                        let v = self.rl(self.reg.h);
                        self.reg.h = v;
                    }
                    0x15 => {
                        let v = self.rl(self.reg.l);
                        self.reg.l = v;
                    }
                    0x16 => {
                        let hl = mmu.read_byte(self.reg.get_hl());
                        let v = self.rl(hl);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0x17 => {
                        let v = self.rl(self.reg.a);
                        self.reg.a = v;
                    }
                    0x18 => {
                        let v = self.rr(self.reg.b);
                        self.reg.b = v;
                    }
                    0x19 => {
                        let v = self.rr(self.reg.c);
                        self.reg.c = v;
                    }
                    0x1A => {
                        let v = self.rr(self.reg.d);
                        self.reg.d = v;
                    }
                    0x1B => {
                        let v = self.rr(self.reg.e);
                        self.reg.e = v;
                    }
                    0x1C => {
                        let v = self.rr(self.reg.h);
                        self.reg.h = v;
                    }
                    0x1D => {
                        let v = self.rr(self.reg.l);
                        self.reg.l = v;
                    }
                    0x1E => {
                        let hl = mmu.read_byte(self.reg.get_hl());
                        let v = self.rr(hl);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0x1F => {
                        let v = self.rr(self.reg.a);
                        self.reg.a = v;
                    }
                    0x20 => {
                        let v = self.sla(self.reg.b);
                        self.reg.b = v;
                    }
                    0x21 => {
                        let v = self.sla(self.reg.c);
                        self.reg.c = v;
                    }
                    0x22 => {
                        let v = self.sla(self.reg.d);
                        self.reg.d = v;
                    }
                    0x23 => {
                        let v = self.sla(self.reg.e);
                        self.reg.e = v;
                    }
                    0x24 => {
                        let v = self.sla(self.reg.h);
                        self.reg.h = v;
                    }
                    0x25 => {
                        let v = self.sla(self.reg.l);
                        self.reg.l = v;
                    }
                    0x26 => {
                        let hl = mmu.read_byte(self.reg.get_hl());
                        let v = self.sla(hl);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0x27 => {
                        let v = self.sla(self.reg.a);
                        self.reg.a = v;
                    }
                    0x28 => {
                        let v = self.sra(self.reg.b);
                        self.reg.b = v;
                    }
                    0x29 => {
                        let v = self.sra(self.reg.c);
                        self.reg.c = v;
                    }
                    0x2A => {
                        let v = self.sra(self.reg.d);
                        self.reg.d = v;
                    }
                    0x2B => {
                        let v = self.sra(self.reg.e);
                        self.reg.e = v;
                    }
                    0x2C => {
                        let v = self.sra(self.reg.h);
                        self.reg.h = v;
                    }
                    0x2D => {
                        let v = self.sra(self.reg.l);
                        self.reg.l = v;
                    }
                    0x2E => {
                        let hl = mmu.read_byte(self.reg.get_hl());
                        let v = self.sra(hl);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0x2F => {
                        let v = self.sra(self.reg.a);
                        self.reg.a = v;
                    }
                    0x30 => {
                        let v = self.swap(self.reg.b);
                        self.reg.b = v;
                    }
                    0x31 => {
                        let v = self.swap(self.reg.c);
                        self.reg.c = v;
                    }
                    0x32 => {
                        let v = self.swap(self.reg.d);
                        self.reg.d = v;
                    }
                    0x33 => {
                        let v = self.swap(self.reg.e);
                        self.reg.e = v;
                    }
                    0x34 => {
                        let v = self.swap(self.reg.h);
                        self.reg.h = v;
                    }
                    0x35 => {
                        let v = self.swap(self.reg.l);
                        self.reg.l = v;
                    }
                    0x36 => {
                        let hl = mmu.read_byte(self.reg.get_hl());
                        let v = self.swap(hl);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0x37 => {
                        let v = self.swap(self.reg.a);
                        self.reg.a = v;
                    }
                    0x38 => {
                        let v = self.srl(self.reg.b);
                        self.reg.b = v;
                    }
                    0x39 => {
                        let v = self.srl(self.reg.c);
                        self.reg.c = v;
                    }
                    0x3A => {
                        let v = self.srl(self.reg.d);
                        self.reg.d = v;
                    }
                    0x3B => {
                        let v = self.srl(self.reg.e);
                        self.reg.e = v;
                    }
                    0x3C => {
                        let v = self.srl(self.reg.h);
                        self.reg.h = v;
                    }
                    0x3D => {
                        let v = self.srl(self.reg.l);
                        self.reg.l = v;
                    }
                    0x3E => {
                        let hl = mmu.read_byte(self.reg.get_hl());
                        let v = self.srl(hl);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0x3F => {
                        let v = self.srl(self.reg.a);
                        self.reg.a = v;
                    }
                    0x40 => self.bit(self.reg.b, 0),
                    0x41 => self.bit(self.reg.c, 0),
                    0x42 => self.bit(self.reg.d, 0),
                    0x43 => self.bit(self.reg.e, 0),
                    0x44 => self.bit(self.reg.h, 0),
                    0x45 => self.bit(self.reg.l, 0),
                    0x46 => self.bit(mmu.read_byte(self.reg.get_hl()), 0),
                    0x47 => self.bit(self.reg.a, 0),
                    0x48 => self.bit(self.reg.b, 1),
                    0x49 => self.bit(self.reg.c, 1),
                    0x4A => self.bit(self.reg.d, 1),
                    0x4B => self.bit(self.reg.e, 1),
                    0x4C => self.bit(self.reg.h, 1),
                    0x4D => self.bit(self.reg.l, 1),
                    0x4E => self.bit(mmu.read_byte(self.reg.get_hl()), 1),
                    0x4F => self.bit(self.reg.a, 1),
                    0x50 => self.bit(self.reg.b, 2),
                    0x51 => self.bit(self.reg.c, 2),
                    0x52 => self.bit(self.reg.d, 2),
                    0x53 => self.bit(self.reg.e, 2),
                    0x54 => self.bit(self.reg.h, 2),
                    0x55 => self.bit(self.reg.l, 2),
                    0x56 => self.bit(mmu.read_byte(self.reg.get_hl()), 2),
                    0x57 => self.bit(self.reg.a, 2),
                    0x58 => self.bit(self.reg.b, 3),
                    0x59 => self.bit(self.reg.c, 3),
                    0x5A => self.bit(self.reg.d, 3),
                    0x5B => self.bit(self.reg.e, 3),
                    0x5C => self.bit(self.reg.h, 3),
                    0x5D => self.bit(self.reg.l, 3),
                    0x5E => self.bit(mmu.read_byte(self.reg.get_hl()), 3),
                    0x5F => self.bit(self.reg.a, 3),
                    0x60 => self.bit(self.reg.b, 4),
                    0x61 => self.bit(self.reg.c, 4),
                    0x62 => self.bit(self.reg.d, 4),
                    0x63 => self.bit(self.reg.e, 4),
                    0x64 => self.bit(self.reg.h, 4),
                    0x65 => self.bit(self.reg.l, 4),
                    0x66 => self.bit(mmu.read_byte(self.reg.get_hl()), 4),
                    0x67 => self.bit(self.reg.a, 4),
                    0x68 => self.bit(self.reg.b, 5),
                    0x69 => self.bit(self.reg.c, 5),
                    0x6A => self.bit(self.reg.d, 5),
                    0x6B => self.bit(self.reg.e, 5),
                    0x6C => self.bit(self.reg.h, 5),
                    0x6D => self.bit(self.reg.l, 5),
                    0x6E => self.bit(mmu.read_byte(self.reg.get_hl()), 5),
                    0x6F => self.bit(self.reg.a, 5),
                    0x70 => self.bit(self.reg.b, 6),
                    0x71 => self.bit(self.reg.c, 6),
                    0x72 => self.bit(self.reg.d, 6),
                    0x73 => self.bit(self.reg.e, 6),
                    0x74 => self.bit(self.reg.h, 6),
                    0x75 => self.bit(self.reg.l, 6),
                    0x76 => self.bit(mmu.read_byte(self.reg.get_hl()), 6),
                    0x77 => self.bit(self.reg.a, 6),
                    0x78 => self.bit(self.reg.b, 7),
                    0x79 => self.bit(self.reg.c, 7),
                    0x7A => self.bit(self.reg.d, 7),
                    0x7B => self.bit(self.reg.e, 7),
                    0x7C => self.bit(self.reg.h, 7),
                    0x7D => self.bit(self.reg.l, 7),
                    0x7E => self.bit(mmu.read_byte(self.reg.get_hl()), 7),
                    0x7F => self.bit(self.reg.a, 7),
                    0x80 => self.reg.b = self.res(self.reg.b, 0),
                    0x81 => self.reg.c = self.res(self.reg.c, 0),
                    0x82 => self.reg.d = self.res(self.reg.d, 0),
                    0x83 => self.reg.e = self.res(self.reg.e, 0),
                    0x84 => self.reg.h = self.res(self.reg.h, 0),
                    0x85 => self.reg.l = self.res(self.reg.l, 0),
                    0x86 => {
                        let v = self.res(mmu.read_byte(self.reg.get_hl()), 0);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0x87 => self.reg.a = self.res(self.reg.a, 0),
                    0x88 => self.reg.b = self.res(self.reg.b, 1),
                    0x89 => self.reg.c = self.res(self.reg.c, 1),
                    0x8A => self.reg.d = self.res(self.reg.d, 1),
                    0x8B => self.reg.e = self.res(self.reg.e, 1),
                    0x8C => self.reg.h = self.res(self.reg.h, 1),
                    0x8D => self.reg.l = self.res(self.reg.l, 1),
                    0x8E => {
                        let v = self.res(mmu.read_byte(self.reg.get_hl()), 1);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0x8F => self.reg.a = self.res(self.reg.a, 1),
                    0x90 => self.reg.b = self.res(self.reg.b, 2),
                    0x91 => self.reg.c = self.res(self.reg.c, 2),
                    0x92 => self.reg.d = self.res(self.reg.d, 2),
                    0x93 => self.reg.e = self.res(self.reg.e, 2),
                    0x94 => self.reg.h = self.res(self.reg.h, 2),
                    0x95 => self.reg.l = self.res(self.reg.l, 2),
                    0x96 => {
                        let v = self.res(mmu.read_byte(self.reg.get_hl()), 2);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0x97 => self.reg.a = self.res(self.reg.a, 2),
                    0x98 => self.reg.b = self.res(self.reg.b, 3),
                    0x99 => self.reg.c = self.res(self.reg.c, 3),
                    0x9A => self.reg.d = self.res(self.reg.d, 3),
                    0x9B => self.reg.e = self.res(self.reg.e, 3),
                    0x9C => self.reg.h = self.res(self.reg.h, 3),
                    0x9D => self.reg.l = self.res(self.reg.l, 3),
                    0x9E => {
                        let v = self.res(mmu.read_byte(self.reg.get_hl()), 3);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0x9F => self.reg.a = self.res(self.reg.a, 3),
                    0xA0 => self.reg.b = self.res(self.reg.b, 4),
                    0xA1 => self.reg.c = self.res(self.reg.c, 4),
                    0xA2 => self.reg.d = self.res(self.reg.d, 4),
                    0xA3 => self.reg.e = self.res(self.reg.e, 4),
                    0xA4 => self.reg.h = self.res(self.reg.h, 4),
                    0xA5 => self.reg.l = self.res(self.reg.l, 4),
                    0xA6 => {
                        let v = self.res(mmu.read_byte(self.reg.get_hl()), 4);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0xA7 => self.reg.a = self.res(self.reg.a, 4),
                    0xA8 => self.reg.b = self.res(self.reg.b, 5),
                    0xA9 => self.reg.c = self.res(self.reg.c, 5),
                    0xAA => self.reg.d = self.res(self.reg.d, 5),
                    0xAB => self.reg.e = self.res(self.reg.e, 5),
                    0xAC => self.reg.h = self.res(self.reg.h, 5),
                    0xAD => self.reg.l = self.res(self.reg.l, 5),
                    0xAE => {
                        let v = self.res(mmu.read_byte(self.reg.get_hl()), 5);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0xAF => self.reg.a = self.res(self.reg.a, 5),
                    0xB0 => self.reg.b = self.res(self.reg.b, 6),
                    0xB1 => self.reg.c = self.res(self.reg.c, 6),
                    0xB2 => self.reg.d = self.res(self.reg.d, 6),
                    0xB3 => self.reg.e = self.res(self.reg.e, 6),
                    0xB4 => self.reg.h = self.res(self.reg.h, 6),
                    0xB5 => self.reg.l = self.res(self.reg.l, 6),
                    0xB6 => {
                        let v = self.res(mmu.read_byte(self.reg.get_hl()), 6);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0xB7 => self.reg.a = self.res(self.reg.a, 6),
                    0xB8 => self.reg.b = self.res(self.reg.b, 7),
                    0xB9 => self.reg.c = self.res(self.reg.c, 7),
                    0xBA => self.reg.d = self.res(self.reg.d, 7),
                    0xBB => self.reg.e = self.res(self.reg.e, 7),
                    0xBC => self.reg.h = self.res(self.reg.h, 7),
                    0xBD => self.reg.l = self.res(self.reg.l, 7),
                    0xBE => {
                        let v = self.res(mmu.read_byte(self.reg.get_hl()), 7);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0xBF => self.reg.a = self.res(self.reg.a, 7),
                    0xC0 => self.reg.b = self.set(self.reg.b, 0),
                    0xC1 => self.reg.c = self.set(self.reg.c, 0),
                    0xC2 => self.reg.d = self.set(self.reg.d, 0),
                    0xC3 => self.reg.e = self.set(self.reg.e, 0),
                    0xC4 => self.reg.h = self.set(self.reg.h, 0),
                    0xC5 => self.reg.l = self.set(self.reg.l, 0),
                    0xC6 => {
                        let v = self.set(mmu.read_byte(self.reg.get_hl()), 0);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0xC7 => self.reg.a = self.set(self.reg.a, 0),
                    0xC8 => self.reg.b = self.set(self.reg.b, 1),
                    0xC9 => self.reg.c = self.set(self.reg.c, 1),
                    0xCA => self.reg.d = self.set(self.reg.d, 1),
                    0xCB => self.reg.e = self.set(self.reg.e, 1),
                    0xCC => self.reg.h = self.set(self.reg.h, 1),
                    0xCD => self.reg.l = self.set(self.reg.l, 1),
                    0xCE => {
                        let v = self.set(mmu.read_byte(self.reg.get_hl()), 1);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0xCF => self.reg.a = self.set(self.reg.a, 1),
                    0xD0 => self.reg.b = self.set(self.reg.b, 2),
                    0xD1 => self.reg.c = self.set(self.reg.c, 2),
                    0xD2 => self.reg.d = self.set(self.reg.d, 2),
                    0xD3 => self.reg.e = self.set(self.reg.e, 2),
                    0xD4 => self.reg.h = self.set(self.reg.h, 2),
                    0xD5 => self.reg.l = self.set(self.reg.l, 2),
                    0xD6 => {
                        let v = self.set(mmu.read_byte(self.reg.get_hl()), 2);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0xD7 => self.reg.a = self.set(self.reg.a, 2),
                    0xD8 => self.reg.b = self.set(self.reg.b, 3),
                    0xD9 => self.reg.c = self.set(self.reg.c, 3),
                    0xDA => self.reg.d = self.set(self.reg.d, 3),
                    0xDB => self.reg.e = self.set(self.reg.e, 3),
                    0xDC => self.reg.h = self.set(self.reg.h, 3),
                    0xDD => self.reg.l = self.set(self.reg.l, 3),
                    0xDE => {
                        let v = self.set(mmu.read_byte(self.reg.get_hl()), 3);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0xDF => self.reg.a = self.set(self.reg.a, 3),
                    0xE0 => self.reg.b = self.set(self.reg.b, 4),
                    0xE1 => self.reg.c = self.set(self.reg.c, 4),
                    0xE2 => self.reg.d = self.set(self.reg.d, 4),
                    0xE3 => self.reg.e = self.set(self.reg.e, 4),
                    0xE4 => self.reg.h = self.set(self.reg.h, 4),
                    0xE5 => self.reg.l = self.set(self.reg.l, 4),
                    0xE6 => {
                        let v = self.set(mmu.read_byte(self.reg.get_hl()), 4);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0xE7 => self.reg.a = self.set(self.reg.a, 4),
                    0xE8 => self.reg.b = self.set(self.reg.b, 5),
                    0xE9 => self.reg.c = self.set(self.reg.c, 5),
                    0xEA => self.reg.d = self.set(self.reg.d, 5),
                    0xEB => self.reg.e = self.set(self.reg.e, 5),
                    0xEC => self.reg.h = self.set(self.reg.h, 5),
                    0xED => self.reg.l = self.set(self.reg.l, 5),
                    0xEE => {
                        let v = self.set(mmu.read_byte(self.reg.get_hl()), 5);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0xEF => self.reg.a = self.set(self.reg.a, 5),
                    0xF0 => self.reg.b = self.set(self.reg.b, 6),
                    0xF1 => self.reg.c = self.set(self.reg.c, 6),
                    0xF2 => self.reg.d = self.set(self.reg.d, 6),
                    0xF3 => self.reg.e = self.set(self.reg.e, 6),
                    0xF4 => self.reg.h = self.set(self.reg.h, 6),
                    0xF5 => self.reg.l = self.set(self.reg.l, 6),
                    0xF6 => {
                        let v = self.set(mmu.read_byte(self.reg.get_hl()), 6);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0xF7 => self.reg.a = self.set(self.reg.a, 6),
                    0xF8 => self.reg.b = self.set(self.reg.b, 7),
                    0xF9 => self.reg.c = self.set(self.reg.c, 7),
                    0xFA => self.reg.d = self.set(self.reg.d, 7),
                    0xFB => self.reg.e = self.set(self.reg.e, 7),
                    0xFC => self.reg.h = self.set(self.reg.h, 7),
                    0xFD => self.reg.l = self.set(self.reg.l, 7),
                    0xFE => {
                        let v = self.set(mmu.read_byte(self.reg.get_hl()), 7);
                        mmu.write_byte(self.reg.get_hl(), v);
                    }
                    0xFF => self.reg.a = self.set(self.reg.a, 7),
                }
            }
            _ => panic!("Unsupported or unimplemented opcode 0x{:X}", opcode),
        };
        if using_cb {
            OPCODE_CB_TABLE[opcode as usize]
        } else {
            OPCODE_TABLE[opcode as usize] + cond_cycles
        }
    }

    /// Reads and returns the value at the current PC location
    /// Increments the PC after reading
    fn imm(&mut self, mmu: &mut mmu::Mmu) -> u8 {
        let v = mmu.read_byte(self.reg.pc);
        self.reg.pc += 1;
        v
    }

    /// Reads and returns the word at the current PC location
    /// Value is little endian representation
    /// Increments PC to after the word
    fn imm_word(&mut self, mmu: &mut mmu::Mmu) -> u16 {
        let lo = self.imm(mmu);
        let hi = self.imm(mmu);
        (u16::from(hi) << 8) | u16::from(lo)
    }

    fn stack_push(&mut self, mmu: &mut mmu::Mmu, v: u16) {
        self.reg.sp -= 2;
        mmu.write_word(self.reg.sp, v);
    }

    fn stack_pop(&mut self, mmu: &mut mmu::Mmu) -> u16 {
        let v = mmu.read_word(self.reg.sp);
        self.reg.sp += 2;
        v
    }

    /// Adds the given register value `r` to the `A` register.
    /// Flags:
    ///
    /// - Z: Set to 1 if resulting value is 0, set to 0 otherwise
    /// - N: Set to 0
    /// - H: Set to 1 if bit 3 has a carry, 0 otherwise
    /// - C: Set to 1 if bit 7 has a carry, 0 otherwise
    fn add(&mut self, r: u8) {
        let v = self.reg.a.wrapping_add(r);
        // Evaluate flags
        self.reg.set_flag(Flag::Z, v == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg
            .set_flag(Flag::H, (self.reg.a & 0x0F) + (r & 0x0F) > 0x0F);
        self.reg
            .set_flag(Flag::C, u16::from(self.reg.a) + u16::from(r) > 0xFF);
        self.reg.a = v;
    }

    /// Adds the given register value `r` and carry flag to the `A` register.
    /// Flags:
    ///
    /// - Z: Set to 1 if resulting value is 0, set to 0 otherwise
    /// - N: Set to 0
    /// - H: Set to 1 if bit 3 has a carry, 0 otherwise
    /// - C: Set to 1 if bit 7 has a carry, 0 otherwise
    fn adc(&mut self, r: u8) {
        let c = u8::from(self.reg.get_flag(Flag::C));
        let v = self.reg.a.wrapping_add(r).wrapping_add(c);
        // Evaluate flags
        self.reg.set_flag(Flag::Z, v == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(
            Flag::H,
            (self.reg.a & 0x0F) + (r & 0x0F) + (c & 0x0F) > 0x0F,
        );
        self.reg.set_flag(
            Flag::C,
            u16::from(self.reg.a) + u16::from(r) + u16::from(c) > 0xFF,
        );
        self.reg.a = v;
    }

    /// Adds an immediate value as a signed 8-bit integer to the
    /// Stack Pointer (SP).
    /// Flags:
    ///
    /// - Z: Set to 0
    /// - N: Set to 0
    /// - H: Set to 1 if bit 3 carries, 0 otherwise
    /// - C: Set to 1 if bit 7 carries, 0 otherwise
    fn add_sp(&mut self, mmu: &mut mmu::Mmu) {
        let v = (i16::from(self.imm(mmu) as i8)) as u16;
        self.reg.set_flag(Flag::Z, false);
        self.reg.set_flag(Flag::N, false);
        self.reg
            .set_flag(Flag::H, (self.reg.sp & 0x000F) + (v & 0x000F) > 0x000F);
        self.reg
            .set_flag(Flag::C, (self.reg.sp & 0x00FF) + (v & 0x00FF) > 0x00FF);
        self.reg.sp = self.reg.sp.wrapping_add(v);
    }

    /// Adds a given 16-bit register value to the HL register.
    /// Flags:
    ///
    /// - Z: Set to 0
    /// - N: Set to 0
    /// - H: Set to 1 if bit 3 carries, 0 otherwise
    /// - C: Set to 1 if bit 7 carries, 0 otherwise
    fn add_hl(&mut self, r: u16) {
        let hl = self.reg.get_hl();
        self.reg.set_flag(Flag::N, false);
        self.reg
            .set_flag(Flag::H, (r & 0x000F) + (hl & 0x000F) > 0x000F);
        self.reg
            .set_flag(Flag::C, (r & 0x00FF) + (hl & 0x00FF) > 0x00FF);
        self.reg.set_hl(hl.wrapping_add(r));
    }

    /// Subtracts the given register value `r` from the `A` register.
    /// Flags:
    ///
    /// - Z: Set to 1 if resulting value is 0, set to 0 otherwise
    /// - N: Set to 1
    /// - H: Set to 1 if bit 3 doesn't borrow, 0 otherwise
    /// - C: Set to 1 if bit 7 doesn't borrow, 0 otherwise
    fn sub(&mut self, r: u8) {
        let v = self.reg.a.wrapping_sub(r);
        // Evaluate flags
        self.reg.set_flag(Flag::Z, v == 0);
        self.reg.set_flag(Flag::N, true);
        self.reg.set_flag(Flag::H, (self.reg.a & 0x0F) < (r & 0x0F));
        self.reg
            .set_flag(Flag::C, u16::from(self.reg.a) < u16::from(r));
        self.reg.a = v;
    }

    /// Subtracts the given register value `r` plus the carry
    /// from the `A` register.
    /// Flags:
    ///
    /// - Z: Set to 1 if resulting value is 0, set to 0 otherwise
    /// - N: Set to 1
    /// - H: Set to 1 if bit 3 doesn't borrow, 0 otherwise
    /// - C: Set to 1 if bit 7 doesn't borrow, 0 otherwise
    fn sbc(&mut self, r: u8) {
        let c = u8::from(self.reg.get_flag(Flag::C));
        let v = self.reg.a.wrapping_sub(r).wrapping_sub(c);
        // Evaluate flags
        self.reg.set_flag(Flag::Z, v == 0);
        self.reg.set_flag(Flag::N, true);
        self.reg
            .set_flag(Flag::H, (self.reg.a & 0x0F) < (r & 0x0F) + (c & 0x0F));
        self.reg
            .set_flag(Flag::C, u16::from(self.reg.a) < u16::from(r) + u16::from(c));
        self.reg.a = v;
    }

    /// Performs a bitwise AND operation between `A` and the given register `r`
    /// Flags:
    ///
    /// - Z: Set to 1 if resulting value is 0, set to 0 otherwise
    /// - N: Set to 0
    /// - H: Set to 1
    /// - C: Set to 0
    fn and(&mut self, r: u8) {
        let v = self.reg.a & r;
        self.reg.set_flag(Flag::Z, v == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, true);
        self.reg.set_flag(Flag::C, false);
        self.reg.a = v;
    }

    /// Performs a bitwise XOR operation between `A` and the given register `r`
    /// Flags:
    ///
    /// - Z: Set to 1 if resulting value is 0, set to 0 otherwise
    /// - N: Set to 0
    /// - H: Set to 0
    /// - C: Set to 0
    fn xor(&mut self, r: u8) {
        let v = self.reg.a ^ r;
        self.reg.set_flag(Flag::Z, v == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, false);
        self.reg.set_flag(Flag::C, false);
        self.reg.a = v;
    }

    /// Performs a bitwise OR operation between `A` and the given register `r`
    /// Flags:
    ///
    /// - Z: Set to 1 if resulting value is 0, set to 0 otherwise
    /// - N: Set to 0
    /// - H: Set to 0
    /// - C: Set to 0
    fn or(&mut self, r: u8) {
        let v = self.reg.a | r;
        self.reg.set_flag(Flag::Z, v == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, false);
        self.reg.set_flag(Flag::C, false);
        self.reg.a = v;
    }

    /// Performs a compare operation between `A` and the given register `r`
    /// Sets the flags similar to a SUB operation, but not writing the result
    /// Flags:
    ///
    /// - Z: Set to 1 if resulting value is 0, set to 0 otherwise
    /// - N: Set to 1
    /// - H: Set to 1 if bit 3 doesn't borrow, 0 otherwise
    /// - C: Set to 1 if bit 7 doesn't borrow, 0 otherwise
    fn cp(&mut self, r: u8) {
        // Save current value of `A` to revert after SUB
        let a = self.reg.a;
        self.sub(r);
        self.reg.a = a;
    }

    /// Increment the given value `r` and returns the incremented value.
    /// Flags:
    ///
    /// - Z: Set to 1 if resulting value is 0, set to 0 otherwise
    /// - N: Set to 0
    /// - H: Set to 1 if bit 3 carries, 0 otherwise
    /// - C: None
    fn inc(&mut self, r: u8) -> u8 {
        let v = r.wrapping_add(1);
        self.reg.set_flag(Flag::Z, v == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, (r & 0x0F) + 0x1 > 0x0F);
        v
    }

    /// Decrement the given value `r` and returns the incremented value.
    /// Flags:
    ///
    /// - Z: Set to 1 if resulting value is 0, set to 0 otherwise
    /// - N: Set to 1
    /// - H: Set to 1 if bit 3 doesn't borrow, 0 otherwise
    /// - C: None
    fn dec(&mut self, r: u8) -> u8 {
        let v = r.wrapping_sub(1);
        self.reg.set_flag(Flag::Z, v == 0);
        self.reg.set_flag(Flag::N, true);
        self.reg.set_flag(Flag::H, r.trailing_zeros() >= 4);
        v
    }

    /// Rotate the given register value left, with bit 7 wrapping to bit 0
    /// Flags:
    ///
    /// - Z: Set to 1 if resulting value is 0, set to 0 otherwise
    /// - N: Set to 0
    /// - H: Set to 0
    /// - C: Set to value of `r` bit 7, before the shift
    fn rlc(&mut self, r: u8) -> u8 {
        let v = r.rotate_left(1);
        self.reg.set_flag(Flag::Z, v == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, false);
        self.reg.set_flag(Flag::C, (r >> 7) == 0x1);
        v
    }

    /// Rotate the given register value right, with bit 0 wrapping to bit 7
    /// Flags:
    ///
    /// - Z: Set to 1 if resulting value is 0, set to 0 otherwise
    /// - N: Set to 0
    /// - H: Set to 0
    /// - C: Set to value of `r` bit 0, before the shift
    fn rrc(&mut self, r: u8) -> u8 {
        let v = r.rotate_right(1);
        self.reg.set_flag(Flag::Z, v == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, false);
        self.reg.set_flag(Flag::C, (r & 0x01) == 0x1);
        v
    }

    /// Rotate the given register value left, with bit 7 set to C,
    /// and bit 0 containing the value of the old C.
    /// Flags:
    ///
    /// - Z: Set to 1 if resulting value is 0, set to 0 otherwise
    /// - N: Set to 0
    /// - H: Set to 0
    /// - C: Set to value of `r` bit 7, before the shift
    fn rl(&mut self, r: u8) -> u8 {
        let mut v = r << 1;
        v |= self.reg.get_flag(Flag::C) as u8;
        self.reg.set_flag(Flag::Z, v == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, false);
        self.reg.set_flag(Flag::C, (r >> 7) == 0x1);
        v
    }

    /// Rotate the given register value right, with bit 0 set to C,
    /// and bit 7 containing the value of the old C.
    /// Flags:
    ///
    /// - Z: Set to 1 if resulting value is 0, set to 0 otherwise
    /// - N: Set to 0
    /// - H: Set to 0
    /// - C: Set to value of `r` bit 0, before the shift
    fn rr(&mut self, r: u8) -> u8 {
        let mut v = r >> 1;
        v |= (self.reg.get_flag(Flag::C) as u8) << 7;
        self.reg.set_flag(Flag::Z, v == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, false);
        self.reg.set_flag(Flag::C, (r & 0x01) == 0x1);
        v
    }

    /// Shift register `r` left into the Carry flag. Bit 0 set to 0.
    /// Flags:
    ///
    /// - Z: Set to 1 if resulting value is 0, set to 0 otherwise
    /// - N: Set to 0
    /// - H: Set to 0
    /// - C: Set to value of `r` bit 7, before the shift
    fn sla(&mut self, r: u8) -> u8 {
        let v = r << 1;
        self.reg.set_flag(Flag::Z, v == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, false);
        self.reg.set_flag(Flag::C, (r >> 7) == 0x1);
        v
    }

    /// Shift register `r` right into the Carry flag. Bit 7 unchanged.
    /// Flags:
    ///
    /// - Z: Set to 1 if resulting value is 0, set to 0 otherwise
    /// - N: Set to 0
    /// - H: Set to 0
    /// - C: Set to value of `r` bit 0, before the shift
    fn sra(&mut self, r: u8) -> u8 {
        let v = r >> 1 | (r & 0x80);
        self.reg.set_flag(Flag::Z, v == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, false);
        self.reg.set_flag(Flag::C, (r & 0x01) == 0x1);
        v
    }

    /// Swap upper and lower 4 bits of `r`
    /// Flags:
    ///
    /// - Z: Set to 1 if resulting value is 0, set to 0 otherwise
    /// - N: Set to 0
    /// - H: Set to 0
    /// - C: Set to 0
    fn swap(&mut self, r: u8) -> u8 {
        self.reg.set_flag(Flag::Z, r == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, false);
        self.reg.set_flag(Flag::C, false);
        (r >> 4) | (r << 4)
    }

    /// Shift register `r` right into the Carry flag. Bit 7 set to 0.
    /// Flags:
    ///
    /// - Z: Set to 1 if resulting value is 0, set to 0 otherwise
    /// - N: Set to 0
    /// - H: Set to 0
    /// - C: Set to value of `r` bit 0, before the shift
    fn srl(&mut self, r: u8) -> u8 {
        let v = r >> 1;
        self.reg.set_flag(Flag::Z, v == 0);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, false);
        self.reg.set_flag(Flag::C, (r & 0x01) == 0x1);
        v
    }

    /// Test bit `b` in register `r`
    /// Flags:
    ///
    /// - Z: Set if bit `b` of register `r` is 0
    /// - N: Set to 0
    /// - H: Set to 1
    /// - C: None
    fn bit(&mut self, r: u8, b: u8) {
        let v = r & (0x1 << b) == 0x0;
        self.reg.set_flag(Flag::Z, v);
        self.reg.set_flag(Flag::N, false);
        self.reg.set_flag(Flag::H, true);
    }

    /// Reset bit `b` in register `r`
    /// Flags:
    ///
    /// - Z: None
    /// - N: None
    /// - H: None
    /// - C: None
    fn res(&mut self, r: u8, b: u8) -> u8 {
        r & !(0x1 << b)
    }

    /// Set bit `b` in register `r`
    /// Flags:
    ///
    /// - Z: None
    /// - N: None
    /// - H: None
    /// - C: None
    fn set(&mut self, r: u8, b: u8) -> u8 {
        r | (0x1 << b)
    }
}

#[cfg(test)]
mod cpu_tests {
    use super::*;
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

    #[test]
    fn rl_test() {
        let mut cpu = Cpu::power_on();
        let mut v = cpu.rl(0b0110_0101);
        assert_eq!(v, 0b1100_1011);
        assert_eq!(cpu.reg.get_flag(Flag::C), false);
        v = cpu.rl(0b1100_1011);
        assert_eq!(v, 0b1001_0110);
        assert_eq!(cpu.reg.get_flag(Flag::C), true);
        v = cpu.rl(0b1001_0110);
        assert_eq!(v, 0b0010_1101);
        assert_eq!(cpu.reg.get_flag(Flag::C), true);
    }

    #[test]
    fn rr_test() {
        let mut cpu = Cpu::power_on();
        let mut v = cpu.rr(0b0110_0101);
        assert_eq!(v, 0b1011_0010);
        assert_eq!(cpu.reg.get_flag(Flag::C), true);
        v = cpu.rr(0b1011_0010);
        assert_eq!(v, 0b1101_1001);
        assert_eq!(cpu.reg.get_flag(Flag::C), false);
        v = cpu.rr(0b1101_1001);
        assert_eq!(v, 0b0110_1100);
        assert_eq!(cpu.reg.get_flag(Flag::C), true);
    }
}
