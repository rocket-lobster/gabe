#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;
extern crate ggez;
extern crate tui;

mod cpu;
mod gb;
mod interrupt;
mod mbc0;
mod memory;
mod mmu;
mod timer;
mod vram;
mod wram;

use clap::{App, Arg};
use crossterm;
use ggez::conf::*;
use ggez::graphics;
use ggez::{event, event::EventHandler};
use ggez::{Context, ContextBuilder, GameResult};
use std::io;
use std::path::Path;
use std::time::Duration;
use tui::backend::CrosstermBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, Paragraph, Text};
use tui::Terminal;

struct Emulator {
    gb: gb::Gameboy,
    debug: bool,
    tui: Option<Terminal<CrosstermBackend<io::Stdout>>>,
}

impl Emulator {
    pub fn power_on(path: impl AsRef<Path>, debug: bool) -> Self {
        if debug {
            let stdout = io::stdout();
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.clear().unwrap();
            Emulator {
                gb: gb::Gameboy::power_on(path).expect("Path invalid"),
                debug,
                tui: Some(terminal),
            }
        } else {
            Emulator {
                gb: gb::Gameboy::power_on(path).expect("Path invalid"),
                debug,
                tui: None,
            }
        }
    }
}

impl EventHandler for Emulator {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if self.debug {
            let state = self.gb.get_debug_state();
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
                        crossterm::event::KeyCode::Char('n') => self.gb.tick(),
                        crossterm::event::KeyCode::Char('q') => self.debug = false,
                        _ => (),
                    }
                };
            };
            Ok(())
        } else {
            while ggez::timer::check_update_time(ctx, 60) {
                self.gb.step();
            }
            Ok(())
        }
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);
        graphics::present(ctx)
    }
}

fn initialize_conf() -> Conf {
    let window_mode = WindowMode {
        width: 160.0,
        height: 144.0,
        maximized: false,
        fullscreen_type: FullscreenType::Windowed,
        borderless: false,
        min_width: 0.0,
        min_height: 0.0,
        max_width: 0.0,
        max_height: 0.0,
        resizable: false,
    };

    let window_setup = WindowSetup {
        title: "GaBE".to_owned(),
        samples: NumSamples::Zero,
        vsync: true,
        icon: "".to_owned(),
        srgb: false,
    };

    let backend = Backend::default();

    let modules = ModuleConf {
        gamepad: false,
        audio: true,
    };

    Conf {
        window_mode,
        window_setup,
        backend,
        modules,
    }
}

fn main() {
    env_logger::init();
    let matches = App::new("GaBE")
        .version("0.1")
        .author("Joe Thill <rocketlobster42@gmail.com>")
        .about("Gameboy Emulator in Rust")
        .arg(
            Arg::with_name("ROM")
                .value_name("FILE")
                .help("Game to run in standard GB file format")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("debug")
                .help("Turns on the TUI debugger")
                .short("d")
                .long("debug"),
        )
        .get_matches();
    let rom_file = matches.value_of("ROM").unwrap();
    let debug_enabled = matches.is_present("debug");

    // Rendering window
    let (mut ctx, mut event_loop) = ContextBuilder::new("GaBE", "Joe Thill")
        .conf(initialize_conf())
        .build()
        .unwrap();
    let mut emu = Emulator::power_on(rom_file, debug_enabled);
    match event::run(&mut ctx, &mut event_loop, &mut emu) {
        Ok(_) => info!("Exiting"),
        Err(e) => error!("{}", e),
    }
}
