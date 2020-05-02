use super::memory::Memory;

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
        assert!(rom.len() <= CART_ROM_SIZE as usize);
        Mbc0 { rom }
    }
}

impl Memory for Mbc0 {
    fn read_byte(&self, addr: u16) -> u8 {
        self.rom[(addr as usize - CART_ROM_START)]
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        self.rom[(addr as usize - CART_ROM_START)] = val;
    }
}
