#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;
extern crate ggez;

mod core;
mod debugger;

use crate::core::gb::Gameboy;
use clap::{App, Arg};
use debugger::{Debugger, DebuggerState};
use ggez::conf::*;
use ggez::graphics::{self, Color, DrawParam, Image};
use ggez::{event, event::EventHandler};
use ggez::{Context, ContextBuilder, GameResult};
use std::path::Path;

struct Emulator {
    gb: Gameboy,
    debugger: Debugger,
    current_frame: Box<[u8]>
}

impl Emulator {
    pub fn power_on(path: impl AsRef<Path>, debug: bool) -> Self {
        let debugger = Debugger::new(debug);
        Emulator {
            gb: Gameboy::power_on(path).expect("Path invalid"),
            debugger,
            current_frame: vec![].into_boxed_slice(),
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
        
        // Convert GB frame from 3 values per-pixel into 4 values per-pixel to convert into Image
        let mut image_vec: Vec<u8> = vec![];
        let mut alpha_count = 0;
        for i in self.current_frame.into_iter() {
            image_vec.push(*i);
            alpha_count += 1;
            if alpha_count == 3 {
                // Every 3rd value, push a dummy alpha channel value
                image_vec.push(0);
                alpha_count = 0;
            } 
        }
        // There should be one additional element in the array
        assert!(image_vec.len() == (self.current_frame.len() + (160 * 144)));
        let image = Image::from_rgba8(ctx, 160, 144, image_vec.as_mut_slice())?;
        graphics::draw(ctx, &image, DrawParam::default())?;
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
