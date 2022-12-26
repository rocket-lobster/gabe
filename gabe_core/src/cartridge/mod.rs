pub mod mbc0;
pub mod mbc1;
pub mod mbc2;

use std::fmt;

/// Error type representing possible errors when using cartridge functions.
#[derive(Debug)]
pub enum CartridgeError {
    /// The operation involved file I/O which failed, providing the underlying io::Error
    Io(std::io::Error),
    /// The operation attempted is unsupported by the cartridge type
    Unsupported(String),
}

impl fmt::Display for CartridgeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CartridgeError::Io(ref e) => write!(f, "I/O Error: {}", e),
            CartridgeError::Unsupported(ref s) => {
                write!(f, "Unsupported function attempted: {}", s)
            }
        }
    }
}

/// Trait representing the functionality that a Gameboy cartridge can perform for the rest of the system.
/// Contains all possible functions for a cartridge, but different Memory Bank Controllers (MBCs) may not
/// support any given function, in which case an error will be returned.
pub trait Cartridge: super::mmu::Memory {
    /// Writes the current content of the Cartridge's battery-backed RAM into the provided
    /// file location. If not supported by the cartridge or fails to write to the location,
    /// returns CartridgeError.
    fn read_save_file(&mut self, file: &mut std::fs::File) -> Result<(), CartridgeError>;

    /// Writes the current content of the Cartridge's battery-backed RAM into the provided
    /// file location. If not supported by the cartridge or fails to write to the location,
    /// returns CartridgeError.
    fn write_save_file(&self, file: &mut std::fs::File) -> Result<(), CartridgeError>;
}
