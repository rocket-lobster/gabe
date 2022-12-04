#[macro_use]
extern crate log;
extern crate env_logger;
mod core;
mod debugger;

use crate::core::gb::{Gameboy, GbKeys};

use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    sync::{Arc, Mutex}, collections::VecDeque,
};

use clap::{App, Arg};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Sample, SampleFormat,
};
use debugger::{Debugger, DebuggerState};
use minifb::{Key, ScaleMode, Window, WindowOptions};

struct Emulator {
    gb: Gameboy,
    debugger: Debugger,
    audio_buffer: Arc<Mutex<VecDeque<(i16, i16)>>>,
    current_frame: Box<[u8]>,
}

impl Emulator {
    pub fn power_on(path: impl AsRef<Path>, sample_rate: u32, debug: bool) -> Self {
        let debugger = Debugger::new(debug);
        let (gb, audio_buffer) = Gameboy::power_on(path, sample_rate).expect("Path invalid");
        Emulator {
            gb,
            debugger,
            audio_buffer,
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
                .long("disassemble"),
        )
        .get_matches();
    let rom_file = matches.value_of("ROM").unwrap();
    let debug_enabled = matches.is_present("debug");
    let do_disassemble = matches.is_present("disassemble");

    if do_disassemble {
        println!("Generating disassembled file from {}", rom_file);
        disassemble_to_file(rom_file).expect("Error with I/O, exiting...");
        println!(
            "Diassembly of {} completed successfully! Exiting.",
            rom_file
        );
        return;
    }

    // Set up audio device, use default config and device.
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("No audio output device available.");

    let mut supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range
        .next()
        .expect("no supported config?!")
        .with_max_sample_rate();

    let mut emu = Emulator::power_on(rom_file, supported_config.sample_rate().0, debug_enabled);

    // Set up cpal audio stream
    let buf = emu.audio_buffer.clone();
    let err_fn = |err| error!("An error occurred on the output audio stream: {}", err);
    let sample_format = supported_config.sample_format();
    info!("Sound: ");
    info!("\t Sample format: {:?}", sample_format);
    info!("\t Sample rate: {:?}", supported_config.sample_rate().0);
    info!("\t Channels: {:?}", supported_config.channels());
    let config = supported_config.into();
    let stream = match sample_format {
        SampleFormat::F32 => device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                write_from_buffer::<f32>(data, buf.clone());
            },
            err_fn,
        ),
        SampleFormat::I16 => device.build_output_stream(
            &config,
            move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                write_from_buffer::<i16>(data, buf.clone());
            },
            err_fn,
        ),
        SampleFormat::U16 => device.build_output_stream(
            &config,
            move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                write_from_buffer::<u16>(data, buf.clone());
            },
            err_fn,
        ),
    }
    .unwrap();

    let mut window = Window::new(
        "Gabe Emulator",
        160 * 4,
        144 * 4,
        WindowOptions {
            resize: false,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )
    .expect("Failed to open window.");

    // Disable minifb's rate limiting
    window.limit_update_rate(None);

    let mut timestamp = std::time::Instant::now();

    stream.play().unwrap();

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

            let new_ts = std::time::Instant::now();
            let dt = new_ts.duration_since(timestamp);
            timestamp = new_ts;

            if let Some(frame) = emu.gb.step_seconds(dt, keys_pressed) {
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
}

fn write_from_buffer<T: Sample>(data: &mut [T], input_buf: Arc<Mutex<VecDeque<(i16, i16)>>>) {
    let mut input_locked = input_buf.lock().unwrap();

    for chunk in data.chunks_exact_mut(2) {
        if let Some(v) = input_locked.pop_front() {
            chunk[0] = Sample::from(&v.0);
            chunk[1] = Sample::from(&v.1);
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
    window.get_keys().iter().for_each(|key| match key {
        Key::X => ret.push(GbKeys::A),
        Key::Z => ret.push(GbKeys::B),
        Key::Enter => ret.push(GbKeys::Start),
        Key::Backspace => ret.push(GbKeys::Select),
        Key::Up => ret.push(GbKeys::Up),
        Key::Down => ret.push(GbKeys::Down),
        Key::Left => ret.push(GbKeys::Left),
        Key::Right => ret.push(GbKeys::Right),
        _ => (),
    });
    ret
}
