use super::memory::Memory;

struct Lcdc {
    /// Bit 7: Enables LCD display on true, disables on false.
    /// *Cannot* be disabled outside of V-blank, enforced by logic
    lcd_enable: bool,
    /// Bit 6: Selects which Tile Map to use in VRAM for window display
    /// False means use 0x9800-0x9BFF, true means use 0x9C00-0x9FFF
    window_tile_map_select: bool,
    /// Bit 5: Enables the window display on true, disables on false.
    window_enable: bool,
    /// Bit 4: Selects which Tile Data set to use for both background and window display
    /// False means use 0x8800-0x97FF, true means use 0x8000-0x8FFF
    tile_data_select: bool,
    /// Bit 3: Selects which Tile Map to use in VRAM for background display
    /// False means use 0x9800-0x9BFF, true means use 0x9C00-0x9FFF
    background_tile_map_select: bool,
    /// Bit 2: Selects what size the sprites will be for displaying
    /// False means 8x8, true means 8x16
    obj_size_select: bool,
    /// Bit 1: Enables sprite objects when making display
    obj_enable: bool,
    /// Bit 0: On DMG Gamboy and SGB: When false, background is blank (white)
    /// On CGB in CGB Mode: When false, background and window have no priority over sprites
    /// On CGB in Non-CGB Mode: When false, both background and window become blank (white)
    background_enable: bool,
}

impl Lcdc {
    pub fn power_on() -> Self {
        Lcdc {
            lcd_enable: true,
            window_tile_map_select: false,
            window_enable: false,
            tile_data_select: true,
            background_tile_map_select: false,
            obj_size_select: false,
            obj_enable: false,
            background_enable: true,
        }
    }
}

impl Memory for Lcdc {
    fn read_byte(&self, addr: u16) -> u8 {
        assert_eq!(0xFF40, addr);
        let mut v = 0;
        v |= (self.lcd_enable as u8) << 7;
        v |= (self.window_tile_map_select as u8) << 6;
        v |= (self.window_enable as u8) << 5;
        v |= (self.tile_data_select as u8) << 4;
        v |= (self.background_tile_map_select as u8) << 3;
        v |= (self.obj_size_select as u8) << 2;
        v |= (self.obj_enable as u8) << 1;
        v |= self.background_enable as u8;
        v
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        assert_eq!(0xFF40, addr);
        self.lcd_enable = (val & 0x80) != 0x0;
        self.window_tile_map_select = (val & 0x40) != 0x0;
        self.window_enable = (val & 0x20) != 0x0;
        self.tile_data_select = (val & 0x10) != 0x0;
        self.background_tile_map_select = (val & 0x08) != 0x0;
        self.obj_size_select = (val & 0x04) != 0x0;
        self.obj_enable = (val & 0x02) != 0x0;
        self.background_enable = (val & 0x01) != 0x0;
    }
}

/// 0xFF41: The STAT register in the LCD controller. Contains interrupt flags set as
/// the LCD controller operates.
struct Stat {
    /// Bit 6: LYC=LY Coincidence Interrupt
    lyc_ly_interrupt: bool,
    /// Bit 5: Mode 2 OAM Interrupt
    oam_interrupt: bool,
    /// Bit 4: Mode 1 V-Blank Interrupt
    vblank_interrupt: bool,
    /// Bit 3: Mode 0 H-Blank Interrupt
    hblank_interrupt: bool,
    /// Bit 2: Coincidence Flag (0: LYC!=LY, 1: LYC=LY)
    lyc_ly_flag: bool,
    /// Bit 1-0: Mode Flag
    ///
    ///     - 00: During H-Blank
    ///     - 01: During V-Blank
    ///     - 10: During OAM Search
    ///     - 11: During Data transfer to LCD
    mode_flag: u8,
}

impl Stat {
    pub fn power_on() -> Self {
        Stat {
            lyc_ly_interrupt: false,
            oam_interrupt: false,
            vblank_interrupt: false,
            hblank_interrupt: false,
            lyc_ly_flag: false,
            mode_flag: 0,
        }
    }
}

pub struct Vram {
    lcdc: Lcdc,
    stat: Stat,
    memory: Vec<u8>,
}

impl Vram {
    pub fn power_on() -> Self {
        Vram {
            lcdc: Lcdc::power_on(),
            stat: Stat::power_on(),
            memory: vec![0; 0x2000],
        }
    }
}

impl Memory for Vram {
    fn read_byte(&self, addr: u16) -> u8 {
        assert!(addr >= 0x8000 && addr <= 0x9FFF);
        self.memory[(addr - 0x8000) as usize]
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        assert!(addr >= 0x8000 && addr <= 0x9FFF);
        self.memory[(addr - 0x8000) as usize] = val;
    }
}
