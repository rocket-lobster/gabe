#[macro_use]
extern crate log;
extern crate env_logger;
mod core;
mod debugger;

use crate::core::gb::{Gameboy, GbKeys};

use std::{path::Path, io::{Read, Write}, fs::File};

use clap::{App, Arg};
use debugger::{Debugger, DebuggerState};
use minifb::{Key, ScaleMode, Window, WindowOptions};

struct Emulator {
    gb: Gameboy,
    debugger: Debugger,
    current_frame: Box<[u8]>,
}

impl Emulator {
    pub fn power_on(path: impl AsRef<Path>, debug: bool) -> Self {
        let debugger = Debugger::new(debug);
        Emulator {
            gb: Gameboy::power_on(path).expect("Path invalid"),
            debugger,
            current_frame: vec![0; 160 * 144 * 3].into_boxed_slice(),
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
        .arg(
            Arg::with_name("disassemble")
                .help("Creates a disassembly output file from the given ROM instead of running.")
                .long("disassemble")
        )
        .get_matches();
    let rom_file = matches.value_of("ROM").unwrap();
    let debug_enabled = matches.is_present("debug");
    let do_disassemble = matches.is_present("disassemble");

    if do_disassemble {
        println!("Generating disassembled file from {}", rom_file);
        disassemble_to_file(rom_file).expect("Error with I/O, exiting...");
        println!("Diassembly of {} completed successfully! Exiting.", rom_file);
        return;
    }

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
    )
    .expect("Failed to open window.");

    if debug_enabled {
        // No frame limiting while debugging
        window.limit_update_rate(None);
    } else {
        // 60 fps framelimit
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    }

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if emu.debugger.is_running() {
            let action = emu.debugger.update(&emu.gb);
            match action {
                DebuggerState::Running => {
                    // Ignore frames
                    let keys = get_key_states(&window);
                    if keys.is_empty() {
                        emu.gb.tick(None);
                    } else {
                        emu.gb.tick(Some(keys.as_slice()));
                    }
                }
                DebuggerState::Stopping => {
                    emu.debugger.quit();
                    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
                }
            }
            window.update();
        } else {
            let keys = get_key_states(&window);
            let keys_pressed = if keys.is_empty() {
                None
            } else {
                Some(keys.as_slice())
            };

            let frame = emu.gb.step(keys_pressed);
            emu.current_frame = frame;
            // Convert the series of u8s into a series of RGB-encoded u32s
            let iter = emu.current_frame.chunks(3);
            let mut image_buffer: Vec<u32> = vec![];
            for chunk in iter {
                let new_val = from_u8_rgb(chunk[0], chunk[1], chunk[2]);
                image_buffer.push(new_val);
            }
            let keys = window.get_keys();
            if keys.contains(&Key::LeftCtrl) && keys.contains(&Key::D) && debug_enabled {
                // Fall back into debug mode on next update
                println!("Received debug command, enabling debugger...");
                emu.debugger.start();
            }
            window.update_with_buffer(&image_buffer, 160, 144).unwrap();
        }
    }
}


fn disassemble_to_file(path: impl AsRef<Path>) -> Result<(), std::io::Error> {
    let mut in_file = File::open(path.as_ref())?;
    let mut out_file = File::create("output.asm")?;
    let mut rom_data = Vec::new();
    in_file.read_to_end(&mut rom_data)?;
    let disasm = core::disassemble::disassemble_block(rom_data.into_boxed_slice(), 0);
    for (p, s) in disasm {
        out_file.write_all(format!("0x{:04X}: {}\n", p, s).as_bytes())?;
    }
    Ok(())
} 

fn get_key_states(window: &Window) -> Vec<GbKeys> {
    let mut ret: Vec<GbKeys> = vec![];
    window.get_keys().iter().for_each(|key| {
        match key {
            Key::Z => ret.push(GbKeys::A),
            Key::X => ret.push(GbKeys::B),
            Key::Enter => ret.push(GbKeys::Start),
            Key::Backspace => ret.push(GbKeys::Select),
            Key::Up => ret.push(GbKeys::Up),
            Key::Down => ret.push(GbKeys::Down),
            Key::Left => ret.push(GbKeys::Left),
            Key::Right => ret.push(GbKeys::Right),
            _ => (),
        }
    });
        ret
}