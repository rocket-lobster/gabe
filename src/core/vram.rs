use super::interrupt::InterruptKind;
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


/// Enumeration representing the different LCD Modes that can be active
/// at a given time. Useful for checking the state of the LCD Controller
#[derive(Clone, Copy, PartialEq, Debug)]
enum LCDMode {
    /// Mode 0: The LCD controller is in the H-Blank period and
    /// the CPU can access both the display RAM (8000h-9FFFh)
    /// and OAM (FE00h-FE9Fh)
    Mode0 = 0b00,
    /// Mode 1: The LCD contoller is in the V-Blank period (or the
    /// display is disabled) and the CPU can access both the
    /// display RAM (8000h-9FFFh) and OAM (FE00h-FE9Fh)
    Mode1 = 0b01,
    /// Mode 2: The LCD controller is reading from OAM memory.
    /// The CPU <cannot> access OAM memory (FE00h-FE9Fh)
    /// during this period.
    Mode2 = 0b10,
    /// Mode 3: The LCD controller is reading from both OAM and VRAM,
    /// The CPU <cannot> access OAM and VRAM during this period.
    /// CGB Mode: Cannot access Palette Data (FF69,FF6B) either.
    Mode3 = 0b11,
}

/// 0xFF41: The STAT register in the LCD controller. Contains interrupt flag enables
/// for the different types of LCD STAT interrupts that can be raised. Also contains
/// the LYC=LY flag and Mode flag to indicate which mode is active.
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
    mode_flag: LCDMode,
}

impl Stat {
    pub fn power_on() -> Self {
        Stat {
            lyc_ly_interrupt: false,
            oam_interrupt: false,
            vblank_interrupt: false,
            hblank_interrupt: false,
            lyc_ly_flag: false,
            mode_flag: LCDMode::Mode2,
        }
    }
}

impl Memory for Stat {
    fn read_byte(&self, addr: u16) -> u8 {
        assert_eq!(0xFF41, addr);
        let mut v = 0;
        v |= (self.lyc_ly_interrupt as u8) << 6;
        v |= (self.oam_interrupt as u8) << 5;
        v |= (self.vblank_interrupt as u8) << 4;
        v |= (self.hblank_interrupt as u8) << 3;
        v |= (self.lyc_ly_flag as u8) << 2;
        v |= self.mode_flag as u8;
        v
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        assert_eq!(0xFF41, addr);
        self.lyc_ly_interrupt = (val & 0x40) != 0x0;
        self.oam_interrupt = (val & 0x20) != 0x0;
        self.vblank_interrupt = (val & 0x10) != 0x0;
        self.hblank_interrupt = (val & 0x08) != 0x0;
        self.lyc_ly_flag = (val & 0x04) != 0x0;
        self.mode_flag = match val & 0x03 {
            0b00 => LCDMode::Mode0,
            0b01 => LCDMode::Mode1,
            0b10 => LCDMode::Mode2,
            0b11 => LCDMode::Mode3,
            _ => LCDMode::Mode0,
        };
    }
}

#[derive(Copy, Clone)]
enum GrayShades {
    White = 0,
    LightGray = 1,
    DarkGray = 2,
    Black = 3,
}

struct PaletteData {
    color0: GrayShades,
    color1: GrayShades,
    color2: GrayShades,
    color3: GrayShades,
}

impl PaletteData {
    fn init() -> Self {
        PaletteData {
            color0: GrayShades::White,
            color1: GrayShades::White,
            color2: GrayShades::White,
            color3: GrayShades::White,
        }
    }
}

impl Memory for PaletteData {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF47 => {
                let mut ret: u8 = 0x0;
                ret |= self.color0 as u8;
                ret
            }
            _ => unimplemented!()
        }
    }
    fn write_byte(&mut self, addr: u16, val: u8) {

    }
}

pub struct Vram {
    /// 0xFF40: LCD Control
    lcdc: Lcdc,

    /// 0xFF41: LCDC Status
    stat: Stat,

    /// (0xFF43, 0xFF42): (Scroll X, Scroll Y)
    ///
    /// The X and Y coordinates of top left of the display window. (0,0) represents the top left,
    /// (255, 255) bottom right.
    scroll_coords: (u8, u8),

    /// 0xFF44: LCDC Y-Coordinate
    ///
    /// Indicates the current Y-coordinate on the LCD, 0-153, with 144-153 indicating V-Blank
    /// Writing to this address resets the value to 0.
    ly: u8,

    /// 0xFF45: LY Compare
    ///
    /// Compares its value to LY, and when equal, sets the STAT Coincident Bit and requests
    /// a STAT Interrupt
    lyc: u8,

    /// 0xFF47: BG Palette Data
    ///
    /// Assigns gray shades to the Background and Window tiles, with four different color numbers.
    bgp: PaletteData,

    /// 0xFF48: Object Palette 0 Data
    ///
    /// Assigns gray shades to the sprite palette 0. Only Color Number 3-1 are recognized, with Color Number 0 
    /// always being transparent
    obp0: PaletteData,

    /// 0xFF49: Object Palette 1 Data
    ///
    /// Assigns gray shades to the sprite palette 1. Only Color Number 3-1 are recognized, with Color Number 0 
    /// always being transparent
    obp1: PaletteData,

    /// (0xFF4B, 0xFF4A): (Window X, Window Y)
    ///
    /// The coordinates of the upper left of the Window area. Window X Position is
    /// minus 7 of the value, Window Y Position is normal.
    /// Window X = 7 and Window = 0 represents a Window position at the top left of the LCD
    window_coords: (u8, u8),

    /// Number of cycles, or dots, that the LCD is in the current scanline. Max is 456, and value
    /// determines which Mode the LCD is in. Corresponds to CPU cycles passed in to MMU.
    scanline_cycles: usize,

    memory: Vec<u8>,
}

impl Vram {
    pub fn power_on() -> Self {
        Vram {
            lcdc: Lcdc::power_on(),
            stat: Stat::power_on(),
            scroll_coords: (0x0, 0x0),
            ly: 0x0,
            lyc: 0x0,
            bgp: PaletteData::init(),
            obp0: PaletteData::init(),
            obp1: PaletteData::init(),
            window_coords: (0x0, 0x0),
            scanline_cycles: 0,
            memory: vec![0; 0x2000],
        }
    }

    pub fn update(&mut self, cycles: usize) -> ([u8; 160*144], Option<InterruptKind>) {
        let mut display = [0; 160*144];

        // If LCD is disabled, nothing is done, blank display
        if !self.lcdc.lcd_enable || cycles == 0 {
            return (display, None);
        } 
        // Update the LCD state for the number of cycles given
        // Add cycles to the number of dots to determine
        // - Which mode we stay/change to
        // - Whether we move to the next line (Increment LY)
        // - Do a LYC=LY compare and set the appropriate flags/interrupts
        // - Do an interrupt if we change modes
        // - Do an interrupt if we enter H-Blank/V-Blank
        self.scanline_cycles += cycles;

        if self.scanline_cycles >= 456 {
            // Reached end of scanline, wrap around and increment LY
            self.scanline_cycles %= 456;
            self.ly = (self.ly + 1) % 153; 
        }

        (display, None)
    }
}

impl Memory for Vram {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x8000..=0x9FFF => self.memory[(addr - 0x8000) as usize],
            0xFF40 => self.lcdc.read_byte(addr),
            0xFF41 => self.stat.read_byte(addr),
            0xFF42 => self.scroll_coords.1,
            0xFF43 => self.scroll_coords.0,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF4A => self.window_coords.1,
            0xFF4B => self.window_coords.0,
            _ => panic!("Incorrect addressing in VRAM: {:X}", addr),
        }
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        match addr {
            0x8000..=0x9FFF => self.memory[(addr - 0x8000) as usize] = val,
            0xFF40 => self.lcdc.write_byte(addr, val),
            0xFF41 => self.stat.write_byte(addr, val),
            0xFF42 => self.scroll_coords.1 = val,
            0xFF43 => self.scroll_coords.0 = val,
            0xFF44 => self.ly = 0x0,
            0xFF45 => self.lyc = val,
            0xFF4A => self.window_coords.1 = val,
            0xFF4B => self.window_coords.0 = val,
            _ => panic!("Incorrect addressing in VRAM: {:X}", addr),
        }
    }
}

#[cfg(test)]
mod vram_tests {
    use super::*;
    #[test]
    fn stat_read_write() {
        let mut stat = Stat::power_on();
        stat.write_byte(0xFF41, 0b0110_0101);
        assert_eq!(true, stat.lyc_ly_interrupt);
        assert_eq!(true, stat.oam_interrupt);
        assert_eq!(false, stat.vblank_interrupt);
        assert_eq!(false, stat.hblank_interrupt);
        assert_eq!(true, stat.lyc_ly_flag);
        assert_eq!(LCDMode::Mode1, stat.mode_flag);
        stat = Stat {
            lyc_ly_interrupt: false,
            oam_interrupt: true,
            vblank_interrupt: false,
            hblank_interrupt: true,
            lyc_ly_flag: true,
            mode_flag: LCDMode::Mode2,
        };
        let v = stat.read_byte(0xFF41);
        assert_eq!(0b0010_1110, v);
    }
}
