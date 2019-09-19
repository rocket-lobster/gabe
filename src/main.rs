#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;
extern crate ggez;

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
use ggez::conf::*;
use ggez::graphics;
use ggez::{event, event::EventHandler};
use ggez::{Context, ContextBuilder, GameResult};
use std::path::Path;

struct Emulator {
    gb: gb::Gameboy,
}

impl Emulator {
    pub fn power_on(path: impl AsRef<Path>) -> Self {
        Emulator {
            gb: gb::Gameboy::power_on(path).expect("Path invalid"),
        }
    }
}

impl EventHandler for Emulator {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while ggez::timer::check_update_time(ctx, 60) {
            self.gb.step();
        }
        Ok(())
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
        .get_matches();
    let rom_file = matches.value_of("ROM").unwrap();
    let (mut ctx, mut event_loop) = ContextBuilder::new("GaBE", "Joe Thill")
        .conf(initialize_conf())
        .build()
        .unwrap();
    let mut emu = Emulator::power_on(rom_file);
    match event::run(&mut ctx, &mut event_loop, &mut emu) {
        Ok(_) => info!("Exiting"),
        Err(e) => error!("{}", e),
    }
}
