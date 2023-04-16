# gabe (**Ga**me**b**oy **E**mulator)

A Gameboy emulator written in Rust. Currently supports original DMG games, with planned support for CGB and more. Includes three crates:

- `gabe_core`: The emulator core, provided as a Rust library. Implemented as a `no_std` crate for easy integration with many platforms and frontends. Library provides both ways to run the emulator and means to get debugging data.
- `gabe_cli`: A simple CLI frontend that is used to run games. Provides a REPL debugger as well as a simple `minifb` window.
- `gabe_gui`: The GUI frontend that uses `egui` as a toolkit. Includes easy ROM loading and eventual debugging tools are planned.

The `gabe_gui` crate is the primary frontend being maintained and developed, and should be the first choice to run.

## Game Support / Memory Bank Controllers

- MBC0
- MBC1
- MBC2
- MBC3 (w/out RTC)

## Features

- Saving and Loading with supported games
- Basic Video and Sound Support (DMG-only)
- blargg tests included in Cargo Test suite, along with detection of success/failure

## Planned Features

- Support for most Memory Bank Controllers
- Gameboy Color (CGB) support
    - Double-speed CPU
    - VRAM Color Palette support
    - HDMA Support
    - Misc CGB Registers
- Testing
    - Full blargg test passing/running
    - Unit tests per-module
- GUI Debugging
    - Memory Search
    - All CPU register state
    - REPL/Command-based debugging
    - Breakpoints/watchpoints