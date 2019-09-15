#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;

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
use std::path::Path;

fn main() {
    println!("Hello, world!");
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
    let mut gb = gb::Gameboy::power_on(Path::new(rom_file)).expect("Path invalid");
    gb.step();
}
