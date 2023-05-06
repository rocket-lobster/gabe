mod common;
use gabe_core::*;

#[test]
fn blargg_dmg_sound_01registers() {
    let rom_data = common::get_rom_data("tests/roms/dmg_sound/01-registers.gb").unwrap();
    let mut gb = gb::Gameboy::power_on(rom_data, None);
    assert!(common::run_dmg_sound_case(&mut gb));
}

#[test]
fn blargg_dmg_sound_02lenctr() {
    let rom_data = common::get_rom_data("tests/roms/dmg_sound/02-len_ctr.gb").unwrap();
    let mut gb = gb::Gameboy::power_on(rom_data, None);
    assert!(common::run_dmg_sound_case(&mut gb));
}

#[test]
fn blargg_dmg_sound_03trigger() {
    let rom_data = common::get_rom_data("tests/roms/dmg_sound/03-trigger.gb").unwrap();
    let mut gb = gb::Gameboy::power_on(rom_data, None);
    assert!(common::run_dmg_sound_case(&mut gb));
}

#[test]
fn blargg_dmg_sound_04sweep() {
    let rom_data = common::get_rom_data("tests/roms/dmg_sound/04-sweep.gb").unwrap();
    let mut gb = gb::Gameboy::power_on(rom_data, None);
    assert!(common::run_dmg_sound_case(&mut gb));
}

#[test]
fn blargg_dmg_sound_05sweep_details() {
    let rom_data = common::get_rom_data("tests/roms/dmg_sound/05-sweep_details.gb").unwrap();
    let mut gb = gb::Gameboy::power_on(rom_data, None);
    assert!(common::run_dmg_sound_case(&mut gb));
}

#[test]
fn blargg_dmg_sound_06overflow_trigger() {
    let rom_data = common::get_rom_data("tests/roms/dmg_sound/06-overflow_on_trigger.gb").unwrap();
    let mut gb = gb::Gameboy::power_on(rom_data, None);
    assert!(common::run_dmg_sound_case(&mut gb));
}

#[test]
fn blargg_dmg_sound_07len_sweep_period_sync() {
    let rom_data =
        common::get_rom_data("tests/roms/dmg_sound/07-len_sweep_period_sync.gb").unwrap();
    let mut gb = gb::Gameboy::power_on(rom_data, None);
    assert!(common::run_dmg_sound_case(&mut gb));
}

#[test]
fn blargg_dmg_sound_08len_ctr_during_power() {
    let rom_data = common::get_rom_data("tests/roms/dmg_sound/08-len_ctr_during_power.gb").unwrap();
    let mut gb = gb::Gameboy::power_on(rom_data, None);
    assert!(common::run_dmg_sound_case(&mut gb));
}

#[test]
fn blargg_dmg_sound_09wave_read_while_on() {
    let rom_data = common::get_rom_data("tests/roms/dmg_sound/09-wave_read_while_on.gb").unwrap();
    let mut gb = gb::Gameboy::power_on(rom_data, None);
    assert!(common::run_dmg_sound_case(&mut gb));
}

#[test]
fn blargg_dmg_sound_10wave_trigger_while_on() {
    let rom_data =
        common::get_rom_data("tests/roms/dmg_sound/10-wave_trigger_while_on.gb").unwrap();
    let mut gb = gb::Gameboy::power_on(rom_data, None);
    assert!(common::run_dmg_sound_case(&mut gb));
}

#[test]
fn blargg_dmg_sound_11regs_after_power() {
    let rom_data = common::get_rom_data("tests/roms/dmg_sound/11-regs_after_power.gb").unwrap();
    let mut gb = gb::Gameboy::power_on(rom_data, None);
    assert!(common::run_dmg_sound_case(&mut gb));
}

#[test]
fn blargg_dmg_sound_12wave_write_while_on() {
    let rom_data = common::get_rom_data("tests/roms/dmg_sound/12-wave_write_while_on.gb").unwrap();
    let mut gb = gb::Gameboy::power_on(rom_data, None);
    assert!(common::run_dmg_sound_case(&mut gb));
}
