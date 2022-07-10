#[macro_use]
extern crate log;
extern crate env_logger;
mod core;
mod debugger;

use crate::core::gb::Gameboy;

use std::path::Path;

use clap::{App, Arg};
use debugger::{Debugger, DebuggerState};
use minifb::{Key, ScaleMode, Window, WindowOptions};

struct Emulator {
    gb: Gameboy,
    debugger: Debugger,
    current_frame: Box<[u8]>,
    has_new_frame: bool,
}

impl Emulator {
    pub fn power_on(path: impl AsRef<Path>, debug: bool) -> Self {
        let debugger = Debugger::new(debug);
        Emulator {
            gb: Gameboy::power_on(path).expect("Path invalid"),
            debugger,
            current_frame: vec![0; 160 * 144 * 3].into_boxed_slice(),
            has_new_frame: false,
        }
    }
}

fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}

fn _upscale_image(input: Vec<u32>, width: usize, height: usize) -> Vec<u32> {
    assert_eq!(input.len(), width * height);
    // Scale by a 2x factor
    let mut ret: Vec<u32> = vec![0; (width * 2) * (height * 2)];
    for (i, v) in input.iter().enumerate() {
        ret[i * 2] = *v;
        ret[(i * 2) + 1] = *v;
        ret[(i * 2) + (width * 2)] = *v;
        ret[(i * 2) + (width * 2) + 1] = *v;
    }
    ret
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

    let mut emu = Emulator::power_on(rom_file, debug_enabled);

    let mut window = Window::new(
        "Gabe Emulator",
        160,
        144,
        WindowOptions { 
            resize: true, 
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    ).expect("Failed to open window.");

    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut color: u8 = 255;

    // Dummy buffer for testing
    let dummy_buffer: Vec<u8> = (0..69120).map(move |v| {
        if v % 3 == 0 {
            color = match color {
                0 => 85,
                85 => 170,
                170 => 255,
                255 => 0,
                _ => panic!()
            };
        }
        color
    }).collect();

    let _dummy_buffer = dummy_buffer.into_boxed_slice();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if emu.debugger.is_running() {
            // Remove frame limiting while debugging
            window.limit_update_rate(None);
            let action = emu.debugger.update(&emu.gb);
            match action {
                DebuggerState::Running => {
                    if let Some(f) = emu.gb.tick() {
                        emu.current_frame = f;
                        emu.has_new_frame = true;
                    }
                }
                DebuggerState::Stopping => {
                    emu.debugger.quit();
                    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
                }
            }
        } else {
            let frame = emu.gb.step();
            emu.current_frame = frame;
            emu.has_new_frame = true;
        }

        if emu.has_new_frame {
            // Convert the series of u8s into a series of RGB-encoded u32s
            let iter = emu.current_frame.chunks(3);
            let mut image_buffer: Vec<u32> = vec![];
            for chunk in iter {
                let new_val = from_u8_rgb(chunk[0], chunk[1], chunk[2]);
                image_buffer.push(new_val);
            }
            window.update_with_buffer(&image_buffer, 160, 144).unwrap();
        } else {
            // No new buffer, just update input
            window.update();
        }
    }
}
