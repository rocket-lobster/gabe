use crate::core::gb::GbDebug;
use crossterm;
use tui::backend::CrosstermBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, Paragraph, Text};
use tui::Terminal;

use std::io;
use std::time::Duration;

pub struct Debugger {
    enabled: bool,
    tui: Option<Terminal<CrosstermBackend<io::Stdout>>>,
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
        if enabled {
            let stdout = io::stdout();
            let backend = CrosstermBackend::new(stdout);
            let mut tui = Terminal::new(backend).unwrap();
            tui.clear().unwrap();
            Debugger {
                enabled,
                tui: Some(tui),
            }
        } else {
            Debugger { enabled, tui: None }
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
        self.tui.as_mut().unwrap().clear().unwrap();
        self.tui = None;
    }

    pub fn update(&mut self, state: &GbDebug) -> DebuggerState {
        let mut ret = DebuggerState::Disabled;
        if self.enabled {
            self.tui
                .as_mut()
                .unwrap()
                .draw(move |mut f| {
                    let chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
                        )
                        .split(f.size());

                    let text = [Text::raw(format!("{}", state.cpu_data))];
                    let paragraph = Paragraph::new(text.iter())
                        .block(Block::default().title("CPU Data").borders(Borders::ALL))
                        .alignment(Alignment::Left)
                        .wrap(false);
                    f.render_widget(paragraph, chunks[0]);
                })
                .unwrap();
            if crossterm::event::poll(Duration::from_millis(100)).unwrap() {
                if let crossterm::event::Event::Key(event) = crossterm::event::read().unwrap() {
                    match event.code {
                        crossterm::event::KeyCode::Char('n') => ret = DebuggerState::Next,
                        crossterm::event::KeyCode::Char('q') => ret = DebuggerState::Quit,
                        _ => (),
                    }
                };
            };
        };
        ret
    }
}
