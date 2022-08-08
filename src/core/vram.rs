use std::{panic, usize};

use super::mmu::{InterruptKind, Memory};

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
            mode_flag: LCDMode::Mode1,
        }
    }
}

impl Memory for Stat {
    fn read_byte(&self, addr: u16) -> u8 {
        assert_eq!(0xFF41, addr);
        let mut v = 0;
        v |= 1 << 7;
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

#[derive(Copy, Clone, PartialEq, Debug)]
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
        assert!(addr == 0xFF47 || addr == 0xFF48 || addr == 0xFF49);
        let mut ret: u8 = 0;
        ret |= (self.color3 as u8) << 6;
        ret |= (self.color2 as u8) << 4;
        ret |= (self.color1 as u8) << 2;
        ret |= self.color0 as u8;
        ret
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        assert!(addr == 0xFF47 || addr == 0xFF48 || addr == 0xFF49);
        let mut colors: Vec<GrayShades> = vec![];
        for i in 0..4 {
            let v = (val >> (i * 2)) & 0b11;
            colors.push(match v {
                0 => GrayShades::White,
                1 => GrayShades::LightGray,
                2 => GrayShades::DarkGray,
                3 => GrayShades::Black,
                _ => panic!("Bad logic"),
            });
        }
        assert!(colors.len() == 4);
        self.color0 = colors[0];
        self.color1 = colors[1];
        self.color2 = colors[2];
        self.color3 = colors[3];
    }
}

/// Type alias for the rendered screen data
pub type FrameData = Box<[u8]>;

const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;

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

    /// A list of OAM entries that will be drawn during the next scanline draw.
    /// Represented as entries in the OAM, 0-39 (40 total entries)
    /// Cleared and repopulated during Mode 2 (OAM Search)
    /// Read during Mode 3 (Draw scanline)
    obj_list: Vec<u8>,

    /// Data containing the rendered scanlines. Presented as row-major, meaning that
    /// the first (top-left) pixel is represented by the first 3 values, the next pixel to the right is
    /// represented by the next 3 values, and the next row doesn't begin until the SCREEN_WIDTH * 3 value.
    screen_data: FrameData,

    /// If true, a new frame has been completed for rendering. Can be requested from VRAM as long as
    /// LCD is still within V-Blank
    has_new_frame: bool,

    /// VRAM data
    memory: Vec<u8>,

    /// OAM Data
    oam: Vec<u8>,
}

impl Vram {
    pub fn power_on() -> Self {
        let mut ret = Vram {
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
            obj_list: Vec::with_capacity(40),
            screen_data: vec![0x0; 3 * SCREEN_WIDTH * SCREEN_HEIGHT].into_boxed_slice(),
            has_new_frame: false,
            memory: vec![0; 0x2000],
            oam: vec![0; 0xA0],
        };

        ret.bgp.write_byte(0xFF47, 0xFC);

        ret
    }

    pub fn update(&mut self, cycles: usize) -> Option<Vec<InterruptKind>> {
        let mut interrupts: Vec<InterruptKind> = vec![];

        // If LCD is disabled, nothing is done, blank display
        if !self.lcdc.lcd_enable || cycles == 0 {
            return None;
        }

        // Each scanline is 456 dots (114 CPU cycles) long and consists of
        // mode 2 (OAM search), mode 3 (active picture), and mode 0 (horizontal blanking).
        // Mode 2 is 80 dots long (2 for each OAM entry), mode 3 is about 168 plus about 10 more
        // for each sprite on a given line, and mode 0 is the rest. After 144 scanlines are drawn
        // are 10 lines of mode 1 (vertical blanking), for a total of 154 lines or 70224 dots per screen.
        // The CPU can't see VRAM (writes are ignored and reads are $FF) during mode 3, but it can during other modes.
        // The CPU can't see OAM during modes 2 and 3, but it can during blanking modes (0 and 1).

        // TODO: If cycles are too high, we don't want to do it all at once. Try and make sure
        // cycles are in groups of 4, i.e. split CPU ticks to cycle operations, not instructions
        self.scanline_cycles += cycles;
        self.stat.lyc_ly_flag = self.ly == self.lyc;

        if self.scanline_cycles >= 456 {
            // Reached end of scanline, wrap around and increment LY
            self.scanline_cycles %= 456;
            self.ly = (self.ly + 1) % 154;
            self.stat.lyc_ly_flag = self.ly == self.lyc;

            if self.stat.lyc_ly_flag
                && self.stat.lyc_ly_interrupt
                && !interrupts.contains(&InterruptKind::LcdStat)
            {
                interrupts.push(InterruptKind::LcdStat);
            }
        }

        if self.ly >= 144 {
            // V-Blank Mode
            if self.stat.mode_flag != LCDMode::Mode1 {
                // If we are just entering V-Blank
                self.stat.mode_flag = LCDMode::Mode1;
                // New frame ready to be rendered
                self.has_new_frame = true;
                interrupts.push(InterruptKind::VBlank);
                if self.stat.vblank_interrupt && !interrupts.contains(&InterruptKind::LcdStat) {
                    interrupts.push(InterruptKind::LcdStat);
                }
            }
        } else if self.scanline_cycles <= 80 {
            // First 80 scanline cycles are in Mode 2
            if self.stat.mode_flag != LCDMode::Mode2 {
                // We are just entering Mode 2
                self.stat.mode_flag = LCDMode::Mode2;
                // Perform the OAM Scan to collect the OBJs on this line
                self.oam_search();
                if self.stat.oam_interrupt && !interrupts.contains(&InterruptKind::LcdStat) {
                    interrupts.push(InterruptKind::LcdStat);
                }
            }
        } else if self.scanline_cycles <= (80 + 172) {
            // TODO: Change cycle check to be non-arbitrary, the number of cycles spent in
            // Mode 3 is variable upon sprite drawing
            if self.stat.mode_flag != LCDMode::Mode3 {
                // Unnecessary, but for consistency
                self.stat.mode_flag = LCDMode::Mode3;
            }
        } else {
            // Spend the rest of the scanline in Mode 0: H-Blank
            if self.stat.mode_flag != LCDMode::Mode0 {
                self.stat.mode_flag = LCDMode::Mode0;
                if self.stat.hblank_interrupt && !interrupts.contains(&InterruptKind::LcdStat) {
                    interrupts.push(InterruptKind::LcdStat);
                }
                // Compute and "render" the scanline into the LCD data
                if self.lcdc.background_enable {
                    self.draw_background();
                }

                if self.lcdc.obj_enable {
                    self.draw_sprites();
                }
            }
        }

        if !interrupts.is_empty() {
            Some(interrupts)
        } else {
            None
        }
    }

    /// Scan the current contents of OAM to find all OBJs that are on the same scanline.
    /// Store into a list that will be searched during draw_sprites() to handle the rendering.
    fn oam_search(&mut self) {
        // Clear old entries since last scanline
        self.obj_list.clear();

        // Check the vertical size of each obj
        let obj_size_adj = if self.lcdc.obj_size_select { 0 } else { 8 };

        // Find all sprites in the current ly row
        for (i, data) in self.oam.chunks(4).enumerate() {
            // Check if the OBJ y-pos is in the range of values that would put a line in the current ly
            if data[0] > self.ly + obj_size_adj && data[0] <= self.ly + 16 {
                // This OBJ is in the current line, add to the list if we have < 10 OBJs already
                if self.obj_list.len() < 10 {
                    self.obj_list.push(i as u8);
                }
            }
        }
    }

    /// Check internal state to determine what horizontal scanline background
    /// pixels should be written to `screen_data`. Includes checking if rendering
    /// window tiles in addition to background tiles. Only called during H-Blank,
    /// and fills the scanline as provided by `ly`, assuming we're not in V-Blank
    fn draw_background(&mut self) {
        // For each pixel in the current scanline given by LY
        for p in 0..SCREEN_WIDTH {
            // Get the tile data index and pixel offsets, either from the window map or the background map
            let (mut tile_data_base, tile_pixel_x, tile_pixel_y) = if self.lcdc.window_enable
                && p as u8 >= self.window_coords.0.saturating_sub(7)
                && self.ly >= self.window_coords.1
            {
                // We are inside the window, so grab window tiles
                let tile_x: u8 = (p as u8 - self.window_coords.0.saturating_sub(7)) / 8;
                let tile_y: u8 = (self.ly - self.window_coords.1) / 8;

                // Get the pixel coordinates for the tile
                let tile_pixel_x: u8 = (p as u8 - self.window_coords.0.saturating_sub(7)) % 8;
                let tile_pixel_y: u8 = (self.ly - self.window_coords.1) % 8;

                // Get the tile map offset from what tile we are using
                let mut tile_map_index: u16 = (tile_y as u16 * 32) + tile_x as u16;

                // Add the relevant base address depending on which tile map is selected
                // Tile Map 0: 0x9800 - 0x8000 = 0x1800
                // Tile Map 1: 0x9C00 - 0x8000 = 0x1C00
                if self.lcdc.window_tile_map_select {
                    tile_map_index += 0x1C00;
                } else {
                    tile_map_index += 0x1800;
                }

                // Grab the tile data index
                (
                    self.memory[tile_map_index as usize] as u16,
                    tile_pixel_x,
                    tile_pixel_y,
                )
            } else {
                // No window, just grab from background map using scroll coords
                let tile_x: u8 = self.scroll_coords.0.wrapping_add(p as u8) / 8;
                let tile_y: u8 = self.scroll_coords.1.wrapping_add(self.ly) / 8;

                // Get the pixel coordinates for the tile
                let tile_pixel_x: u8 = self.scroll_coords.0.wrapping_add(p as u8) % 8;
                let tile_pixel_y: u8 = self.scroll_coords.1.wrapping_add(self.ly) % 8;

                // Get the tile map offset from what tile we are using
                let mut tile_map_index: u16 = (tile_y as u16 * 32) + tile_x as u16;

                // Add the relevant base address depending on which tile map is selected
                // Tile Map 0: 0x9800 - 0x8000 = 0x1800
                // Tile Map 1: 0x9C00 - 0x8000 = 0x1C00
                if self.lcdc.background_tile_map_select {
                    tile_map_index += 0x1C00;
                } else {
                    tile_map_index += 0x1800;
                }

                // Grab the tile data index
                (
                    self.memory[tile_map_index as usize] as u16,
                    tile_pixel_x,
                    tile_pixel_y,
                )
            };

            // Add the relevant base address depending on which tile data is selected
            if !self.lcdc.tile_data_select {
                // The Tile Data index is a signed byte value when using Tile Table 1, reinterpret as an i8.
                let tile_data_signed = i8::from_le_bytes([tile_data_base as u8]);
                // Each Tile Data Table entry is 16 bytes, then offset by signed index.
                // Value of 0 is at 0x1000 into the VRAM, then subtracted or added to by the signed index
                tile_data_base = (((tile_data_signed) as i16 * 16) + 0x1000) as u16;
            } else {
                // Each Tile Data Table entry is 16 bytes, starting at 0x0000
                tile_data_base *= 16;
            }

            // Each set of 2 bytes represets the least and most signficant bits in the tile's color number, respectively,
            // for each line of 8 pixels in the tile.
            // Byte 0-1 is first line, Byte 2-3 is second line, etc.
            // Offset the line we're looking for by applying the tile pixel y-offset, and grab both color bytes
            let tile_colors_lsb =
                self.memory[(tile_data_base + (tile_pixel_y as u16 * 2)) as usize];
            let tile_colors_msb =
                self.memory[(tile_data_base + (tile_pixel_y as u16 * 2) + 1) as usize];

            let pixel_shift = tile_pixel_x ^ 0x7;
            let tile_color_number = (((tile_colors_msb >> pixel_shift) & 0x1) << 1)
                | ((tile_colors_lsb >> pixel_shift) & 0x1);

            let pixel_shade = match tile_color_number {
                0 => self.bgp.color0,
                1 => self.bgp.color1,
                2 => self.bgp.color2,
                3 => self.bgp.color3,
                _ => panic!("Incorrect color number selection logic."),
            };

            let pixel_rgb = Self::shade_to_rgb_u8(&pixel_shade);

            self.screen_data[((self.ly as usize * (SCREEN_WIDTH * 3)) + (p * 3))] = pixel_rgb.0;
            self.screen_data[((self.ly as usize * (SCREEN_WIDTH * 3)) + (p * 3) + 1)] = pixel_rgb.1;
            self.screen_data[((self.ly as usize * (SCREEN_WIDTH * 3)) + (p * 3) + 2)] = pixel_rgb.2;
        }
    }

    /// Called after `draw_background` fills scanline `ly` with data inside `screen_data`
    /// with background and window tiles. Goes through OBJ memory to determine the
    /// sprites to be drawn over the background tiles, and writes them in the same
    /// `ly` scanline within `screen_data`.
    fn draw_sprites(&mut self) {
        for p in 0..SCREEN_WIDTH {
            let mut lowest_x = 0xFFu8;
            // Once all OBJs are found, go through the line and check the valid OBJs for the current scanline pixel being placed
            // Go in reverse so that the first valid OAM entries override past ones
            for i in self.obj_list.iter().rev() {
                let y_pos = self.oam[(i * 4) as usize];
                let x_pos = self.oam[((i * 4) + 1) as usize];
                let tile_idx = self.oam[((i * 4) + 2) as usize];
                let attribs = self.oam[((i * 4) + 3) as usize];

                // Check x-pos for this OBJ
                if x_pos > p as u8 && x_pos <= p as u8 + 8 {
                    let tile_pixel_x = p as u8 + 8 - x_pos;
                    let mut tile_pixel_y = (self.ly as u8 + 16).wrapping_sub(y_pos);

                    // Parse attributes
                    let bg_prio = (attribs & 0b1000_0000) != 0;
                    let y_flip = (attribs & 0b0100_0000) != 0;
                    let x_flip = (attribs & 0b0010_0000) != 0;
                    let obp1 = (attribs & 0b0001_0000) != 0;

                    // Get the location of the tile data, starting at 0x8000
                    // Internally, we start at 0x0000
                    let tile_data_base = if self.lcdc.obj_size_select {
                        // 8x16
                        if (tile_pixel_y > 7 && !y_flip) || (tile_pixel_y <= 7 && y_flip) {
                            // Bottom tile
                            (tile_idx | 0x01) as u16 * 16
                        } else {
                            // Top tile
                            (tile_idx & 0xFE) as u16 * 16
                        }
                    } else {
                        tile_idx as u16 * 16
                    };

                    if y_flip {
                        // Invert the bits and mask the lower 3 to get the new line offset
                        tile_pixel_y = !tile_pixel_y & 0x7;
                    } else {
                        // Just mask the lower 3 bits to contain it within the given tile
                        tile_pixel_y &= 0x7
                    }

                    // Each set of 2 bytes represets the least and most signficant bits in the tile's color number, respectively,
                    // for each line of 8 pixels in the tile.
                    // Byte 0-1 is first line, Byte 2-3 is second line, etc.
                    // Offset the line we're looking for by applying the tile pixel y-offset, and grab both color bytes
                    let tile_colors_lsb =
                        self.memory[(tile_data_base + (tile_pixel_y as u16 * 2)) as usize];
                    let tile_colors_msb =
                        self.memory[(tile_data_base + (tile_pixel_y as u16 * 2) + 1) as usize];

                    // Which pixel in the line we shift over changes on the status of x_flip
                    let pixel_shift = if x_flip {
                        tile_pixel_x
                    } else {
                        !tile_pixel_x & 0x7
                    };

                    let tile_color_number = (((tile_colors_msb >> pixel_shift) & 0x1) << 1)
                        | ((tile_colors_lsb >> pixel_shift) & 0x1);

                    let pixel_shade = if obp1 {
                        match tile_color_number {
                            0 => continue, // Color 0 is transparent, ignore
                            1 => self.obp1.color1,
                            2 => self.obp1.color2,
                            3 => self.obp1.color3,
                            _ => panic!("Incorrect color number selection logic."),
                        }
                    } else {
                        match tile_color_number {
                            0 => continue, // Color 0 is transparent, ignore
                            1 => self.obp0.color1,
                            2 => self.obp0.color2,
                            3 => self.obp0.color3,
                            _ => panic!("Incorrect color number selection logic."),
                        }
                    };

                    if x_pos <= lowest_x {
                        // This OBJ has higher priority than any previous one
                        lowest_x = x_pos;
                    }

                    let pixel_rgb = Self::shade_to_rgb_u8(&pixel_shade);

                    self.screen_data[((self.ly as usize * (SCREEN_WIDTH * 3)) + (p * 3))] =
                        pixel_rgb.0;
                    self.screen_data[((self.ly as usize * (SCREEN_WIDTH * 3)) + (p * 3) + 1)] =
                        pixel_rgb.1;
                    self.screen_data[((self.ly as usize * (SCREEN_WIDTH * 3)) + (p * 3) + 2)] =
                        pixel_rgb.2;
                }
            }
        }
    }

    /// Converts the given GrayShade enum value into a tuple of
    /// u8 values representing the RGB of the shade
    fn shade_to_rgb_u8(shade: &GrayShades) -> (u8, u8, u8) {
        match shade {
            GrayShades::Black => (0, 0, 0),
            GrayShades::DarkGray => (85, 85, 85),
            GrayShades::LightGray => (170, 170, 170),
            GrayShades::White => (255, 255, 255),
        }
    }

    /// Returns if there's a new frame completed and ready to render. Call this before
    /// calling `request_frame`, unless multiple copies of the same frame are needed.
    pub fn new_frame_ready(&self) -> bool {
        self.has_new_frame
    }

    /// Request a frame to display from the LCD controller. Only returns screen data during
    /// V-Blank, otherwise returns None.
    pub fn request_frame(&mut self) -> Option<FrameData> {
        if self.stat.mode_flag == LCDMode::Mode1 {
            // Frame has been requested, so frame is stale until another is rendered.
            self.has_new_frame = false;
            Some(self.screen_data.clone())
        } else {
            None
        }
    }
}

impl Memory for Vram {
    fn read_byte(&self, addr: u16) -> u8 {
        // TODO: Limit reads depending on Mode
        match addr {
            0x8000..=0x9FFF => self.memory[(addr - 0x8000) as usize],
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize],
            0xFF40 => self.lcdc.read_byte(addr),
            0xFF41 => self.stat.read_byte(addr),
            0xFF42 => self.scroll_coords.1,
            0xFF43 => self.scroll_coords.0,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.bgp.read_byte(addr),
            0xFF48 => self.obp0.read_byte(addr),
            0xFF49 => self.obp1.read_byte(addr),
            0xFF4A => self.window_coords.1,
            0xFF4B => self.window_coords.0,
            _ => {
                error!("Unassigned read in VRAM: {:X}", addr);
                0xFF
            }
        }
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        // TODO: Limit writes depending on Mode
        match addr {
            0x8000..=0x9FFF => self.memory[(addr - 0x8000) as usize] = val,
            0xFE00..=0xFE9F => self.oam[(addr - 0xFE00) as usize] = val,
            0xFF40 => {
                self.lcdc.write_byte(addr, val);
                if !self.lcdc.lcd_enable {
                    // LCD disabled, reset all LCD driver variables
                    self.ly = 0;
                    self.scanline_cycles = 0;
                    self.stat.mode_flag = LCDMode::Mode0;
                    for i in 0..self.screen_data.len() {
                        // Clear all screen data to white
                        self.screen_data[i] = 255;
                    }
                }
            }
            0xFF41 => self.stat.write_byte(addr, val),
            0xFF42 => self.scroll_coords.1 = val,
            0xFF43 => self.scroll_coords.0 = val,
            0xFF44 => self.ly = 0x0,
            0xFF45 => self.lyc = val,
            0xFF47 => self.bgp.write_byte(addr, val),
            0xFF48 => self.obp0.write_byte(addr, val),
            0xFF49 => self.obp1.write_byte(addr, val),
            0xFF4A => self.window_coords.1 = val,
            0xFF4B => self.window_coords.0 = val,
            _ => {
                error!("Unassigned write in VRAM: {:X}", addr);
                ()
            }
        }
    }
}

#[cfg(test)]
mod vram_tests {
    use super::*;
    #[test]
    fn lcdc_read_write() {
        let mut lcdc: Lcdc = Lcdc::power_on();
        lcdc.write_byte(0xFF40, 0b1001_1010);
        assert_eq!(true, lcdc.lcd_enable);
        assert_eq!(false, lcdc.window_tile_map_select);
        assert_eq!(false, lcdc.window_enable);
        assert_eq!(true, lcdc.tile_data_select);
        assert_eq!(true, lcdc.background_tile_map_select);
        assert_eq!(false, lcdc.obj_size_select);
        assert_eq!(true, lcdc.obj_enable);
        assert_eq!(false, lcdc.background_enable);
        lcdc = Lcdc {
            lcd_enable: false,
            window_tile_map_select: true,
            window_enable: true,
            tile_data_select: false,
            background_tile_map_select: true,
            obj_size_select: false,
            obj_enable: false,
            background_enable: true,
        };
        let v = lcdc.read_byte(0xFF40);
        assert_eq!(0b0110_1001, v);
    }

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
        assert_eq!(0b1010_1110, v);
    }

    #[test]
    fn palette_read_write() {
        let mut p = PaletteData::init();
        p.write_byte(0xFF47, 0b1101_1000);
        assert_eq!(GrayShades::White, p.color0);
        assert_eq!(GrayShades::DarkGray, p.color1);
        assert_eq!(GrayShades::LightGray, p.color2);
        assert_eq!(GrayShades::Black, p.color3);
        assert_eq!(0b1101_1000, p.read_byte(0xFF47));
    }
}
