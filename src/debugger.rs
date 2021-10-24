use crate::core::gb::GbDebug;

use std::io;
use std::time::Duration;

pub struct Debugger {
    enabled: bool,
    breakpoints: Vec<u16>,
}

pub enum DebuggerState {
    /// Move to the next instruction
    Next,
    /// Continue execution until next breakpoint
    Continue,
    /// End the debugger session, ignoring breakpoints
    Quit,
    /// Not enabled and not running TUI
    Disabled,
}

impl Debugger {
    pub fn new(enabled: bool) -> Self {
        Debugger {
            enabled,
            breakpoints: vec![],
        }
    }

    pub fn is_running(&self) -> bool {
        self.enabled
    }

    /// Temporarily suspends debugger until breakpoint is reached
    pub fn suspend(&mut self) {
        self.enabled = false;
    }

    /// Stops the debugger and stops the TUI for the remaining
    /// program lifetime
    pub fn quit(&mut self) {
        self.suspend();
        self.breakpoints.clear();
    }

    pub fn update(&mut self, state: &GbDebug) -> DebuggerState {
        let mut ret = DebuggerState::Disabled;
        // if self.enabled {
            
        // };
        ret
    }
}
