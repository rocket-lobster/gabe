pub mod mbc0;
pub mod mbc1;
pub mod mbc2;
pub mod mbc3;

use alloc::boxed::Box;
use alloc::fmt;
use alloc::string::String;

/// Error type representing possible errors when using cartridge functions.
#[derive(Debug)]
pub enum CartridgeError {
    /// The operation attempted is unsupported by the cartridge type
    Unsupported(String),
}

impl fmt::Display for CartridgeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
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
    fn read_save_data(&mut self, data: Box<[u8]>) -> Result<(), CartridgeError>;

    /// Writes the current content of the Cartridge's battery-backed RAM into the provided
    /// file location. If not supported by the cartridge or fails to write to the location,
    /// returns CartridgeError.
    fn write_save_data(&self) -> Result<Box<[u8]>, CartridgeError>;
}
