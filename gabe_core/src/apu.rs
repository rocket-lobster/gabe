#![allow(dead_code)]
#![allow(unused_variables)]

use super::sink::*;
use super::{mmu::Memory, util::bit::*};

// Use SAMPLE_RATE exported from lib to match
const SAMPLE_RATE: u32 = super::SAMPLE_RATE;

// 4.19 MHz / 65.536 KHz
const SAMPLE_RATE_PERIOD: u32 = 64;

// 4.19 MHz / 512 Hz
const FRAME_SEQ_PERIOD: u32 = 8192;

struct SquareChannel1 {
    /// Flag indicating if the internal DAC is enabled
    /// If false, no sound will be emitted, even on channel trigger
    dac_enabled: bool,

    /// Flag indicating if the sound is currently playing
    /// Set to true on a NR14 b7 trigger write, and reported by NR52
    channel_enabled: bool,

    /// CH1 Sweep Control (R/W)
    /// NR10 ($FF10)
    /// Bit 6-4 - Sweep pace
    /// Bit 3   - Sweep increase/decrease
    ///     0: Addition    (wavelength increases)
    ///     1: Subtraction (wavelength decreases)
    /// Bit 2-0 - Sweep slope control (n: 0-7)
    nr10_sweep_control: u8,

    /// CH1 Legnth Control (R/W)
    /// NR11 ($FF11)
    /// Bit 7-6 - Wave Pattern Duty (Read/Write)
    /// Bit 5-0 - Sound length data (Write Only) (t1: 0-63)
    /// Sound Length = (64-t1)*(1/256) seconds.
    /// The Length value is used only if Bit 6 in NR14 is set.
    nr11_length_data: u8,

    /// CH1 Volume Control (R/W)
    /// NR12 ($FF12)
    /// Bit 7-4 - Initial Volume of envelope (0-0Fh) (0=No Sound)
    /// Bit 3   - Envelope Direction (0=Decrease, 1=Increase)
    /// Bit 2-0 - Number of envelope sweep (n: 0-7)
    /// (If zero, stop envelope operation.)
    /// Length of 1 step = n*(1/64) seconds
    nr12_volume_control: u8,

    /// NR13 CH1 Wavelength Low (W)
    /// Lower 8-bits of frequency (wavelength) data
    /// Frequency = 131072/(2048-x) Hz
    nr13_frequency_low: u8,

    /// NR14 CH1 Wavelength High / Control (W)
    /// Bit 7   - Trigger (1=Restart channel)  (Write Only)
    /// Bit 6   - Sound Length enable          (Read/Write)
    ///           (1=Stop output when length in NR11 expires)
    /// Bit 2-0 - "Wavelength"'s higher 3 bits (Write Only)
    nr14_freq_high_control: u8,

    /// The period of the frequency timer for waveform generation.
    /// Calculated every time the frequency is changed with the formula:
    ///     Period = 4 * (2048 - frequency)
    frequency_timer: u32,

    /// The number of sweep steps needed before calculating the next frequency in the sweep.
    /// Loaded from NR10 [6:4] on reaching zero or on a channel trigger.
    sweep_timer: u8,

    /// Internal flag set on a channel trigger. Set if the sweep pace or slope are
    /// non-zero, otherwise cleared.
    sweep_enabled: bool,

    /// Internal frequency register that holds the frequencies currently being sweeped.
    /// Set on channel trigger and updated each time the sweep is updated
    sweep_shadow: u32,

    /// The volume of the channel, modified by the volume envelope if necessary
    /// Starts at NR12 [7:4] when channel is triggered
    current_volume: u8,

    /// The state of the volume envelope, loaded from NR12 [3] on channel trigger
    volume_increasing: bool,

    /// The number of envelope steps needed before changing volume
    /// Loaded from `envelope_period` when reaching 0 or on channel trigger
    envelope_timer: u8,

    /// The value reloaded into the envelope timer when it reaches zero
    /// Loaded from NR12 [2:0] on channel trigger
    envelope_period: u8,

    /// Loaded on channel trigger from NR11 [5:0], subtracted from 64
    /// If length is enabled, once period is reached, channel is disabled
    length_timer: u8,

    /// The current location in the wave pattern given by wave_pattern
    wave_index: usize,
}

impl SquareChannel1 {
    fn step_freq(&mut self) {
        // Check if the buffer needs to be updated with new samples to match the frequency
        if self.frequency_timer == 0 {
            // Move wave duty to next index slot
            self.wave_index = (self.wave_index + 1) % 8;

            // Reset Frequency period to match current frequency value
            self.frequency_timer = (2048
                - (((self.nr14_freq_high_control as u32 & 0b111) << 8)
                    | self.nr13_frequency_low as u32))
                * 4;
        }
        self.frequency_timer -= 1;
    }

    fn step_sweep(&mut self) {
        self.sweep_timer = self.sweep_timer.saturating_sub(1);
        if self.sweep_timer == 0
            && self.sweep_enabled
            && extract_bits(self.nr10_sweep_control, 6, 4) != 0x0
        {
            self.sweep_timer = extract_bits(self.nr10_sweep_control, 6, 4);
            // Calculate new freq and check overflow
            let mut freq =
                (self.sweep_shadow >> extract_bits(self.nr10_sweep_control, 2, 0)) as i32;
            if test_bit(self.nr10_sweep_control, 3) {
                freq = -freq;
            }
            freq += self.sweep_shadow as i32;
            if freq > 2047 || freq < 0 {
                self.channel_enabled = false;
            } else if extract_bits(self.nr10_sweep_control, 2, 0) != 0 {
                // Write the new freq into shadow and NR13+NR14
                self.sweep_shadow = freq as u32;
                self.nr13_frequency_low = (self.sweep_shadow & 0xFF) as u8;
                self.nr14_freq_high_control =
                    (self.nr14_freq_high_control & 0xF8) | ((self.sweep_shadow >> 8) & 0x7) as u8;

                // Freq calc and overflow check again
                freq = (self.sweep_shadow >> extract_bits(self.nr10_sweep_control, 2, 0)) as i32;
                if test_bit(self.nr10_sweep_control, 3) {
                    freq = -freq;
                }
                freq += self.sweep_shadow as i32;
                if freq > 2047 || freq < 0 {
                    self.channel_enabled = false;
                }
            }
        }
    }

    fn step_envelope(&mut self) {
        if self.envelope_period != 0 {
            self.envelope_timer -= 1;
            if self.envelope_timer == 0 {
                self.envelope_timer = self.envelope_period;
                if self.current_volume < 0xF && self.volume_increasing {
                    self.current_volume += 1;
                } else if self.current_volume > 0x0 && !self.volume_increasing {
                    self.current_volume -= 1;
                }
            }
        }
    }

    fn step_length(&mut self) {
        if test_bit(self.nr14_freq_high_control, 6) && (self.length_timer > 0) {
            self.length_timer -= 1;

            if self.length_timer == 0 {
                self.channel_enabled = false;
            }
        }
    }

    fn get_amp(&self) -> i16 {
        if self.dac_enabled && self.channel_enabled {
            let pattern = match extract_bits(self.nr11_length_data, 7, 6) {
                0x0 => 0b0000_0001, // 12.5%
                0x1 => 0b1000_0001, // 25%
                0x2 => 0b1000_0111, // 50%
                0x3 => 0b0111_1110, // 75%
                _ => unreachable!(),
            };

            if ((pattern >> self.wave_index) & 0x1) != 0x0 {
                self.current_volume as i16 * (i16::MAX / 0xF)
            } else {
                self.current_volume as i16 * (i16::MIN / 0xF)
            }
        } else {
            0
        }
    }
}

impl Memory for SquareChannel1 {
    fn read_byte(&self, addr: u16) -> u8 {
        assert!((0xFF10..=0xFF14).contains(&addr));
        match addr {
            0xFF10 => self.nr10_sweep_control,
            0xFF11 => self.nr11_length_data | 0x3F,
            0xFF12 => self.nr12_volume_control,
            0xFF13 => 0xFF,
            0xFF14 => self.nr14_freq_high_control | 0xBF,
            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        assert!((0xFF10..=0xFF14).contains(&addr));
        match addr {
            0xFF10 => self.nr10_sweep_control = val,
            0xFF11 => self.nr11_length_data = val,
            0xFF12 => {
                self.nr12_volume_control = val;
                self.dac_enabled = extract_bits(val, 7, 3) != 0x0;
            }
            0xFF13 => self.nr13_frequency_low = val,
            0xFF14 => {
                self.nr14_freq_high_control = val;
                if test_bit(val, 7) {
                    // Channel is triggered, init state
                    self.channel_enabled = true;
                    // Length counter set
                    self.length_timer = 64 - extract_bits(self.nr11_length_data, 5, 0);
                    // Reset frequency period
                    self.frequency_timer = (2048
                        - (((self.nr14_freq_high_control as u32 & 0b111) << 8)
                            | self.nr13_frequency_low as u32))
                        * 4;
                    // Update sweep state
                    self.sweep_shadow = ((self.nr14_freq_high_control as u32 & 0b111) << 8)
                        | self.nr13_frequency_low as u32;
                    self.sweep_timer = extract_bits(self.nr10_sweep_control, 6, 4);
                    if extract_bits(self.nr10_sweep_control, 2, 0) != 0x0 {
                        // Sweep shift is non-zero, set sweep-enable to true
                        self.sweep_enabled = true;
                        // Immediately perform frequency calc and overflow check
                        let mut freq = (self.sweep_shadow
                            >> extract_bits(self.nr10_sweep_control, 2, 0))
                            as i32;
                        if test_bit(self.nr10_sweep_control, 3) {
                            freq = -freq;
                        }
                        freq += self.sweep_shadow as i32;
                        if freq > 2047 {
                            self.channel_enabled = false;
                        }
                    } else if self.sweep_timer != 0 {
                        self.sweep_enabled = true;
                    } else {
                        self.sweep_enabled = false;
                    }
                    // Reload envelope period
                    self.envelope_period = extract_bits(self.nr12_volume_control, 2, 0);
                    self.envelope_timer = self.envelope_period;
                    // Reload current volume
                    self.current_volume = extract_bits(self.nr12_volume_control, 7, 4);
                    // Load envelope direction
                    self.volume_increasing = test_bit(val, 3);
                }
            }
            _ => unreachable!(),
        }
    }
}

struct SquareChannel2 {
    /// Flag indicating if the internal DAC is enabled
    /// If false, no sound will be emitted, even on channel trigger
    dac_enabled: bool,

    /// Flag indicating if the sound is currently playing
    /// Set to true on a NR14 b7 trigger write, and reported by NR52
    channel_enabled: bool,

    /// CH2 Legnth Control (R/W)
    /// NR21 ($FF16)
    /// Bit 7-6 - Wave Pattern Duty (Read/Write)
    /// Bit 5-0 - Sound length data (Write Only) (t1: 0-63)
    /// Sound Length = (64-t1)*(1/256) seconds.
    /// The Length value is used only if Bit 6 in NR14 is set.
    nr21_length_data: u8,

    /// CH2 Volume Control (R/W)
    /// NR22 ($FF17)
    /// Bit 7-4 - Initial Volume of envelope (0-0Fh) (0=No Sound)
    /// Bit 3   - Envelope Direction (0=Decrease, 1=Increase)
    /// Bit 2-0 - Number of envelope sweep (n: 0-7)
    /// (If zero, stop envelope operation.)
    /// Length of 1 step = n*(1/64) seconds
    nr22_volume_control: u8,

    /// NR23 CH1 Wavelength Low (W)
    /// Lower 8-bits of frequency (wavelength) data
    /// Frequency = 131072/(2048-x) Hz
    nr23_frequency_low: u8,

    /// NR24 CH2 Wavelength High / Control (W)
    /// Bit 7   - Trigger (1=Restart channel)  (Write Only)
    /// Bit 6   - Sound Length enable          (Read/Write)
    ///           (1=Stop output when length in NR11 expires)
    /// Bit 2-0 - "Wavelength"'s higher 3 bits (Write Only)
    nr24_freq_high_control: u8,

    /// The period of the frequency timer for waveform generation.
    /// Calculated every time the frequency is changed with the formula:
    ///     Period = 4 * (2048 - frequency)
    frequency_timer: u32,

    /// The volume of the channel, modified by the volume envelope if necessary
    /// Starts at NR12 [7:4] when channel is triggered
    current_volume: u8,

    /// The state of the volume envelope, loaded from NR12 [3] on channel trigger
    volume_increasing: bool,

    /// The number of envelope steps needed before changing volume
    /// Loaded from `envelope_period` when reaching 0 or on channel trigger
    envelope_timer: u8,

    /// The value reloaded into the envelope timer when it reaches zero
    /// Loaded from NR12 [2:0] on channel trigger
    envelope_period: u8,

    /// Loaded on channel trigger from NR11 [5:0], subtracted from 64
    /// If length is enabled, once period is reached, channel is disabled
    length_timer: u8,

    /// The current location in the wave pattern given by wave_pattern
    wave_index: usize,
}

impl SquareChannel2 {
    fn step_freq(&mut self) {
        // Check if the buffer needs to be updated with new samples to match the frequency
        if self.frequency_timer == 0 {
            // Move wave duty to next index slot
            self.wave_index = (self.wave_index + 1) % 8;

            // Reset Frequency period to match current frequency value
            self.frequency_timer = (2048
                - (((self.nr24_freq_high_control as u32 & 0b111) << 8)
                    | self.nr23_frequency_low as u32))
                * 4;
        }
        self.frequency_timer -= 1;
    }

    fn step_envelope(&mut self) {
        if self.envelope_period != 0 {
            self.envelope_timer -= 1;
            if self.envelope_timer == 0 {
                self.envelope_timer = self.envelope_period;
                if self.current_volume < 0xF && self.volume_increasing {
                    self.current_volume += 1;
                } else if self.current_volume > 0x0 && !self.volume_increasing {
                    self.current_volume -= 1;
                }
            }
        }
    }

    fn step_length(&mut self) {
        if test_bit(self.nr24_freq_high_control, 6) && (self.length_timer > 0) {
            self.length_timer -= 1;

            if self.length_timer == 0 {
                self.channel_enabled = false;
            }
        }
    }

    fn get_amp(&self) -> i16 {
        if self.dac_enabled && self.channel_enabled {
            let pattern = match extract_bits(self.nr21_length_data, 7, 6) {
                0x0 => 0b0000_0001, // 12.5%
                0x1 => 0b1000_0001, // 25%
                0x2 => 0b1000_0111, // 50%
                0x3 => 0b0111_1110, // 75%
                _ => unreachable!(),
            };

            if ((pattern >> self.wave_index) & 0x1) != 0x0 {
                self.current_volume as i16 * (i16::MAX / 0xF)
            } else {
                self.current_volume as i16 * (i16::MIN / 0xF)
            }
        } else {
            0
        }
    }
}

impl Memory for SquareChannel2 {
    fn read_byte(&self, addr: u16) -> u8 {
        assert!((0xFF16..=0xFF19).contains(&addr));
        match addr {
            0xFF16 => self.nr21_length_data | 0x3F,
            0xFF17 => self.nr22_volume_control,
            0xFF18 => 0xFF,
            0xFF19 => self.nr24_freq_high_control | 0xBF,
            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        assert!((0xFF16..=0xFF19).contains(&addr));
        match addr {
            0xFF16 => self.nr21_length_data = val,
            0xFF17 => {
                self.nr22_volume_control = val;
                self.dac_enabled = extract_bits(val, 7, 3) != 0x0;
            }
            0xFF18 => self.nr23_frequency_low = val,
            0xFF19 => {
                self.nr24_freq_high_control = val;
                if test_bit(val, 7) {
                    // Channel is triggered, init state
                    self.channel_enabled = true;
                    // Length counter set
                    self.length_timer = 64 - extract_bits(self.nr21_length_data, 5, 0);
                    // Reset frequency period
                    self.frequency_timer = (2048
                        - (((self.nr24_freq_high_control as u32 & 0b111) << 8)
                            | self.nr23_frequency_low as u32))
                        * 4;
                    // Reload envelope period
                    self.envelope_period = extract_bits(self.nr22_volume_control, 2, 0);
                    self.envelope_timer = self.envelope_period;
                    // Reload current volume
                    self.current_volume = extract_bits(self.nr22_volume_control, 7, 4);
                    // Load envelope direction
                    self.volume_increasing = test_bit(val, 3);
                }
            }
            _ => unreachable!(),
        }
    }
}

pub struct Apu {
    // Global Registers
    /// Channel control / ON-OFF / Volume (R/W)
    /// NR50 ($FF24)
    /// Bit 7   - Output Vin to SO2 terminal (1=Enable)
    /// Bit 6-4 - SO2 output level (volume)  (0-7)
    /// Bit 3   - Output Vin to SO1 terminal (1=Enable)
    /// Bit 2-0 - SO1 output level (volume)  (0-7)
    nr50_output_control: u8,

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
    nr51_channel_pan: u8,

    /// Sound on/off
    /// NR52 ($FF26)
    /// Bit 7 - All sound on/off  (0: stop all sound circuits) (Read/Write)
    all_sound_on: bool,

    /// Sound Channel 1 - Tone and Sweep
    /// NR10-NR14 ($FF10-$FF14)
    square1: SquareChannel1,

    /// Sound Channel 2 - Pulse
    /// NR21-NR24 ($FF16 - $FF19)
    square2: SquareChannel2,

    /// The current cycle count in CPU cycles at 4.19 MHz
    /// Used to step the frame sequencer and determine
    /// sound sample generation
    /// Wraps every 8192 cycles back to zero, aligning with a full set
    /// of frame sequencer clocks.
    cycle_count: u32,

    /// The current clock of the Frame Sequencer, values only from 0-7.
    /// Clocked every 8192 cycles, then passed to each channel to update
    /// Length counter, Frequency Sweep, and Volume Envelopes.
    /// Also marks the generation of samples to the host device.
    frame_cycle: u8,
}

impl Apu {
    pub fn power_on() -> Self {
        Apu {
            square1: SquareChannel1 {
                dac_enabled: true,
                channel_enabled: false,
                nr10_sweep_control: 0x80,
                nr11_length_data: 0xBF,
                nr12_volume_control: 0xF3,
                nr13_frequency_low: 0xFF,
                nr14_freq_high_control: 0xBF,
                frequency_timer: 8192,
                wave_index: 0,
                sweep_timer: 0,
                sweep_enabled: false,
                sweep_shadow: 0,
                current_volume: 0,
                volume_increasing: false,
                envelope_timer: 0,
                envelope_period: 0,
                length_timer: 0,
            },
            square2: SquareChannel2 {
                dac_enabled: true,
                channel_enabled: false,
                nr21_length_data: 0x3F,
                nr22_volume_control: 0x00,
                nr23_frequency_low: 0xFF,
                nr24_freq_high_control: 0xBF,
                frequency_timer: 0,
                current_volume: 0,
                volume_increasing: false,
                envelope_timer: 0,
                envelope_period: 0,
                length_timer: 0,
                wave_index: 0,
            },
            nr50_output_control: 0x77,
            nr51_channel_pan: 0xF3,
            all_sound_on: true,
            cycle_count: 0,
            frame_cycle: 0,
        }
    }

    pub fn update(&mut self, cycles: u32, audio_sink: &mut dyn Sink<AudioFrame>) {
        if self.all_sound_on {
            for _ in 0..cycles {
                self.cycle_count += 1;

                self.square1.step_freq();
                self.square2.step_freq();

                if self.cycle_count >= 8192 {
                    // Increment the number of frame sequencer clocks
                    self.cycle_count -= 8192;
                    self.frame_cycle = (self.frame_cycle + 1) % 8;
                    if [0, 2, 4, 6].contains(&self.frame_cycle) {
                        // Update length counter if enabled
                        self.square1.step_length();
                        self.square2.step_length();
                    }
                    if [2, 6].contains(&self.frame_cycle) {
                        // Update Freq Sweep
                        self.square1.step_sweep();
                    }
                    if self.frame_cycle == 7 {
                        // Update volume envelope
                        self.square1.step_envelope();
                        self.square2.step_envelope();
                    }
                }

                if self.cycle_count % SAMPLE_RATE_PERIOD == 0 {
                    // Reached period needed to generate a sample
                    let left_amp = {
                        let mut amp_acc: i32 = 0;
                        let mut acc_count = 0;
                        if test_bit(self.nr51_channel_pan, 4) {
                            amp_acc += self.square1.get_amp() as i32;
                            acc_count += 1;
                        }
                        if test_bit(self.nr51_channel_pan, 5) {
                            amp_acc += self.square2.get_amp() as i32;
                            acc_count += 1;
                        }
                        (amp_acc / acc_count) as i16
                    };
                    let right_amp = {
                        let mut amp_acc: i32 = 0;
                        let mut acc_count = 0;
                        if test_bit(self.nr51_channel_pan, 0) {
                            amp_acc += self.square1.get_amp() as i32;
                            acc_count += 1;
                        }
                        if test_bit(self.nr51_channel_pan, 1) {
                            amp_acc += self.square2.get_amp() as i32;
                            acc_count += 1;
                        }
                        (amp_acc / acc_count) as i16
                    };
                    let left_vol = (extract_bits(self.nr50_output_control, 6, 4) + 1) as f32 / 8.0;
                    let right_vol = (extract_bits(self.nr50_output_control, 2, 0) + 1) as f32 / 8.0;
                    audio_sink.append(((left_amp as f32 * left_vol) as i16, (right_amp as f32 * right_vol) as i16));
                }
            }
        }
    }
}

impl Memory for Apu {
    fn read_byte(&self, addr: u16) -> u8 {
        assert!((0xFF10..=0xFF3F).contains(&addr));
        match addr {
            0xFF10..=0xFF14 => self.square1.read_byte(addr),
            0xFF16..=0xFF19 => self.square2.read_byte(addr),
            0xFF24 => self.nr50_output_control,
            0xFF25 => self.nr51_channel_pan,
            0xFF26 => {
                let mut ret = 0b0111_0000;
                if self.all_sound_on {
                    ret = set_bit(ret, 7);
                }
                if self.square1.channel_enabled {
                    ret = set_bit(ret, 0);
                }
                if self.square2.channel_enabled {
                    ret = set_bit(ret, 1);
                }
                ret
            }
            _ => {
                debug!("Unassigned APU memory location {:04X}", addr);
                0
            }
        }
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        assert!((0xFF10..=0xFF3F).contains(&addr));
        match addr {
            0xFF10..=0xFF14 => self.square1.write_byte(addr, val),
            0xFF16..=0xFF19 => self.square2.write_byte(addr, val),
            0xFF24 => self.nr50_output_control = val,
            0xFF25 => self.nr51_channel_pan = val,
            0xFF26 => self.all_sound_on = val & 0x80 != 0, // Only bit 7 is writable
            _ => debug!("Unassigned APU memory location {:04X}", addr),
        }
    }
}
