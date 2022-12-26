use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{io, panic};

use super::apu::Apu;
use super::cartridge::Cartridge;
use super::joypad::Joypad;
use super::serial::Serial;
use super::sink::*;
use super::timer::Timer;
use super::vram::Vram;
use super::wram::Wram;

/// The possible states of a DMA transfer running within the MMU. Until a write is performed
/// at 0xFF46, the state will always be `Stopped`. Once a valid write at 0xFF46 occurs, the
/// state is set to `Starting` to begin during the next MMU update at the provided u8 value.
/// The value is the upper byte of the starting address, i.e. a value of 0x80 written will start
/// the DMA at 0x8000 and stop at 0x809F.
/// `Running` comes with a u16 value representing the current address the DMA is at. Multiple writes
/// will be performed during an MMU update, so this tracks the value between `update` calls.
#[derive(PartialEq)]
enum DmaState {
    Stopped,
    Starting(u8),
    Running(u16),
}

/// Enumeration of the different possible Gameboy interrupts.
/// The values of each interrupt represent the bitmask when enabling and
/// requesting interrupts of the IE register and IF register respectively
///
/// Order represents the priority of interrupt execution when multiple
/// interrupts are enabled and requested at once.
#[derive(PartialEq)]
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

/// Trait representing a piece of memory in the system that can have bytes read and written to.
/// write/read words are just composed from write/read byte, so implementors only need to implement
/// `read_byte` and `write_byte`.
pub trait Memory {
    fn read_byte(&self, addr: u16) -> u8;
    fn read_word(&self, addr: u16) -> u16 {
        (u16::from(self.read_byte(addr))) | (u16::from(self.read_byte(addr + 1)) << 8)
    }
    fn write_byte(&mut self, addr: u16, val: u8);
    fn write_word(&mut self, addr: u16, val: u16) {
        self.write_byte(addr, (val & 0xFF) as u8);
        self.write_byte(addr + 1, (val >> 8) as u8);
    }
}

/// The state of all Gameboy memory, both internal memory and external cartridge memory
///
/// This structure is used whenever the CPU needs to write into or read from memory,
/// and then each block provides the services necessary when updated. MMU only handles
/// reading and writing into each block, no logic is performed otherwise.
pub struct Mmu {
    cart: Box<dyn Cartridge>,
    apu: Apu,
    vram: Vram,
    wram: Wram,
    timer: Timer,
    pub joypad: Joypad,
    serial: Serial,
    hram: [u8; 0x7F],
    intf: u8,
    ie: u8,
    dma_state: DmaState,
    previous_dma: u8,
}

impl Mmu {
    /// Initializes the MMU with the given ROM path.
    /// Opens the given file and reads cartridge header information to find
    /// the MBC type.
    pub fn power_on(path: impl AsRef<Path>) -> io::Result<Self> {
        use super::cartridge::mbc0::Mbc0;
        use super::cartridge::mbc1::Mbc1;
        use super::cartridge::mbc2::Mbc2;

        let mut f = File::open(path.as_ref())?;
        let mut rom_data = Vec::new();
        f.read_to_end(&mut rom_data)?;
        let title =
            std::str::from_utf8(&rom_data[0x134..0x13F]).map_or_else(|_| "Invalid Title", |v| v);
        let rom_size = rom_data[0x148];
        let ram_size = rom_data[0x149];
        info!("Cartridge Info:");
        info!("\tTitle: {}", title);
        info!("\tROM Size: {} KiB", 32 * (1 << rom_size));
        match ram_size {
            0x0 | 0x1 => info!("\tRAM Size: None"),
            0x2 => info!("\tRAM Size: 8 KiB"),
            0x3 => info!("\tRAM Size: 32 KiB"),
            0x4 => info!("\tRAM Size: 128 KiB"),
            0x5 => info!("\tRAM Size: 64 KiB"),
            _ => info!("\tRAM Size: Unknown"),
        };
        let cart: Box<dyn Cartridge> = match rom_data[0x147] {
            0x00 => {
                info!("\tMBC Type: MBC0/No MBC.");
                Box::new(Mbc0::power_on(rom_data))
            }
            0x01 => {
                info!("\tMBC Type: MBC1 w/o RAM");
                Box::new(Mbc1::power_on(rom_data, rom_size, 0, false))
            }
            0x02 => {
                info!("\tMBC Type: MBC1 w/ RAM");
                Box::new(Mbc1::power_on(rom_data, rom_size, ram_size, false))
            }
            0x03 => {
                info!("\tMBC Type: MBC1 w/ RAM and Battery");
                Box::new(Mbc1::power_on(rom_data, rom_size, ram_size, true))
            }
            0x05 => {
                info!("\tMBC Type: MBC2");
                Box::new(Mbc2::power_on(rom_data, rom_size, false))
            }
            0x06 => {
                info!("\tMBC Type: MBC2 w/ Battery");
                Box::new(Mbc2::power_on(rom_data, rom_size, true))
            }
            _ => unimplemented!("MBC value {:02X} not supported!", rom_data[0x147]),
        };
        let mmu = Mmu {
            cart,
            apu: Apu::power_on(),
            vram: Vram::power_on(),
            wram: Wram::power_on(),
            timer: Timer::power_on(),
            joypad: Joypad::power_on(),
            serial: Serial::power_on(),
            hram: [0; 0x7F],
            intf: 0xE1,
            ie: 0x00,
            dma_state: DmaState::Stopped,
            previous_dma: 0xFF,
        };

        Ok(mmu)
    }

    /// Updates all memory components to align with the number of cycles
    /// run by the CPU, given by `cycles`.
    /// Handles updates in response to Interrupts being returned by each
    /// block, for the CPU to handle on the next fetch.
    /// If a frame was completed during execution, return `FrameData` to caller,
    /// otherwise return `None`
    pub fn update(&mut self, cycles: u32, video_sink: &mut dyn Sink<VideoFrame>, audio_sink: &mut dyn Sink<AudioFrame>) {
        if self.dma_state != DmaState::Stopped {
            self.dma_state = self.run_dma(cycles);
        }
        // Update APU
        self.apu.update(cycles, audio_sink);

        // Update Joypad
        if let Some(i) = self.joypad.update() {
            self.request_interrupt(i);
        }

        // Update Timers
        if let Some(i) = self.timer.update(cycles) {
            self.request_interrupt(i);
        }
        // Update VRAM
        if let Some(i) = self.vram.update(cycles, video_sink) {
            for interrupt in i {
                self.request_interrupt(interrupt);
            }
        }
    }

    /// Takes the given Interrupt enum value, and sets the corresponding bit
    /// in the IF register. CPU will run interrupt handler on next fetch cycle.
    pub fn request_interrupt(&mut self, int: InterruptKind) {
        // Grab the IF register of current interrupt requests
        let mut int_flag = self.read_byte(0xFF0F);
        int_flag |= int as u8;
        self.write_byte(0xFF0F, int_flag);
    }

    /// Debug function. Returns a simple Vec of the requested range of data. Only returns
    /// data visible to MMU, so any non-selected banks or block-internal data not memory-mapped
    /// will not be returned.
    pub fn get_memory_range(&self, range: std::ops::Range<usize>) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::new();
        for addr in range {
            // Check the bounds of u16
            if addr <= u16::MAX as usize {
                vec.push(self.read_byte(addr as u16));
            }
        }
        vec
    }

    /// Run the DMA for the remaining
    /// 671 cycles roughly needed for full DMA transfer.
    /// It takes about 160 us for a full DMA, which is a little more than
    /// 1 us per cycle. Doing 1-to-1 cycles into a write of data for simplicity
    /// even though that will complete DMA a *bit* faster than hardware.
    fn run_dma(&mut self, cycles: u32) -> DmaState {
        match self.dma_state {
            DmaState::Starting(s) => {
                let addr = (s as u16) << 8;
                for i in 0..cycles {
                    let src_addr = addr + i as u16;
                    let val = match src_addr {
                        0x0000..=0x7F9F => self.cart.read_byte(src_addr),
                        0x8000..=0x9F9F => self.vram.read_byte(src_addr),
                        0xA000..=0xBF9F => self.cart.read_byte(src_addr),
                        0xC000..=0xF19F => self.wram.read_byte(src_addr),
                        _ => panic!("Invalid DMA read location {:4X}", src_addr),
                    };
                    let oam_addr = 0xFE00 | (src_addr & 0xFF);
                    self.vram.write_byte(oam_addr, val);
                }
                DmaState::Running(addr + cycles as u16)
            }
            DmaState::Running(a) => {
                let addr = a;
                for i in 0..cycles {
                    let src_addr = addr + i as u16;
                    if src_addr & 0xFF >= 0xA0 {
                        // DMA complete, return Stopped
                        trace!("DMA Transfer complete.");
                        return DmaState::Stopped;
                    } else {
                        let val = match src_addr {
                            0x0000..=0x7F9F => self.cart.read_byte(src_addr),
                            0x8000..=0x9F9F => self.vram.read_byte(src_addr),
                            0xA000..=0xBF9F => self.cart.read_byte(src_addr),
                            0xC000..=0xF19F => self.wram.read_byte(src_addr),
                            _ => panic!("Invalid DMA read location {:4X}", src_addr),
                        };
                        let oam_addr = 0xFE00 | (src_addr & 0xFF);
                        self.vram.write_byte(oam_addr, val);
                    }
                }
                DmaState::Running(addr + cycles as u16)
            }
            DmaState::Stopped => DmaState::Stopped,
        }
    }

    fn unassigned_read(&self, addr: u16) -> u8 {
        error!("Memory Read at unassigned location {:4X}", addr);
        0xFF
    }

    fn unassigned_write(&mut self, addr: u16, val: u8) {
        error!(
            "Memory Write at unassigned location {:4X} of value {:2X}",
            addr, val
        );
    }
}

impl Memory for Mmu {
    fn read_byte(&self, addr: u16) -> u8 {
        if self.dma_state != DmaState::Stopped && !(0xFF80..=0xFFFE).contains(&addr) {
            warn!(
                "CPU attempting read at {:4X} during DMA, returning 0xFF",
                addr
            );
            0xFF
        } else {
            match addr {
                0x0000..=0x7FFF => self.cart.read_byte(addr),
                0x8000..=0x9FFF => self.vram.read_byte(addr),
                0xA000..=0xBFFF => self.cart.read_byte(addr),
                0xC000..=0xFDFF => self.wram.read_byte(addr),
                0xFE00..=0xFE9F => self.vram.read_byte(addr),
                0xFF00 => self.joypad.read_byte(addr),
                0xFF01..=0xFF02 => self.serial.read_byte(addr),
                0xFF04..=0xFF07 => self.timer.read_byte(addr),
                0xFF0F => self.intf,
                0xFF10..=0xFF3F => self.apu.read_byte(addr),
                0xFF46 => self.previous_dma,
                0xFF40..=0xFF6F => self.vram.read_byte(addr),
                0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize],
                0xFFFF => self.ie,
                _ => self.unassigned_read(addr),
            }
        }
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        if self.dma_state != DmaState::Stopped && !(0xFF80..=0xFFFE).contains(&addr) {
            warn!("CPU attempting write at {:4X} during DMA, ignoring.", addr);
        } else {
            match addr {
                0x0000..=0x7FFF => self.cart.write_byte(addr, val),
                0x8000..=0x9FFF => self.vram.write_byte(addr, val),
                0xA000..=0xBFFF => self.cart.write_byte(addr, val),
                0xC000..=0xFDFF => self.wram.write_byte(addr, val),
                0xFE00..=0xFE9F => self.vram.write_byte(addr, val),
                0xFF00 => self.joypad.write_byte(addr, val),
                0xFF01..=0xFF02 => self.serial.write_byte(addr, val),
                0xFF04..=0xFF07 => self.timer.write_byte(addr, val),
                0xFF0F => self.intf = val,
                0xFF10..=0xFF3F => self.apu.write_byte(addr, val),
                0xFF46 => {
                    trace!("Beginning DMA Transfer at {:2X}00...", val);
                    self.dma_state = DmaState::Starting(val);
                    self.previous_dma = val;
                }
                0xFF40..=0xFF6F => self.vram.write_byte(addr, val),
                0xFF80..=0xFFFE => self.hram[(addr - 0xFF80) as usize] = val,
                0xFFFF => self.ie = val,
                _ => self.unassigned_write(addr, val),
            }
        }
    }
}

#[cfg(test)]
mod mmu_tests {
    #[test]
    fn interrupt_requests() {}
}
