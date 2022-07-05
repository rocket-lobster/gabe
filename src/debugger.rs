use std::io::Write;

use crate::core::gb::{Gameboy, GbDebug};

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
    Evaluate,
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

    /// Stops the debugger and stops the TUI for the remaining
    /// program lifetime
    pub fn quit(&mut self) {
        self.enabled = false;
        self.breakpoints.clear();
    }

    pub fn update(&mut self, state: &Gameboy) -> DebuggerState {
        let mut ret = DebuggerState::Running;
        let debug_state = state.get_debug_state();

        // Check if we're at a breakpoint
        if self.breakpoints.contains(&debug_state.cpu_data.reg.pc) {
            // Stop execution and get next command
            println!(
                "Stopping execution at breakpoint: {}",
                &debug_state.cpu_data.reg.pc
            );
            self.current_command = self.get_command();
        }

        loop {
            match self.current_command {
                DebugCommand::Step(n) => {
                    if n == 0 {
                        println!("Completed steps.");
                    } else {
                        self.current_command = DebugCommand::Step(n - 1);
                        break;
                    }
                }
                DebugCommand::Continue => break,
                DebugCommand::Evaluate => todo!(),
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
        let mut ret: DebugCommand;
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
                // Remaining commands pass back to caller
                DebugCommand::Step(s) => {
                    ret = DebugCommand::Step(s);
                    break;
                }
                DebugCommand::Continue => {
                    ret = DebugCommand::Continue;
                    break;
                }
                DebugCommand::Evaluate => {
                    ret = DebugCommand::Evaluate;
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
                                if c3.is_some() {
                                    if let Ok(addr) = u16::from_str_radix(c3.unwrap(), 16) {
                                        DebugCommand::BreakpointAdd(addr)
                                    } else {
                                        DebugCommand::Error("Unable to parse address.".to_string())
                                    }
                                } else {
                                    DebugCommand::Error("No address provided.".to_string())
                                }
                            }
                            Some("delete") => {
                                if c3.is_some() {
                                    if let Ok(addr) = u16::from_str_radix(c3.unwrap(), 16) {
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
                    "step" => {
                        if let Some(c2) = iter.next() {
                            // Provided number of steps
                            if let Ok(num) = usize::from_str_radix(c2, 10) {
                                DebugCommand::Step(num)
                            } else {
                                DebugCommand::Error("Unable to parse step count.".to_string())
                            }
                        } else {
                            DebugCommand::Step(1)
                        }
                    }
                    "s" => DebugCommand::Step(1),
                    "continue" => DebugCommand::Continue,
                    "c" => DebugCommand::Continue,
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
        println!("\tquit: Shuts down the debugger and falls into normal code execution for the rest of the emulator run.");
        println!("\thelp: Displays this help text.");
    }
}
