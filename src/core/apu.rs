use blip_buf::BlipBuf;
use std::sync::{Arc, Mutex};

use super::memory::Memory;

struct SquareChannel1 {
    /// Bit 6-4 - Sweep Time
    sweep_time: u8,
    ///  Bit 3   - Sweep Increase/Decrease
    ///     0: Addition    (frequency increases)
    ///     1: Subtraction (frequency decreases)
    sweep_decrease: bool,
    /// Number of sweep shift (n: 0-7)
    sweep_shift: u8,

    /// Bit 7-6 - Wave Pattern Duty (Read/Write)
    wave_pattern: u8,
    /// Bit 5-0 - Sound length data (Write Only) (t1: 0-63)
    /// Sound Length = (64-t1)*(1/256) seconds.
    /// The Length value is used only if Bit 6 in NR14 is set.
    length_data: u8,

    /// Bit 7-4 - Initial Volume of envelope (0-0Fh) (0=No Sound)
    envelope_vol: u8,
    /// Bit 3   - Envelope Direction (0=Decrease, 1=Increase)
    envelope_increase: bool,
    /// Bit 2-0 - Number of envelope sweep (n: 0-7)
    /// (If zero, stop envelope operation.)
    /// Length of 1 step = n*(1/64) seconds
    envelope_steps: u8,

    /// NR14 Bit 2-0 Upper bits + NR13 Bit 7-0 Lower bits
    /// 11-bit Frequency data x
    /// Frequency = 131072/(2048-x) Hz
    frequency: u16,

    /// Bit 7 - Initial (1=Restart Sound)  (Write Only)
    init_sound: bool,
    /// Bit 6   - Counter/consecutive selection (Read/Write)
    /// (1=Stop output when length in NR11 expires)
    length_enable: bool,

    /// The current cycle count used to synchronize the timing of waveform generation
    /// with the rest of the system
    frequency_cycles: usize,

    /// The period of the frequency timer for waveform generation.
    /// Calculated every time the frequency is changed with the formula:
    ///     Period = 4 * (2048 - frequency)
    frequency_period: u32,

    /// The number of frame sequencer cycles, with a period of 8.
    /// Updates the following:
    /// Length Counter: 0, 2, 4, 6
    /// Volume Envelope: 7
    /// Sweep: 2, 6
    frame_cycles: usize,
    
    /// The current location in the wave pattern given by wave_pattern
    wave_index: usize,

    /// A BlipBuf object that takes input clocks and amplitudes of the channel
    /// and generates samples at the host sample rate
    buffer: BlipBuf,
}

impl SquareChannel1 {
    fn power_on(sample_rate: u32) -> Self {
        // Create buffer with enough samples for 1 second
        // TODO: Probably could be lower
        let mut buf = BlipBuf::new(sample_rate);

        // 4.19 MHz is the system clock rate to convert samples from
        buf.set_rates(4_194_304f64, f64::from(sample_rate));

        SquareChannel1 {
            sweep_time: 0,
            sweep_decrease: false,
            sweep_shift: 0,
            wave_pattern: 0x2,
            wave_index: 0,
            length_data: 0,
            envelope_vol: 0xF,
            envelope_increase: false,
            envelope_steps: 0x3,
            frequency: 0,
            frequency_period: 8192,
            frequency_cycles: 0,
            frame_cycles: 0,
            init_sound: false,
            length_enable: false,
            buffer: buf,
        }
    }

    fn update(&mut self, cycles: usize, frame_cycles: usize) {
        self.frequency_cycles += cycles;

        // Check if the buffer needs to be updated with new samples to match the frequency
        while self.frequency_cycles > self.frequency_period as usize {
            self.frequency_cycles -= self.frequency_period as usize;
            // Get Duty cycle pattern for wave
            let pattern = match self.wave_pattern {
                0x0 => 0b0000_0001, // 12.5%
                0x1 => 0b1000_0001, // 25%
                0x2 => 0b1000_0111, // 50%
                0x3 => 0b0111_1110, // 75%
                _ => unreachable!()
            };
            // Get the current volume based on the volume envelope state
            // Set amplitude to 0 if volume is zero or the channel hasn't been triggered
            // Otherwise amplitude is vol if pattern is high at this step, -vol if pattern is low
            // Put amplitude value into buffer at the next base+period location
        }
    }
}

impl Memory for SquareChannel1 {
    fn read_byte(&self, addr: u16) -> u8 {
        assert!((0xFF10..=0xFF14).contains(&addr));
        match addr {
            0xFF10 => {
                let mut v = 0x0;
                v |= self.sweep_time << 4;
                v |= (self.sweep_decrease as u8) << 3;
                v |= self.sweep_shift;
                v | 0x80
            }
            0xFF11 => {
                let mut v = 0x0;
                v |= self.wave_pattern << 6;
                v | 0x3
            }
            0xFF12 => {
                let mut v = 0x00;
                v |= self.envelope_vol << 4;
                v |= (self.envelope_increase as u8) << 3;
                v |= self.envelope_steps;
                v
            }
            0xFF13 => 0xFF,
            0xFF14 => {
                let mut v = 0x00;
                v |= (self.length_enable as u8) << 6;
                v | 0xBF
            }
            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        assert!((0xFF10..=0xFF14).contains(&addr));
        match addr {
            0xFF10 => {
                self.sweep_time = (val >> 4) & 0x7;
                self.sweep_decrease = (val >> 3) & 0x1 != 0x0;
                self.sweep_shift = val & 0x7;
            }
            0xFF11 => {
                self.wave_pattern = (val >> 6) & 0x3;
                self.length_data = val & 0x3F;
            }
            0xFF12 => {
                self.envelope_vol = (val >> 4) & 0xF;
                self.envelope_increase = (val >> 3) & 0x1 != 0x0;
                self.envelope_steps = val & 0x7;
            }
            0xFF13 => {
                self.frequency &= val as u16;
                self.frequency_period = 4 * (2048 - u32::from(self.frequency));
            }
            0xFF14 => {
                self.init_sound = (val >> 7) & 0x1 != 0x0;
                self.length_enable = (val >> 6) & 0x1 != 0x0;
                self.frequency &= (((val as u16) << 8) & 0x0700) | 0xFF;
                self.frequency_period = 4 * (2048 - u32::from(self.frequency));
            }
            _ => unreachable!(),
        }
    }
}

/// Type alias for easier usage by the caller
pub type AudioBuffer = Arc<Mutex<Vec<(f32, f32)>>>;

pub struct Apu {
    /// Sound Channel 1 - Tone and Sweep
    /// NR10-NR14 ($FF10-$FF14)
    square1: SquareChannel1,

    /// Channel control / ON-OFF / Volume (R/W)
    /// NR50 ($FF24)
    /// Bit 7   - Output Vin to SO2 terminal (1=Enable)
    so2_vin_enable: bool,
    /// Bit 6-4 - SO2 output level (volume)  (0-7)
    so2_volume: u8,
    /// Bit 3   - Output Vin to SO1 terminal (1=Enable)
    so1_vin_enable: bool,
    /// Bit 2-0 - SO1 output level (volume)  (0-7)
    so1_volume: u8,

    /// Selection of Sound output terminal (R/W)
    /// NR51 ($FF25)
    /// Bit 7 - Output sound 4 to SO2 terminal
    /// Bit 6 - Output sound 3 to SO2 terminal
    /// Bit 5 - Output sound 2 to SO2 terminal
    /// Bit 4 - Output sound 1 to SO2 terminal
    /// Bit 3 - Output sound 4 to SO1 terminal
    /// Bit 2 - Output sound 3 to SO1 terminal
    /// Bit 1 - Output sound 2 to SO1 terminal
    /// Bit 0 - Output sound 1 to SO1 terminal
    channel_pan: u8,

    /// Sound on/off
    /// NR52 ($FF26)
    /// Bit 7 - All sound on/off  (0: stop all sound circuits) (Read/Write)
    all_sound_enable: bool,
    /// Bit 3 - Sound 4 ON flag (Read Only)
    channel4_on: bool,
    /// Bit 2 - Sound 3 ON flag (Read Only)
    channel3_on: bool,
    /// Bit 1 - Sound 2 ON flag (Read Only)
    channel2_on: bool,
    /// Bit 0 - Sound 1 ON flag (Read Only)
    channel1_on: bool,

    /// The host sample rate to convert the generated waveforms into,
    /// provided at emulator power-on.
    sample_rate: u32,

    /// The current cycle count in CPU cycles at 4.19 MHz
    /// Used to step the frame sequencer and determine 
    /// sound sample generation
    /// Wraps every 8192 cycles back to zero, aligning with a full set
    /// of frame sequencer clocks.
    cycle_count: usize,

    /// The final stereo output buffer at the host sample rate, after
    /// all mixing. A thread-safe Vec buffer of f32 samples, filled
    /// as the emulator generates samples. If the buffer is full,
    /// the APU will skip the samples until there's room.
    out_buffer: AudioBuffer,
}

impl Apu {
    pub fn power_on(sample_rate: u32) -> (Self, AudioBuffer) {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let ret = buf.clone();
        (
            Apu {
                square1: SquareChannel1::power_on(sample_rate),
                so2_vin_enable: false,
                so2_volume: 0x7,
                so1_vin_enable: false,
                so1_volume: 0x7,
                channel_pan: 0xF3,
                all_sound_enable: true,
                channel4_on: false,
                channel3_on: false,
                channel2_on: false,
                channel1_on: true,
                sample_rate,
                cycle_count: 0,
                out_buffer: buf,
            },
            ret,
        )
    }

    pub fn update(&mut self, cycles: usize) {
        if self.all_sound_enable {
            self.cycle_count += cycles;
            let mut frame_cycles = 0;
            while self.cycle_count >= 8192 {
                // Increment the number of frame sequencer clocks
                self.cycle_count -= 8192;
                frame_cycles += 1;
            }
            self.square1.update(cycles, frame_cycles);
        }
    }
}

impl Memory for Apu {
    fn read_byte(&self, addr: u16) -> u8 {
        assert!((0xFF10..=0xFF3F).contains(&addr));
        match addr {
            0xFF10..=0xFF14 => self.square1.read_byte(addr),
            0xFF24 => {
                let mut v = 0x0;
                v |= (self.so2_vin_enable as u8) << 7;
                v |= self.so2_volume << 4;
                v |= (self.so1_vin_enable as u8) << 3;
                v |= self.so1_volume;
                v
            }
            0xFF25 => self.channel_pan,
            0xFF26 => {
                let mut v = 0x0;
                v |= (self.all_sound_enable as u8) << 7;
                v |= (self.channel4_on as u8) << 3;
                v |= (self.channel3_on as u8) << 2;
                v |= (self.channel2_on as u8) << 1;
                v |= self.channel1_on as u8;
                v
            }
            _ => {
                warn!("Unassigned APU memory location {:04X}", addr);
                0
            }
        }
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        assert!((0xFF10..=0xFF3F).contains(&addr));
        match addr {
            0xFF10..=0xFF14 => self.square1.write_byte(addr, val),
            0xFF24 => {
                self.so2_vin_enable = (val >> 7) & 0x1 != 0;
                self.so2_volume = (val >> 4) & 0x7;
                self.so1_vin_enable = (val >> 3) & 0x1 != 0;
                self.so1_volume = val & 0x7;
            }
            0xFF25 => self.channel_pan = val,
            0xFF26 => self.all_sound_enable = (val >> 7) & 0x1 != 0,
            _ => warn!("Unassigned APU memory location {:04X}", addr),
        }
    }
}
