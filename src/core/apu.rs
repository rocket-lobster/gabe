use blip_buf::BlipBuf;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use super::{mmu::Memory, util::bit::extract_bits};

const BUFFER_SIZE: usize = 2048;

/// Helper struct to work with BlipBuf objects and manage the data needed to
/// use the API, such as calculating the delta, tracking the current buffer
/// clocks, and ending frames.
struct SampleBuffer {
    /// A BlipBuf object that takes input clocks and amplitude deltas of the channel
    /// and generates samples at the host sample rate
    buffer: BlipBuf,

    /// The last clock value provided that was associated with a change in amplitude
    /// Reduced by 8192 whenever an audio frame is generated.
    previous_clock: u32,

    /// The last amplitude value provided. Used to calculate the amplitude delta
    /// for subsequent samples.
    previous_ampl: i32,
}

impl SampleBuffer {
    /// Create a new SampleBuffer that will generate samples at the given sample rate.
    fn new(sample_rate: u32) -> Self {
        // Create buffer with enough samples for 1/10 second
        let mut buffer = BlipBuf::new(sample_rate / 10);

        // 4.19 MHz is the system clock rate to convert samples from
        buffer.set_rates(4_194_304f64, f64::from(sample_rate));

        SampleBuffer {
            buffer,
            previous_clock: 0,
            previous_ampl: 0,
        }
    }

    /// Add a new sample by providing the amplitude as an i32 value, and how many clocks
    /// after the previously added sample.
    fn add_sample(&mut self, clock_offset: u32, sample: i32) {
        self.buffer.add_delta(
            self.previous_clock + clock_offset,
            -(self.previous_ampl - sample),
        );
        self.previous_clock += clock_offset;
        self.previous_ampl = sample;
    }

    /// Marks the end of the current frame of sample data to be generated.
    /// Flags the buffer to generate samples, resets the running clock offset,
    /// and then returns a Vec<i16> of the generated samples.
    fn create_frame(&mut self) -> Vec<i16> {
        self.buffer.end_frame(8192);
        self.previous_clock = self.previous_clock.saturating_sub(8192);
        let samples = self.buffer.samples_avail();
        let mut ret = vec![0; samples as usize];
        self.buffer.read_samples(ret.as_mut_slice(), false);
        ret
    }
}

struct SquareChannel1 {
    /// CH1 Sweep Control (R/W)
    /// NR10 ($FF10)
    /// Bit 6-4 - Sweep pace
    /// Bit 3   - Sweep increase/decrease
    ///     0: Addition    (wavelength increases)
    ///     1: Subtraction (wavelength decreases)
    /// Bit 2-0 - Sweep slope control (n: 0-7)
    sweep_control: u8,

    /// CH1 Legnth Control (R/W)
    /// NR11 ($FF11)
    /// Bit 7-6 - Wave Pattern Duty (Read/Write)
    /// Bit 5-0 - Sound length data (Write Only) (t1: 0-63)
    /// Sound Length = (64-t1)*(1/256) seconds.
    /// The Length value is used only if Bit 6 in NR14 is set.
    length_data: u8,

    /// CH1 Volume Control (R/W)
    /// NR12 ($FF12)
    /// Bit 7-4 - Initial Volume of envelope (0-0Fh) (0=No Sound)
    /// Bit 3   - Envelope Direction (0=Decrease, 1=Increase)
    /// Bit 2-0 - Number of envelope sweep (n: 0-7)
    /// (If zero, stop envelope operation.)
    /// Length of 1 step = n*(1/64) seconds
    volume_control: u8,

    /// NR13 CH1 Wavelength Low (W)
    /// Lower 8-bits of frequency (wavelength) data
    /// Frequency = 131072/(2048-x) Hz
    frequency_low: u8,

    /// NR14 CH1 Wavelength High / Control (W)
    /// Bit 7   - Trigger (1=Restart channel)  (Write Only)
    /// Bit 6   - Sound Length enable          (Read/Write)
    ///           (1=Stop output when length in NR11 expires)
    /// Bit 2-0 - "Wavelength"'s higher 3 bits (Write Only)
    freq_high_control: u8,

    /// The current cycle count used to synchronize the timing of waveform generation
    /// with the rest of the system
    frequency_cycles: usize,

    /// The period of the frequency timer for waveform generation.
    /// Calculated every time the frequency is changed with the formula:
    ///     Period = 4 * (2048 - frequency)
    frequency_period: u32,

    /// The current location in the wave pattern given by wave_pattern
    wave_index: usize,

    /// Buffer containing the generated waveforms. Outputs data every 8192 CPU clocks,
    /// i.e. every clock of the frame sequencer
    buffer: SampleBuffer,
}

impl Memory for SquareChannel1 {
    fn read_byte(&self, addr: u16) -> u8 {
        assert!((0xFF10..=0xFF14).contains(&addr));
        match addr {
            0xFF10 => self.sweep_control,
            0xFF11 => self.length_data | 0x3F,
            0xFF12 => self.volume_control,
            0xFF13 => 0xFF,
            0xFF14 => self.freq_high_control | 0xBF,
            _ => unreachable!(),
        }
    }

    fn write_byte(&mut self, addr: u16, val: u8) {
        assert!((0xFF10..=0xFF14).contains(&addr));
        match addr {
            0xFF10 => self.sweep_control = val,
            0xFF11 => self.length_data = val,
            0xFF12 => self.volume_control = val,
            0xFF13 => self.frequency_low = val,
            0xFF14 => self.freq_high_control = val,
            _ => unreachable!(),
        }
    }
}

/// Type alias for easier usage by the caller
pub type AudioBuffer = Arc<Mutex<VecDeque<(i16, i16)>>>;

pub struct Apu {
    // Global Registers
    /// Channel control / ON-OFF / Volume (R/W)
    /// NR50 ($FF24)
    /// Bit 7   - Output Vin to SO2 terminal (1=Enable)
    /// Bit 6-4 - SO2 output level (volume)  (0-7)
    /// Bit 3   - Output Vin to SO1 terminal (1=Enable)
    /// Bit 2-0 - SO1 output level (volume)  (0-7)
    output_control: u8,

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
    /// Bit 3 - Sound 4 ON flag (Read Only)
    /// Bit 2 - Sound 3 ON flag (Read Only)
    /// Bit 1 - Sound 2 ON flag (Read Only)
    /// Bit 0 - Sound 1 ON flag (Read Only)
    sound_on: u8,

    /// Sound Channel 1 - Tone and Sweep
    /// NR10-NR14 ($FF10-$FF14)
    square1: SquareChannel1,

    /// The host sample rate to convert the generated waveforms into,
    /// provided at emulator power-on.
    sample_rate: u32,

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

    /// A buffer of a given buffer size that is used to store the generated
    /// samples prior to writing to audio driver output. Reduces the number
    /// of mutex locks on the driver output and provides a way to tweak
    /// the latency/crackling tradeoff.
    internal_buffer: VecDeque<(i16, i16)>,

    /// The final stereo output buffer at the host sample rate, after
    /// all mixing. A thread-safe Vec buffer of f32 samples, filled
    /// as the emulator generates samples. If the buffer is full,
    /// the APU will skip the samples until there's room.
    out_buffer: AudioBuffer,
}

impl Apu {
    pub fn power_on(sample_rate: u32) -> (Self, AudioBuffer) {
        let buf = Arc::new(Mutex::new(VecDeque::new()));
        let ret = buf.clone();
        (
            Apu {
                square1: SquareChannel1 {
                    sweep_control: 0x80,
                    length_data: 0xBF,
                    volume_control: 0xF3,
                    //frequency_low: 0xFF,
                    //freq_high_control: 0xBF,
                    frequency_low: 0xD6,
                    freq_high_control: 0xB6,
                    //frequency_period: 8192,
                    frequency_period: (2048 - 1750) * 4,
                    frequency_cycles: 0,
                    wave_index: 0,
                    buffer: SampleBuffer::new(sample_rate),
                },
                output_control: 0x77,
                channel_pan: 0xF3,
                sound_on: 0xF1,
                sample_rate,
                cycle_count: 0,
                frame_cycle: 0,
                internal_buffer: VecDeque::new(),
                out_buffer: buf,
            },
            ret,
        )
    }

    pub fn update(&mut self, cycles: usize) {
        //if test_bit(self.sound_on, 7) {
        for _ in 0..cycles {
            self.cycle_count += 1;

            // Update all channels
            self.square1.frequency_cycles += 1;

            // Check if the buffer needs to be updated with new samples to match the frequency
            if self.square1.frequency_cycles >= self.square1.frequency_period as usize {
                self.square1.frequency_cycles -= self.square1.frequency_period as usize;
                // Get Duty cycle pattern for wave
                let pattern = match extract_bits(self.square1.length_data, 7, 6) {
                    0x0 => 0b0000_0001, // 12.5%
                    0x1 => 0b1000_0001, // 25%
                    0x2 => 0b1000_0111, // 50%
                    0x3 => 0b0111_1110, // 75%
                    _ => unreachable!(),
                };

                // Get the current volume based on the volume envelope state
                // TODO: half volume for now, use stored envelope value
                let vol = 7;
                // Set amplitude to 0 if volume is zero or the channel hasn't been triggered
                let amp = //if test_bit(self.square1.freq_high_control, 7) && vol != 0 {
                        // Otherwise amplitude is vol if pattern is high at this step, -vol if pattern is low
                        if (pattern >> self.square1.wave_index) & 0x1 != 0x0 {
                            (i16::MAX / 100) as i32
                        } else {
                            (i16::MIN / 100) as i32
                        };
                // } else {
                //     0x0
                // };
                // Put amplitude value into buffer at the next base+period location
                self.square1
                    .buffer
                    .add_sample(self.square1.frequency_period, amp);
                self.square1.wave_index = (self.square1.wave_index + 1) % 8;

                self.square1.frequency_period = (2048
                    - (((self.square1.freq_high_control as u32 & 0b111) << 8)
                        | self.square1.frequency_low as u32))
                    * 4;

                if self.cycle_count >= 8192 {
                    // Increment the number of frame sequencer clocks
                    self.cycle_count -= 8192;
                    self.frame_cycle = (self.frame_cycle + 1) % 8;
                    if [0, 2, 4, 6].contains(&self.frame_cycle) {
                        // Update length counter if enabled
                    }
                    if [2, 6].contains(&self.frame_cycle) {
                        // Update Freq Sweep
                    }
                    if self.frame_cycle == 7 {
                        // Update volume envelope
                    }
                    let sq1_samples = self.square1.buffer.create_frame();
                    for s in sq1_samples {
                        self.internal_buffer.push_back((s, s));
                    }
                    if self.internal_buffer.len() >= BUFFER_SIZE {
                        let mut driver = self.out_buffer.lock().unwrap();
                        for d in self.internal_buffer.drain(0..BUFFER_SIZE) {
                            driver.push_back(d);
                        }
                    }
                }
            }
            // }
        }
    }
}

impl Memory for Apu {
    fn read_byte(&self, addr: u16) -> u8 {
        assert!((0xFF10..=0xFF3F).contains(&addr));
        match addr {
            0xFF10 => self.square1.sweep_control,
            0xFF11 => self.square1.length_data | 0x3F,
            0xFF12 => self.square1.volume_control,
            0xFF13 => 0xFF,
            0xFF14 => self.square1.freq_high_control | 0xBF,
            0xFF24 => self.output_control,
            0xFF25 => self.channel_pan,
            0xFF26 => self.sound_on,
            _ => {
                debug!("Unassigned APU memory location {:04X}", addr);
                0
            }
        }
    }
    fn write_byte(&mut self, addr: u16, val: u8) {
        assert!((0xFF10..=0xFF3F).contains(&addr));
        match addr {
            0xFF10 => self.square1.sweep_control = val,
            0xFF11 => self.square1.length_data = val,
            0xFF12 => self.square1.volume_control = val,
            0xFF13 => self.square1.frequency_low = val,
            0xFF14 => self.square1.freq_high_control = val,
            0xFF24 => self.output_control = val,
            0xFF25 => self.channel_pan = val,
            0xFF26 => self.sound_on = val & 0x80, // Only bit 7 is writable
            _ => debug!("Unassigned APU memory location {:04X}", addr),
        }
    }
}
