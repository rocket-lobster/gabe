use std::{io::Write, ops::Range};

use crate::core::{disassemble, gb::Gameboy};

pub struct Debugger {
    enabled: bool,
    breakpoints: Vec<u16>,
    current_command: DebugCommand,
}

pub enum DebuggerState {
    /// Continue execution until next breakpoint
    Running,
    /// Commanded to stop all debugger operations
    Stopping,
}

enum DebugCommand {
    BreakpointAdd(u16),
    BreakpointDelete(u16),
    BreakpointList,
    Step(usize),
    Continue,
    Print(String),
    Dump(Range<u16>),
    Disassemble(usize),
    Help,
    Quit,
    Nothing,
    Error(String),
}

impl Debugger {
    pub fn new(enabled: bool) -> Self {
        Debugger {
            enabled,
            breakpoints: vec![],
            current_command: DebugCommand::Nothing,
        }
    }

    pub fn is_running(&self) -> bool {
        self.enabled
    }

    /// Start the debugger
    pub fn start(&mut self) {
        self.enabled = true;
        self.current_command = DebugCommand::Nothing;
    }

    /// Stops the debugger until commanded back on
    pub fn quit(&mut self) {
        self.enabled = false;
        self.breakpoints.clear();
    }

    pub fn update(&mut self, state: &Gameboy) -> DebuggerState {
        let mut ret = DebuggerState::Running;
        let pc = state.get_pc();
        // Check if we're at a breakpoint
        if self.breakpoints.contains(&pc) {
            // Stop execution and get next command
            println!("Stopping execution at breakpoint: {:04X}", &pc);
            self.current_command = self.get_command();
        }

        loop {
            match &self.current_command {
                DebugCommand::Disassemble(n) => {
                    let mem = state.get_memory_range(pc..pc + (*n as u16));
                    let disasm = disassemble::disassemble_block(mem, pc);
                    for (p, s) in disasm {
                        println!("0x{:04X}: {}", p, s);
                    }
                }
                DebugCommand::Step(n) => {
                    if *n == 0 {
                        println!("Completed steps.");
                        let debug_data = state.get_debug_state();
                        println!("{}", debug_data.cpu_data);
                        println!("Total T-cycles: {}", debug_data.total_cycles);
                        println!("IE: {:02X}  IF: {:02X}", debug_data.ie_data, debug_data.if_data);
                        // Grab max number of bytes needed for any instruction
                        let mem = state.get_memory_range(pc..pc + 3);
                        let disasm = disassemble::disassemble_block(mem, pc);
                        println!("0x{:04X}: {}", disasm[0].0, disasm[0].1);
                        println!("LCDC: {:02X}  STAT: {:02X}  LY: {:02X}", debug_data.vram_lcdc, debug_data.vram_stat, debug_data.vram_ly);
                    } else {
                        self.current_command = DebugCommand::Step(n - 1);
                        break;
                    }
                }
                DebugCommand::Continue => break,
                DebugCommand::Print(s) => {
                    let target = s.clone();
                    let mut iter = target.split_terminator('.');
                    if let Some(c1) = iter.next() {
                        match c1 {
                            "cpu" => {
                                let cpu_state = state.get_debug_state().cpu_data;
                                println!("{}", cpu_state);
                            }
                            _ => println!("Cannot resolve target \"{}\"", c1),
                        }
                    }
                }
                DebugCommand::Dump(r) => {
                    let new_start: u16 = if r.start % 16 != 0 {
                        // We start in the middle of the line, find nearest line start
                        let pad = r.start % 16;
                        r.start - pad
                    } else {
                        r.start
                    };

                    let new_end: u16 = if r.end % 16 != 0 {
                        // We end in the middle of the line, fill remaining line
                        let pad = 16 - (r.end % 16);
                        r.end + pad
                    } else {
                        r.end
                    };
                    let vals = state.get_memory_range(std::ops::Range {
                        start: new_start,
                        end: new_end,
                    });
                    let mut current_line = new_start;
                    for v in vals.chunks(16) {
                        print!("0x{:04X}: ", current_line);
                        for x in v.iter() {
                            print!("{:02X} ", x);
                        }
                        println!();
                        current_line += 16;
                    }
                }
                DebugCommand::Quit => {
                    ret = DebuggerState::Stopping;
                    break;
                }
                DebugCommand::Nothing => (),
                _ => panic!("Incorrect command fetch, exiting."),
            };
            self.current_command = self.get_command();
        }
        ret
    }

    fn get_command(&mut self) -> DebugCommand {
        let ret: DebugCommand;
        loop {
            print!("gabe> ");
            std::io::stdout().flush().unwrap();

            // Wait for stdin input
            let mut in_buffer = String::new();
            let stdin = std::io::stdin();
            stdin
                .read_line(&mut in_buffer)
                .expect("Failed to read input from stdin.");
            match Self::parse_input(in_buffer) {
                DebugCommand::BreakpointAdd(b) => self.breakpoints.push(b),
                DebugCommand::BreakpointDelete(b) => self.breakpoints.retain(|&x| x != b),
                DebugCommand::BreakpointList => {
                    if self.breakpoints.is_empty() {
                        println!("No breakpoints currently set.")
                    } else {
                        println!("{:#?}", self.breakpoints)
                    }
                }
                DebugCommand::Help => Self::print_help(),
                DebugCommand::Error(s) => println!(
                    "{}\n Use command 'help' to see available commands and their arguments.",
                    s
                ),
                DebugCommand::Nothing => (),
                DebugCommand::Disassemble(s) => {
                    ret = DebugCommand::Disassemble(s);
                    break;
                }
                // Remaining commands pass back to caller
                DebugCommand::Step(s) => {
                    ret = DebugCommand::Step(s);
                    break;
                }
                DebugCommand::Continue => {
                    ret = DebugCommand::Continue;
                    break;
                }
                DebugCommand::Print(s) => {
                    ret = DebugCommand::Print(s);
                    break;
                }
                DebugCommand::Dump(r) => {
                    ret = DebugCommand::Dump(r);
                    break;
                }
                DebugCommand::Quit => {
                    ret = DebugCommand::Quit;
                    break;
                }
            }
        }
        ret
    }

    fn parse_input(mut input: String) -> DebugCommand {
        // Only accept ASCII and non-empty commands
        if input.is_empty() || !input.is_ascii() {
            DebugCommand::Error("Only accepts non-empty and ASCII input.".to_string())
        } else {
            // Standarize into lowercase for comparisons
            input.make_ascii_lowercase();
            let mut iter = input.split_ascii_whitespace();
            if let Some(c) = iter.next() {
                match c {
                    "break" => {
                        // Grab two values from string
                        let c2 = iter.next();
                        let c3 = iter.next();

                        match c2 {
                            Some("list") => DebugCommand::BreakpointList,
                            Some("add") => {
                                if let Some(c3) = c3 {
                                    if let Ok(addr) = u16::from_str_radix(c3, 16) {
                                        DebugCommand::BreakpointAdd(addr)
                                    } else {
                                        DebugCommand::Error("Unable to parse address.".to_string())
                                    }
                                } else {
                                    DebugCommand::Error("No address provided.".to_string())
                                }
                            }
                            Some("delete") => {
                                if let Some(c3) = c3 {
                                    if let Ok(addr) = u16::from_str_radix(c3, 16) {
                                        DebugCommand::BreakpointDelete(addr)
                                    } else {
                                        DebugCommand::Error("Unable to parse address.".to_string())
                                    }
                                } else {
                                    DebugCommand::Error("No address provided.".to_string())
                                }
                            }
                            _ => {
                                DebugCommand::Error("Unrecognized breakpoint command.".to_string())
                            }
                        }
                    }
                    "disassemble" | "disasm" => {
                        if let Some(c2) = iter.next() {
                            // Provided number of steps
                            if let Ok(num) = c2.parse::<usize>() {
                                DebugCommand::Disassemble(num)
                            } else {
                                DebugCommand::Error(
                                    "Unable to parse disassemble count.".to_string(),
                                )
                            }
                        } else {
                            DebugCommand::Disassemble(1)
                        }
                    }
                    "step" => {
                        if let Some(c2) = iter.next() {
                            // Provided number of steps
                            if let Ok(num) = c2.parse::<usize>() {
                                DebugCommand::Step(num)
                            } else {
                                DebugCommand::Error("Unable to parse step count.".to_string())
                            }
                        } else {
                            DebugCommand::Step(1)
                        }
                    }
                    "s" => DebugCommand::Step(1),
                    "print" | "p" => {
                        if let Some(c2) = iter.next() {
                            DebugCommand::Print(c2.to_string())
                        } else {
                            DebugCommand::Error("No print target provided.".to_string())
                        }
                    }
                    "dump" | "d" => {
                        if let Some(c2) = iter.next() {
                            if let Ok(start) = u16::from_str_radix(c2, 16) {
                                if let Some(c3) = iter.next() {
                                    if let Ok(end) = u16::from_str_radix(c3, 16) {
                                        DebugCommand::Dump(start..end)
                                    } else {
                                        DebugCommand::Error(
                                            "Unable to parse address end.".to_string(),
                                        )
                                    }
                                } else {
                                    // Just print 10 lines
                                    let end = start + (16 * 10);
                                    DebugCommand::Dump(start..end)
                                }
                            } else {
                                DebugCommand::Error("Unable to parse address start.".to_string())
                            }
                        } else {
                            DebugCommand::Error("No address provided.".to_string())
                        }
                    }
                    "continue" | "c" => DebugCommand::Continue,
                    "help" => DebugCommand::Help,
                    "h" => DebugCommand::Help,
                    "quit" => DebugCommand::Quit,
                    "q" => DebugCommand::Quit,
                    _ => DebugCommand::Help,
                }
            } else {
                DebugCommand::Nothing
            }
        }
    }

    fn print_help() {
        println!("The following are the available debugger commands for the Gabe Emulator:");
        println!("\tbreak add [addr]: Adds a breakpoint at the given 16-bit hexidecimal address location 'addr'.");
        println!("\tbreak remove [addr]: Removes the breakpoint at the given 16-bit hexidecimal address location 'addr'.");
        println!("\tbreak list: Lists all currently tracked breakpoints.");
        println!("\tstep (number): Executes the given number of CPU instructions. If number isn't given, executes once.");
        println!("\tcontinue: Runs the emulator until reaching a breakpoint.");
        println!("\tprint [target]: Prints the state of the given target. Target should be blocks separated by periods, e.g. getting the CPU PC would be \"cpu.pc\"");
        println!("\tdump [addr1] (addr2): Prints the memory value at each address \'a\' given by the range addr1 <= a < addr2. If the end of the range isn't provided, it will print 10 lines.");
        println!("\tquit: Shuts down the debugger and falls into normal code execution for the rest of the emulator run.");
        println!("\thelp: Displays this help text.");
    }
}
