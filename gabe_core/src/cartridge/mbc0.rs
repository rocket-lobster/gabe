use super::super::mmu::Memory;
use super::{Cartridge, CartridgeError};

const CART_ROM_START: usize = 0x0000;
const CART_ROM_END: usize = 0x7FFF;
const CART_ROM_SIZE: usize = CART_ROM_END - CART_ROM_START + 1;

/// Cartridges that use the MBC0 type don't actually have any (or minimal)
/// circuitry to control memory banks. Such cartridges only have 32 Kb
/// of ROM storage and no RAM storage and no bank switching.
pub struct Mbc0 {
    rom: Vec<u8>,
}

impl Mbc0 {
    pub fn power_on(rom: Vec<u8>) -> Self {
        assert!(rom.len() <= CART_ROM_SIZE);
        Mbc0 { rom }
    }
}

impl Memory for Mbc0 {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.rom[(addr as usize - CART_ROM_START)],
            _ => {
                error!("Unassigned read to MBC0 location {:04X}", addr);
                0xFF
            }
        }
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        error!(
            "Unassigned write to MBC0 location {:04X} of value {:02X}",
            addr, val
        );
    }
}

impl Cartridge for Mbc0 {
    fn read_save_file(&mut self, _file: &mut std::fs::File) -> Result<(), CartridgeError> {
        // No RAM file to write save to, do nothing
        Err(CartridgeError::Unsupported(
            "MBC0 does not support save file writing.".to_string(),
        ))
    }

    fn write_save_file(&self, _file: &mut std::fs::File) -> Result<(), CartridgeError> {
        // No RAM file to write save to, do nothing
        Err(CartridgeError::Unsupported(
            "MBC0 does not support save file writing.".to_string(),
        ))
    }
}
