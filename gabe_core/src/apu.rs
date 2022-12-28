use super::sink::*;
use super::{mmu::Memory, util::bit::*};

// Use SAMPLE_RATE exported from lib to match
const SAMPLE_RATE: u32 = super::SAMPLE_RATE;

// 4.19 MHz / 65.536 KHz
const SAMPLE_RATE_PERIOD: u32 = super::CLOCK_RATE / SAMPLE_RATE;

// 4.19 MHz / 512 Hz
const FRAME_SEQ_PERIOD: u32 = 8192;

#[derive(Default)]
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

    /// Flag indicating if the length_timer gets an extra clock when being set
    /// Happens on first-half of the frame sequencer period for length clocks
    extra_length: bool,
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
        if self.sweep_timer == 0 && self.sweep_enabled {
            self.sweep_timer = extract_bits(self.nr10_sweep_control, 6, 4);
            if self.sweep_timer == 0 {
                // Treat period of 0 as 8
                self.sweep_timer = 8;
            }
            // Calculate new freq and check overflow
            let mut freq =
                (self.sweep_shadow >> extract_bits(self.nr10_sweep_control, 2, 0)) as i32;
            if test_bit(self.nr10_sweep_control, 3) {
                freq = -freq;
            }
            freq += self.sweep_shadow as i32;
            if !(0..=2047).contains(&freq) {
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
                if !(0..=2047).contains(&freq) {
                    self.channel_enabled = false;
                }
            }
        }
        self.sweep_timer = self.sweep_timer.saturating_sub(1);
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
        if test_bit(self.nr14_freq_high_control, 6)
            && (self.length_timer > 0)
            && self.channel_enabled
        {
            self.length_timer -= 1;

            if self.length_timer == 0 {
                self.channel_enabled = false;
            }
        }
        self.extra_length = true;
    }

    fn get_amp(&self) -> f32 {
        if self.dac_enabled && self.channel_enabled {
            let pattern = match extract_bits(self.nr11_length_data, 7, 6) {
                0x0 => 0b0000_0001, // 12.5%
                0x1 => 0b1000_0001, // 25%
                0x2 => 0b1000_0111, // 50%
                0x3 => 0b0111_1110, // 75%
                _ => unreachable!(),
            };
            convert_u4_to_f32_sample(((pattern >> self.wave_index) & 0x1) * 0xF)
                * (self.current_volume as f32 / 15.0)
        } else {
            0.0
        }
    }
}

impl Memory for SquareChannel1 {
    fn read_byte(&self, addr: u16) -> u8 {
        assert!((0xFF10..=0xFF14).contains(&addr));
        match addr {
            0xFF10 => self.nr10_sweep_control | 0x80,
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
            0xFF11 => {
                self.nr11_length_data = val;
                self.length_timer = 64 - extract_bits(self.nr11_length_data, 5, 0);
            }
            0xFF12 => {
                self.nr12_volume_control = val;
                self.dac_enabled = extract_bits(val, 7, 3) != 0x0;
                if !self.dac_enabled {
                    self.channel_enabled = false;
                }
            }
            0xFF13 => self.nr13_frequency_low = val,
            0xFF14 => {
                let was_enabled_length = test_bit(self.nr14_freq_high_control, 6);
                self.nr14_freq_high_control = val;
                if self.extra_length
                    && !was_enabled_length
                    && test_bit(val, 6)
                    && self.length_timer != 0
                {
                    // Extra clock should occur when:
                    // Next frame clock will not be a length clock
                    // Length counter was disabled, and is now being enabled
                    // Length timer is not already zero
                    self.length_timer -= 1;
                    if self.length_timer == 0 {
                        self.channel_enabled = false;
                    }
                }
                if test_bit(val, 7) {
                    // Channel is triggered, init state
                    self.channel_enabled = true;
                    // Length counter set
                    if self.length_timer == 0 {
                        self.length_timer = 64;
                        if self.extra_length && test_bit(self.nr14_freq_high_control, 6) {
                            self.length_timer -= 1;
                        }
                    }
                    // Reset frequency period
                    self.frequency_timer = (2048
                        - (((self.nr14_freq_high_control as u32 & 0b111) << 8)
                            | self.nr13_frequency_low as u32))
                        * 4;

                    self.sweep_timer = extract_bits(self.nr10_sweep_control, 6, 4);
                    if self.sweep_timer == 0 {
                        // Treat period of 0 as 8
                        self.sweep_timer = 8;
                    }
                    if extract_bits(self.nr10_sweep_control, 2, 0) != 0x0 {
                        // Update sweep state
                        self.sweep_shadow = ((self.nr14_freq_high_control as u32 & 0b111) << 8)
                            | self.nr13_frequency_low as u32;
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

                    if !self.dac_enabled {
                        self.channel_enabled = false;
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Default)]
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

    /// Flag indicating if the length_timer gets an extra clock when being set
    /// Happens on first-half of the frame sequencer period for length clocks
    extra_length: bool,
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
        self.extra_length = true;
    }

    fn get_amp(&self) -> f32 {
        if self.dac_enabled && self.channel_enabled {
            let pattern = match extract_bits(self.nr21_length_data, 7, 6) {
                0x0 => 0b0000_0001, // 12.5%
                0x1 => 0b1000_0001, // 25%
                0x2 => 0b1000_0111, // 50%
                0x3 => 0b0111_1110, // 75%
                _ => unreachable!(),
            };

            convert_u4_to_f32_sample(((pattern >> self.wave_index) & 0x1) * 0xF)
                * (self.current_volume as f32 / 15.0)
        } else {
            0.0
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
            0xFF16 => {
                self.nr21_length_data = val;
                self.length_timer = 64 - extract_bits(self.nr21_length_data, 5, 0);
            }
            0xFF17 => {
                self.nr22_volume_control = val;
                self.dac_enabled = extract_bits(val, 7, 3) != 0x0;
                if !self.dac_enabled {
                    self.channel_enabled = false;
                }
            }
            0xFF18 => self.nr23_frequency_low = val,
            0xFF19 => {
                let was_enabled_length = test_bit(self.nr24_freq_high_control, 6);
                self.nr24_freq_high_control = val;
                if self.extra_length
                    && !was_enabled_length
                    && test_bit(val, 6)
                    && self.length_timer != 0
                {
                    // Extra clock should occur when:
                    // Next frame clock will not be a length clock
                    // Length counter was disabled, and is now being enabled
                    // Length timer is not already zero
                    self.length_timer -= 1;
                    if self.length_timer == 0 {
                        self.channel_enabled = false;
                    }
                }
                if test_bit(val, 7) {
                    // Channel is triggered, init state
                    self.channel_enabled = true;
                    // Length counter set
                    if self.length_timer == 0 {
                        self.length_timer = 64;
                        if self.extra_length && test_bit(self.nr24_freq_high_control, 6) {
                            self.length_timer -= 1;
                        }
                    }
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

                    if !self.dac_enabled {
                        self.channel_enabled = false;
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Default)]
struct WaveChannel {
    /// Flag indicating if the sound is currently playing
    /// Set to true on a NR34 b7 trigger write, and reported by NR52
    channel_enabled: bool,

    /// CH3 DAC Enable
    /// NR30 ($FF1A)
    /// Bit 7 - Sound Channel 3 DAC  (0=Off, 1=On)
    nr30_dac_enable: u8,

    /// CH3 Length Timer (W)
    /// NR31 ($FF1B)
    /// Bit 7-0 - length timer
    nr31_length_timer: u8,

    /// CH3 Output Level (R/W)
    /// NR32 ($FF1C)
    /// Bits 6-5 - Output level selection
    ///     00 - Mute (No sound)
    ///     01 - 100% volume (use samples read from Wave RAM as-is)
    ///     10 - 50% volume (shift samples read from Wave RAM right once)
    ///     11 - 25% volume (shift samples read from Wave RAM right twice)
    nr32_output_level: u8,

    /// NR33 CH3 Wavelength Low (W)
    /// Lower 8-bits of frequency (wavelength) data
    nr33_frequency_low: u8,

    /// NR34 CH3 Wavelength High / Control (W)
    /// Bit 7   - Trigger (1=Restart channel)  (Write Only)
    /// Bit 6   - Sound Length enable          (Read/Write)
    ///           (1=Stop output when length in NR11 expires)
    /// Bit 2-0 - "Wavelength"'s higher 3 bits (Write Only)
    nr34_freq_high_control: u8,

    /// The period of the frequency timer for waveform generation.
    /// Calculated every time the frequency is changed with the formula:
    ///     Period = 2 * (2048 - frequency)
    frequency_timer: u32,

    /// Loaded on channel trigger from NR31 [7:0], subtracted from 256
    /// If length is enabled, once period is reached, channel is disabled
    length_timer: u16,

    /// The current 4-bit sample to output to the DAC. Loaded from RAM
    /// every frequency timer clock.
    sample_buffer: u8,

    /// The ram containing a sample waveform of 32 4-bit samples
    /// Indexed into by
    wave_ram: [u8; 16],

    /// The current location in the wave pattern given by wave_pattern
    wave_index: usize,

    /// Flag indicating if the length_timer gets an extra clock when being set
    /// Happens on first-half of the frame sequencer period for length clocks
    extra_length: bool,
}

impl WaveChannel {
    fn step_freq(&mut self) {
        // Check if the buffer needs to be updated with new samples to match the frequency
        if self.frequency_timer == 0 {
            // Move wave duty to next index slot
            self.wave_index = (self.wave_index + 1) % 32;

            self.sample_buffer = {
                let entry = self.wave_ram[self.wave_index / 2];
                if self.wave_index & 0x1 == 0 {
                    (entry >> 4) & 0xF
                } else {
                    entry & 0xF
                }
            };

            // Reset Frequency period to match current frequency value
            self.frequency_timer = (2048
                - (((self.nr34_freq_high_control as u32 & 0b111) << 8)
                    | self.nr33_frequency_low as u32))
                * 2;
        }
        self.frequency_timer -= 1;
    }

    fn step_length(&mut self) {
        if test_bit(self.nr34_freq_high_control, 6) && (self.length_timer > 0) {
            self.length_timer -= 1;

            if self.length_timer == 0 {
                self.channel_enabled = false;
            }
        }
        self.extra_length = true;
    }

    fn get_amp(&self) -> f32 {
        if test_bit(self.nr30_dac_enable, 7) {
            let vol_shift = match extract_bits(self.nr32_output_level, 6, 5) {
                0b00 => 4,
                0b01 => 0,
                0b10 => 1,
                0b11 => 2,
                _ => unreachable!(),
            };
            convert_u4_to_f32_sample(self.sample_buffer >> vol_shift)
        } else {
            0.0
        }
    }
}

impl Memory for WaveChannel {
    fn read_byte(&self, addr: u16) -> u8 {
        assert!((0xFF1A..=0xFF1E).contains(&addr) || (0xFF30..=0xFF3F).contains(&addr));
        match addr {
            0xFF1A => self.nr30_dac_enable | 0x7F,
            0xFF1B => 0xFF,
            0xFF1C => self.nr32_output_level | 0x9F,
            0xFF1D => 0xFF,
            0xFF1E => self.nr34_freq_high_control | 0xBF,
            0xFF30..=0xFF3F => self.wave_ram[(addr - 0xFF30) as usize],
            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        assert!((0xFF1A..=0xFF1E).contains(&addr) || (0xFF30..=0xFF3F).contains(&addr));
        match addr {
            0xFF1A => {
                self.nr30_dac_enable = val;
                if !test_bit(val, 7) {
                    self.channel_enabled = false;
                }
            }
            0xFF1B => {
                self.nr31_length_timer = val;
                self.length_timer = 256 - self.nr31_length_timer as u16;
            }
            0xFF1C => self.nr32_output_level = val,
            0xFF1D => self.nr33_frequency_low = val,
            0xFF1E => {
                let was_enabled_length = test_bit(self.nr34_freq_high_control, 6);
                self.nr34_freq_high_control = val;
                if self.extra_length
                    && !was_enabled_length
                    && test_bit(val, 6)
                    && self.length_timer != 0
                {
                    // Extra clock should occur when:
                    // Next frame clock will not be a length clock
                    // Length counter was disabled, and is now being enabled
                    // Length timer is not already zero
                    self.length_timer -= 1;
                    if self.length_timer == 0 {
                        self.channel_enabled = false;
                    }
                }
                if test_bit(val, 7) {
                    self.channel_enabled = true;
                    // Length counter set
                    if self.length_timer == 0 {
                        self.length_timer = 256;
                        if self.extra_length && test_bit(self.nr34_freq_high_control, 6) {
                            self.length_timer -= 1;
                        }
                    }
                    // Reset frequency period
                    self.frequency_timer = (2048
                        - (((self.nr34_freq_high_control as u32 & 0b111) << 8)
                            | self.nr33_frequency_low as u32))
                        * 2;
                    // Reset wave index
                    self.wave_index = 0;

                    if !test_bit(self.nr30_dac_enable, 7) {
                        self.channel_enabled = false;
                    }
                }
            }
            0xFF30..=0xFF3F => self.wave_ram[(addr - 0xFF30) as usize] = val,
            _ => unreachable!(),
        }
    }
}

#[derive(Default)]
struct NoiseChannel {
    /// Flag indicating if the sound is currently playing
    /// Set to true on a NR34 b7 trigger write, and reported by NR52
    channel_enabled: bool,

    /// Flag indicating if the DAC is generating samples from the
    /// waveform generator
    dac_enabled: bool,

    /// CH4 Length Timer (W)
    /// NR41 ($FF20)
    /// Bit 5-0 - length timer
    nr41_length_timer: u8,

    /// CH4 Volume Control (R/W)
    /// NR42 ($FF21)
    /// Bit 7-4 - Initial Volume of envelope (0-0Fh) (0=No Sound)
    /// Bit 3   - Envelope Direction (0=Decrease, 1=Increase)
    /// Bit 2-0 - Number of envelope sweep (n: 0-7)
    /// (If zero, stop envelope operation.)
    nr42_volume_control: u8,

    /// NR43 CH4 Freq and RNG
    /// Bit 7-4 - Clock shift (s)
    /// Bit 3   - LFSR width (0=15 bits, 1=7 bits)
    /// Bit 2-0 - Clock divider (r)
    nr43_freq_rng: u8,

    /// NR44 CH4 Wavelength High / Control (W)
    /// Bit 7   - Trigger (1=Restart channel)  (Write Only)
    /// Bit 6   - Sound Length enable          (Read/Write)
    ///           (1=Stop output when length in NR41 expires)
    nr44_channel_control: u8,

    /// The period of the frequency timer for noise generation.
    frequency_timer: u32,

    /// Loaded on channel trigger from NR41 [5:0], subtracted from 256
    /// If length is enabled, once period is reached, channel is disabled
    length_timer: u16,

    /// The volume of the channel, modified by the volume envelope if necessary
    /// Starts at NR42 [7:4] when channel is triggered
    current_volume: u8,

    /// The state of the volume envelope, loaded from NR42 [3] on channel trigger
    volume_increasing: bool,

    /// The number of envelope steps needed before changing volume
    /// Loaded from `envelope_period` when reaching 0 or on channel trigger
    envelope_timer: u8,

    /// The value reloaded into the envelope timer when it reaches zero
    /// Loaded from NR42 [2:0] on channel trigger
    envelope_period: u8,

    /// The state of the Linear Feedback Shift Register (LFSR) that
    /// generates pseudo-random values for the noise generation
    lfsr: u16,

    /// The divisor value used when reloading the frequency period
    divisor: u8,

    /// Flag indicating if the length_timer gets an extra clock when being set
    /// Happens on first-half of the frame sequencer period for length clocks
    extra_length: bool,
}

impl NoiseChannel {
    fn step_freq(&mut self) {
        // Check if the buffer needs to be updated with new samples to match the frequency
        if self.frequency_timer == 0 {
            let output = !(self.lfsr & 0x1) ^ ((self.lfsr >> 1) & 0x1);
            self.lfsr |= output << 15;
            if test_bit(self.nr43_freq_rng, 3) {
                self.lfsr |= output << 7;
            }
            self.lfsr >>= 1;
            self.frequency_timer = (self.divisor as u32) << extract_bits(self.nr43_freq_rng, 7, 4);
        }
        self.frequency_timer = self.frequency_timer.saturating_sub(1);
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
        if test_bit(self.nr44_channel_control, 6) && (self.length_timer > 0) {
            self.length_timer -= 1;

            if self.length_timer == 0 {
                self.channel_enabled = false;
            }
        }
        self.extra_length = true;
    }

    fn get_amp(&self) -> f32 {
        if self.dac_enabled && self.channel_enabled {
            convert_u4_to_f32_sample((!self.lfsr & 0x1) as u8 * 0xF)
                * (self.current_volume as f32 / 15.0)
        } else {
            0.0
        }
    }
}

impl Memory for NoiseChannel {
    fn read_byte(&self, addr: u16) -> u8 {
        assert!((0xFF20..=0xFF23).contains(&addr) || (0xFF30..=0xFF3F).contains(&addr));
        match addr {
            0xFF20 => 0xFF,
            0xFF21 => self.nr42_volume_control,
            0xFF22 => self.nr43_freq_rng,
            0xFF23 => self.nr44_channel_control | 0xBF,
            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        assert!((0xFF20..=0xFF23).contains(&addr) || (0xFF30..=0xFF3F).contains(&addr));
        match addr {
            0xFF20 => {
                self.nr41_length_timer = val & 0x3F;
                self.length_timer = 64 - self.nr41_length_timer as u16;
            }
            0xFF21 => {
                self.nr42_volume_control = val;
                self.dac_enabled = extract_bits(val, 7, 3) != 0x0;
                if !self.dac_enabled {
                    self.channel_enabled = false;
                }
            }
            0xFF22 => {
                self.nr43_freq_rng = val;
                self.divisor = if extract_bits(self.nr43_freq_rng, 2, 0) == 0 {
                    8
                } else {
                    extract_bits(self.nr43_freq_rng, 2, 0) << 4
                };
            }
            0xFF23 => {
                let was_enabled_length = test_bit(self.nr44_channel_control, 6);
                self.nr44_channel_control = val;
                if self.extra_length
                    && !was_enabled_length
                    && test_bit(val, 6)
                    && self.length_timer != 0
                {
                    // Extra clock should occur when:
                    // Next frame clock will not be a length clock
                    // Length counter was disabled, and is now being enabled
                    // Length timer is not already zero
                    self.length_timer -= 1;
                    if self.length_timer == 0 {
                        self.channel_enabled = false;
                    }
                }
                if test_bit(val, 7) {
                    // Channel is triggered, init state
                    self.channel_enabled = true;
                    // Length counter set
                    if self.length_timer == 0 {
                        self.length_timer = 64;
                        if self.extra_length && test_bit(self.nr44_channel_control, 6) {
                            self.length_timer -= 1;
                        }
                    }
                    // Reset frequency period
                    self.frequency_timer =
                        (self.divisor as u32) << extract_bits(self.nr43_freq_rng, 7, 4);
                    // Reload envelope period
                    self.envelope_period = extract_bits(self.nr42_volume_control, 2, 0);
                    self.envelope_timer = self.envelope_period;
                    // Reload current volume
                    self.current_volume = extract_bits(self.nr42_volume_control, 7, 4);
                    // Load envelope direction
                    self.volume_increasing = test_bit(val, 3);
                    // Reset LFSR bits
                    self.lfsr = 0;

                    if !self.dac_enabled {
                        self.channel_enabled = false;
                    }
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

    /// Sound Channel 3 - Wave
    /// NR30-NR34, Wave RAM ($FF1A - $FF1E, $FF30 - $FF3F)
    wave: WaveChannel,

    /// Sound Channel 4 - Noise
    /// NR41-NR44 ($FF20 - $FF23)
    noise: NoiseChannel,

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

    /// When any DAC is enabled, a high-pass filter capacitor is slowly applied
    /// to each of the two analog signals.
    _hpf_capacitor_l: f32,
    _hpf_capacitor_r: f32,
}

impl Apu {
    pub fn power_on() -> Self {
        Apu {
            nr50_output_control: 0x77,
            nr51_channel_pan: 0xF3,
            all_sound_on: true,
            square1: SquareChannel1 {
                dac_enabled: true,
                channel_enabled: false,
                nr10_sweep_control: 0x80,
                nr11_length_data: 0x00,
                nr12_volume_control: 0x00,
                nr13_frequency_low: 0x00,
                nr14_freq_high_control: 0x00,
                frequency_timer: 0,
                wave_index: 0,
                sweep_timer: 0,
                sweep_enabled: false,
                sweep_shadow: 0,
                current_volume: 0,
                volume_increasing: false,
                envelope_timer: 0,
                envelope_period: 0,
                length_timer: 0,
                extra_length: false,
            },
            square2: SquareChannel2 {
                dac_enabled: true,
                channel_enabled: false,
                nr21_length_data: 0x00,
                nr22_volume_control: 0x00,
                nr23_frequency_low: 0x00,
                nr24_freq_high_control: 0x00,
                frequency_timer: 0,
                current_volume: 0,
                volume_increasing: false,
                envelope_timer: 0,
                envelope_period: 0,
                length_timer: 0,
                wave_index: 0,
                extra_length: false,
            },
            wave: WaveChannel {
                channel_enabled: false,
                nr30_dac_enable: 0x00,
                nr31_length_timer: 0x00,
                nr32_output_level: 0x00,
                nr33_frequency_low: 0x00,
                nr34_freq_high_control: 0x00,
                frequency_timer: 0,
                length_timer: 0,
                sample_buffer: 0,
                wave_ram: [0; 16],
                wave_index: 0,
                extra_length: false,
            },
            noise: NoiseChannel {
                channel_enabled: false,
                dac_enabled: true,
                nr41_length_timer: 0x00,
                nr42_volume_control: 0x00,
                nr43_freq_rng: 0x00,
                nr44_channel_control: 0x00,
                frequency_timer: 0,
                length_timer: 0,
                current_volume: 0,
                volume_increasing: false,
                envelope_timer: 0,
                envelope_period: 0,
                lfsr: 0x0,
                divisor: 8,
                extra_length: false,
            },
            cycle_count: 0,
            frame_cycle: 0,
            _hpf_capacitor_l: 0.0,
            _hpf_capacitor_r: 0.0,
        }
    }

    pub fn update(&mut self, cycles: u32, audio_sink: &mut dyn Sink<AudioFrame>) {
        if self.all_sound_on {
            for _ in 0..cycles {
                self.cycle_count += 1;

                self.square1.step_freq();
                self.square2.step_freq();
                self.wave.step_freq();
                self.noise.step_freq();

                if self.cycle_count >= FRAME_SEQ_PERIOD {
                    // Increment the number of frame sequencer clocks
                    self.cycle_count -= FRAME_SEQ_PERIOD;
                    self.frame_cycle = (self.frame_cycle + 1) % 8;
                    if [0, 2, 4, 6].contains(&self.frame_cycle) {
                        // Update length counter if enabled
                        self.square1.step_length();
                        self.square2.step_length();
                        self.wave.step_length();
                        self.noise.step_length();
                    }
                    if [2, 6].contains(&self.frame_cycle) {
                        // Update Freq Sweep
                        self.square1.step_sweep();
                    }
                    if self.frame_cycle == 7 {
                        // Update volume envelope
                        self.square1.step_envelope();
                        self.square2.step_envelope();
                        self.noise.step_envelope();
                    }
                    if [1, 3, 5, 7].contains(&self.frame_cycle) {
                        self.square1.extra_length = false;
                        self.square2.extra_length = false;
                        self.wave.extra_length = false;
                        self.noise.extra_length = false;
                    }
                }

                if self.cycle_count % SAMPLE_RATE_PERIOD == 0 {
                    // Reached period needed to generate a sample
                    let left_amp = {
                        let mut amp_acc: f32 = 0.0;
                        if test_bit(self.nr51_channel_pan, 4) {
                            amp_acc += self.square1.get_amp();
                        }
                        if test_bit(self.nr51_channel_pan, 5) {
                            amp_acc += self.square2.get_amp();
                        }
                        if test_bit(self.nr51_channel_pan, 6) {
                            amp_acc += self.wave.get_amp();
                        }
                        if test_bit(self.nr51_channel_pan, 7) {
                            amp_acc += self.noise.get_amp();
                        }
                        amp_acc / 4.0
                    };
                    let right_amp = {
                        let mut amp_acc: f32 = 0.0;
                        if test_bit(self.nr51_channel_pan, 0) {
                            amp_acc += self.square1.get_amp();
                        }
                        if test_bit(self.nr51_channel_pan, 1) {
                            amp_acc += self.square2.get_amp();
                        }
                        if test_bit(self.nr51_channel_pan, 2) {
                            amp_acc += self.wave.get_amp();
                        }
                        if test_bit(self.nr51_channel_pan, 4) {
                            amp_acc += self.noise.get_amp();
                        }
                        amp_acc / 4.0
                    };
                    let left_vol =
                        (extract_bits(self.nr50_output_control, 6, 4) as f32 + 1.0) / 8.0;
                    let right_vol =
                        (extract_bits(self.nr50_output_control, 2, 0) as f32 + 1.0) / 8.0;
                    let left_output = left_amp * left_vol;
                    let right_output = right_amp * right_vol;
                    audio_sink.append(((left_output), (right_output)));
                }
            }
        }
    }

    // TODO: no_std prevents the powf function, rework without math
    // fn high_pass_filter(&mut self, in_sample: f32, capacitor: f32) -> (f32, f32) {
    //     let mut out_sample = 0.0;
    //     let mut out_cap = 0.0;
    //     let charge_factor = 0.999958f32.powf(SAMPLE_RATE_PERIOD as f32);
    //     if self.square1.dac_enabled
    //         || self.square2.dac_enabled
    //         || test_bit(self.wave.nr30_dac_enable, 7)
    //     {
    //         out_sample = in_sample - capacitor;
    //         out_cap = in_sample - out_sample * charge_factor;
    //     }
    //     (out_sample, out_cap)
    // }
}

impl Memory for Apu {
    fn read_byte(&self, addr: u16) -> u8 {
        assert!((0xFF10..=0xFF3F).contains(&addr));
        match addr {
            0xFF10..=0xFF14 => self.square1.read_byte(addr),
            0xFF16..=0xFF19 => self.square2.read_byte(addr),
            0xFF1A..=0xFF1E => self.wave.read_byte(addr),
            0xFF20..=0xFF23 => self.noise.read_byte(addr),
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
                if self.wave.channel_enabled {
                    ret = set_bit(ret, 2);
                }
                if self.noise.channel_enabled {
                    ret = set_bit(ret, 3);
                }
                ret
            }
            0xFF30..=0xFF3F => self.wave.read_byte(addr),
            _ => {
                debug!("Unassigned APU memory location {:04X}", addr);
                0xFF
            }
        }
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        assert!((0xFF10..=0xFF3F).contains(&addr));
        if self.all_sound_on {
            match addr {
                0xFF10..=0xFF14 => self.square1.write_byte(addr, val),
                0xFF16..=0xFF19 => self.square2.write_byte(addr, val),
                0xFF1A..=0xFF1E => self.wave.write_byte(addr, val),
                0xFF20..=0xFF23 => self.noise.write_byte(addr, val),
                0xFF24 => self.nr50_output_control = val,
                0xFF25 => self.nr51_channel_pan = val,
                0xFF26 => {
                    self.all_sound_on = val & 0x80 != 0; // Only bit 7 is writable
                    if !self.all_sound_on {
                        // APU disabled, clear all registers
                        self.nr50_output_control = 0;
                        self.nr51_channel_pan = 0;
                        self.square1 = SquareChannel1::default();
                        self.square2 = SquareChannel2::default();
                        // Copy over wave ram, shouldn't be affected by APU power
                        let new_wave = WaveChannel {
                            wave_ram: self.wave.wave_ram,
                            ..Default::default()
                        };
                        self.wave = new_wave;
                        self.noise = NoiseChannel::default();
                    }
                }
                0xFF30..=0xFF3F => self.wave.write_byte(addr, val),
                _ => debug!("Unassigned APU memory location {:04X}", addr),
            }
        } else {
            // Most writes are ignored while APU is powered off
            match addr {
                0xFF26 => {
                    self.all_sound_on = val & 0x80 != 0; // Only bit 7 is writable
                }
                0xFF30..=0xFF3F => self.wave.write_byte(addr, val),
                _ => debug!("Writing to APU while powered off {:04X}", addr),
            }
        }
    }
}

/// The channel DACs convert 4-bit unsigned digital signals to -1.0 to 1.0 analog signals.
fn convert_u4_to_f32_sample(sample: u8) -> f32 {
    // Mask off upper nibble to make sure it's 4-bit
    let sample = sample & 0xF;

    (sample as f32 / 7.5) - 1.0
}
