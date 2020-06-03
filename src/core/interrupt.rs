/// Enumeration of the different possible Gameboy interrupts.
/// The values of each interrupt represent the bitmask when enabling and
/// requesting interrupts of the IE register and IF register respectively
///
/// Order represents the priority of interrupt execution when multiple
/// interrupts are enabled and requested at once.
pub enum InterruptKind {
    /// Vertical Blank interrupt whenever the LCD enters the V-Blank period.
    /// (INT 0x40)
    VBlank = 0b0000_0001,
    /// LCD STAT interrupts, such as when entering H-blank, V-blank, LYC=LY,
    /// and when OAM is being read
    /// (INT 0x48)
    LcdStat = 0b0000_0010,
    /// Timer interrupt for whenever the TIMA register wraps
    /// (INT 0x50)
    Timer = 0b0000_0100,
    /// Serial Port-related interrupt
    /// (INT 0x58)
    Serial = 0b0000_1000,
    /// Joypad Input interrupt for when the joypad registers are set from input
    /// (INT 0x60)
    Joypad = 0b0001_0000,
}


