mod audio_driver;
mod debugger;
mod time_source;

use gabe_core::{gb::*, sink::{VideoFrame, Sink, AudioFrame}};
use time_source::TimeSource;

use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    time::{Instant, SystemTime, Duration}, alloc::System, collections::VecDeque,
};

use clap::{App, Arg};

use debugger::{Debugger, DebuggerState};
use minifb::{Key, ScaleMode, Window, WindowOptions, KeyRepeat};

const CYCLE_TIME_NS: u64 = 238;

struct SystemTimeSource {
    start: Instant
}

impl SystemTimeSource {
    fn new() -> Self {
        SystemTimeSource { start: Instant::now() }
    }
}

impl TimeSource for SystemTimeSource {
    fn time_ns(&self) -> u64 {
        let elapsed = self.start.elapsed();
        elapsed.as_secs() * 1_000_000_000 + (elapsed.subsec_nanos() as u64)
    }
}

struct MostRecentSink {
    inner: Option<VideoFrame>,
}

impl MostRecentSink {
    fn new() -> Self {
        MostRecentSink { inner: None }
    }

    fn has_frame(&self) -> bool {
        self.inner.is_some()
    }

    fn into_inner(self) -> Option<VideoFrame> {
        self.inner
    }
}

impl Sink<VideoFrame> for MostRecentSink {
    fn append(&mut self, value: VideoFrame) {
        self.inner = Some(value);
    }
}

struct SimpleAudioSink {
    inner: VecDeque<AudioFrame>
}

impl Sink<AudioFrame> for SimpleAudioSink {
    fn append(&mut self, value: AudioFrame) {
        self.inner.push_back(value);
    }
}

struct Emulator {
    gb: Gameboy,
    debugger: Debugger,
    emulated_cycles: u64,
}

impl Emulator {
    pub fn power_on(path: impl AsRef<Path>, debug: bool) -> Self {
        let debugger = Debugger::new(debug);
        let gb = Gameboy::power_on(path).expect("Path invalid");
        Emulator {
            gb,
            debugger,
            emulated_cycles: 0,
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

    let mut emu = Emulator::power_on(rom_file, debug_enabled);

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

    let audio_driver = audio_driver::AudioDriver::new(gabe_core::SAMPLE_RATE, 100);

    let mut audio_buffer_sink = audio_driver.sink();

    // let time_source = SystemTimeSource::new();
    let time_source = audio_driver.time_source();

    let start_time_ns = time_source.time_ns();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut video_sink = MostRecentSink::new();
        let mut audio_sink = SimpleAudioSink {
            inner: VecDeque::new()
        };

        let target_emu_time_ns = time_source.time_ns() - start_time_ns;
        let target_emu_cycles = target_emu_time_ns / CYCLE_TIME_NS;

        if emu.debugger.is_running() {
            let action = emu.debugger.update(&emu.gb);
            match action {
                DebuggerState::Running => {
                    // Ignore frames
                    get_key_states(&window, &mut emu.gb);
                }
                DebuggerState::Stopping => {
                    emu.debugger.quit();
                }
            }
            window.update();
        } else {
            while emu.emulated_cycles < target_emu_cycles { 
                emu.emulated_cycles += emu.gb.step(&mut video_sink, &mut audio_sink) as u64;
            }
            
            if let Some(frame) = video_sink.into_inner() {
                let iter = frame.chunks(3);
                // Convert the series of u8s into a series of RGB-encoded u32s
                let image_buffer: Vec<u32> = iter.map(|x| from_u8_rgb(x[0], x[1], x[2])).collect();
                window.update_with_buffer(&image_buffer, 160, 144).unwrap();
                
                let keys = window.get_keys();
                get_key_states(&window, &mut emu.gb);
                if keys.contains(&Key::LeftCtrl) && keys.contains(&Key::D) && debug_enabled {
                    // Fall back into debug mode on next update
                    println!("Received debug command, enabling debugger...");
                    emu.debugger.start();
                }
            }

            audio_buffer_sink.append(audio_sink.inner.as_slices().0);
        }
        spin_sleep::sleep(std::time::Duration::from_millis(1));
    }
}

fn disassemble_to_file(path: impl AsRef<Path>) -> Result<(), std::io::Error> {
    let mut in_file = File::open(path.as_ref())?;
    let mut out_file = File::create("output.asm")?;
    let mut rom_data = Vec::new();
    in_file.read_to_end(&mut rom_data)?;
    let disasm = gabe_core::disassemble::disassemble_block(rom_data.into_boxed_slice(), 0);
    for (p, s) in disasm {
        out_file.write_all(format!("0x{:04X}: {}\n", p, s).as_bytes())?;
    }
    Ok(())
}

fn get_key_states(window: &Window, gb: &mut Gameboy) {
    gb.update_key_state(GbKeys::A, window.is_key_down(Key::X));
    gb.update_key_state(GbKeys::B, window.is_key_down(Key::Z));
    gb.update_key_state(GbKeys::Start, window.is_key_down(Key::Enter));
    gb.update_key_state(GbKeys::Select, window.is_key_down(Key::Backspace));
    gb.update_key_state(GbKeys::Up, window.is_key_down(Key::Up));
    gb.update_key_state(GbKeys::Down, window.is_key_down(Key::Down));
    gb.update_key_state(GbKeys::Left, window.is_key_down(Key::Left));
    gb.update_key_state(GbKeys::Right, window.is_key_down(Key::Right));
}
