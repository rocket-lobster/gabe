#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;
extern crate ggez;

mod core;
mod debugger;

use crate::core::gb::Gameboy;
use crate::core::FrameData;
use clap::{App, Arg};
use debugger::{Debugger, DebuggerState};
use ggez::conf::*;
use ggez::graphics::{self, Color};
use ggez::{event, event::EventHandler};
use ggez::{Context, ContextBuilder, GameResult};
use std::path::Path;

struct Emulator {
    gb: Gameboy,
    debugger: Debugger,
    current_frame: FrameData
}

impl Emulator {
    pub fn power_on(path: impl AsRef<Path>, debug: bool) -> Self {
        let debugger = Debugger::new(debug);
        Emulator {
            gb: Gameboy::power_on(path).expect("Path invalid"),
            debugger,
            current_frame: [[[0x0; 3]; 160]; 144],
        }
    }
}

impl EventHandler<ggez::GameError> for Emulator {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if self.debugger.is_running() {
            let state = self.gb.get_debug_state();
            let action = self.debugger.update(&state);
            match action {
                DebuggerState::Next => {
                    if let Some(f) = self.gb.tick() {
                        self.current_frame = f;
                    }
                }
                DebuggerState::Continue => self.debugger.suspend(),
                DebuggerState::Quit => self.debugger.quit(),
                _ => (),
            };
            Ok(())
        } else {
            while ggez::timer::check_update_time(ctx, 60) {
                let frame = self.gb.step();
                self.current_frame = frame;
            }
            Ok(())
        }
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::WHITE);
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
        visible: true,
        resize_on_scale_factor_change: false,
    };

    let window_setup = WindowSetup {
        title: "GaBE".to_owned(),
        samples: NumSamples::One,
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
                .help("Turns on the REPL debugger")
                .short("d")
                .long("debug"),
        )
        .get_matches();
    let rom_file = matches.value_of("ROM").unwrap();
    let debug_enabled = matches.is_present("debug");

    // Rendering window
    let (ctx, event_loop) = ContextBuilder::new("GaBE", "Joe Thill")
        .default_conf(initialize_conf())
        .build()
        .unwrap();
    let emu = Emulator::power_on(rom_file, debug_enabled);
    event::run(ctx, event_loop, emu);
}
