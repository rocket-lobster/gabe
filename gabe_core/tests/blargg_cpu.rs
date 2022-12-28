mod common;

use std::io::Write;

use gabe_core::*;

#[test]
fn blargg_cpu_instrs() {
    let mut video_sink = common::NullSink;
    let mut audio_sink = common::NullSink;
    let rom_data = common::get_rom_data("tests/roms/cpu_instrs/cpu_instrs.gb").unwrap();
    let mut gb = gb::Gameboy::power_on(rom_data, None);
    let mut result = std::string::String::new();
    loop {
        gb.step(&mut video_sink, &mut audio_sink);
        // Check if SC is $81 to signal serial data in SB
        if let Some(v) = gb.poll_serial() {
            print!("{}", v as char);
            result += &(v as char).to_string();
            std::io::stdout().flush().unwrap();
            if result.contains("Passed all tests") {
                break;
            }
            assert!(!result.contains("Failed"));
        }
    }
}
