use super::time_source::*;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Sample, SampleFormat,
};
use gabe_core::sink::*;
use log::*;

use std::sync::*;

/// A ring buffer of audio samples
/// Tracks sample count in order to provide a time source
struct SampleBuffer {
    inner: Box<[f32]>,
    write_index: usize,
    read_index: usize,
    count: usize,
    samples_read: u64,
    sample_rate: u32,
}

impl SampleBuffer {
    /// Pushes the given sample into the ring buffer.
    /// Increments the internal sample counter.
    fn push(&mut self, value: f32) {
        self.inner[self.write_index] = value;
        self.write_index += 1;

        self.count += 1;

        if self.count >= self.inner.len() {
            self.count = self.inner.len()
        }

        if self.write_index >= self.inner.len() {
            self.write_index = 0;
        }
    }
}

impl Iterator for SampleBuffer {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.samples_read += 1;
        if self.count != 0 {
            let ret = self.inner[self.read_index];
            self.read_index += 1;

            if self.read_index >= self.inner.len() {
                self.read_index = 0;
            }
            self.count -= 1;
            Some(ret)
        } else {
            None
        }
    }
}

pub struct AudioDriverTimeSource {
    buffer: Arc<Mutex<SampleBuffer>>,
}

impl TimeSource for AudioDriverTimeSource {
    fn time_ns(&self) -> u64 {
        let buf = self.buffer.lock().unwrap();
        1_000_000_000 * (buf.samples_read / 2) / (buf.sample_rate as u64)
    }
}

pub struct AudioDriverSink {
    buffer: Arc<Mutex<SampleBuffer>>,
}

impl SinkRef<[AudioFrame]> for AudioDriverSink {
    fn append(&mut self, value: &[AudioFrame]) {
        let mut buf = self.buffer.lock().unwrap();
        for &(l, r) in value {
            buf.push(l);
            buf.push(r);
        }
    }
}

pub struct AudioDriver {
    buffer: Arc<Mutex<SampleBuffer>>,
    _stream: cpal::Stream,
}

impl AudioDriver {
    pub fn new(sample_rate: u32, latency_ms: u32) -> Self {
        // Set up audio device, use default device.
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("No audio output device available.");

        let supported_configs_range = device
            .supported_output_configs()
            .expect("error while querying configs");

        // Use the provided cmp_default_heuristics to find the best config supported
        // Prioritizes 2 channels, gets highest sample rate.
        let best_config = supported_configs_range
            .max_by(|x, y| x.cmp_default_heuristics(y))
            .expect("No supported output configs for device.")
            .with_sample_rate(cpal::SampleRate(48000));

        let err_fn = |err| error!("An error occurred on the output audio stream: {}", err);
        let sample_format = best_config.sample_format();
        let buffer_samples = (sample_rate * latency_ms / 1000 * 2) as usize;
        info!("Sound: ");
        info!("\t Device: {:?}", device.name().unwrap());
        info!("\t Device sample format: {:?}", sample_format);
        info!("\t Device sample rate: {:?}", best_config.sample_rate().0);
        info!("\t Device channels: {:?}", best_config.channels());

        let config = best_config.config();
        let audio_buffer = Arc::new(Mutex::new(SampleBuffer {
            inner: vec![0.0; buffer_samples].into_boxed_slice(),
            samples_read: 0,
            sample_rate,
            count: 0,
            write_index: 0,
            read_index: 0,
        }));

        // Resample from requested sample rate to the config's sample rate
        let mut resampler = LinearResampler::new(sample_rate, config.sample_rate.0);

        let read_audio_buffer = audio_buffer.clone();
        let stream = match sample_format {
            SampleFormat::F32 => device.build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut buffer = read_audio_buffer.lock().unwrap();
                    for frame in data.chunks_mut(2) {
                        for sample in frame.iter_mut() {
                            *sample = Sample::from(&resampler.next(&mut *buffer));
                        }
                    }
                },
                err_fn,
            ),
            SampleFormat::I16 => device.build_output_stream(
                &config,
                move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                    let mut buffer = read_audio_buffer.lock().unwrap();
                    for frame in data.chunks_mut(2) {
                        for sample in frame.iter_mut() {
                            *sample = Sample::from(&resampler.next(&mut *buffer));
                        }
                    }
                },
                err_fn,
            ),
            SampleFormat::U16 => device.build_output_stream(
                &config,
                move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                    let mut buffer = read_audio_buffer.lock().unwrap();
                    for frame in data.chunks_mut(2) {
                        for sample in frame.iter_mut() {
                            *sample = Sample::from(&resampler.next(&mut *buffer));
                        }
                    }
                },
                err_fn,
            ),
        }
        .unwrap();

        stream.play().unwrap();

        AudioDriver {
            buffer: audio_buffer,
            _stream: stream,
        }
    }

    pub fn sink(&self) -> Box<dyn SinkRef<[AudioFrame]>> {
        Box::new(AudioDriverSink {
            buffer: self.buffer.clone(),
        })
    }

    pub fn time_source(&self) -> Box<dyn TimeSource> {
        Box::new(AudioDriverTimeSource {
            buffer: self.buffer.clone(),
        })
    }
}

/// Performs linear interpolation on audio samples
/// Can upsample or downsample, depending on the provided sample rates
struct LinearResampler {
    from_rate: u32,
    to_rate: u32,
    current_from: AudioFrame,
    next_from: AudioFrame,
    from_fractional_pos: u32,
    current_frame_channel: u32,
}

impl LinearResampler {
    /// Creates a new LinearResampler, resampling at `from_sample_rate` into `to_sample_rate`
    fn new(from_sample_rate: u32, to_sample_rate: u32) -> Self {
        let sample_rate_gcd = {
            fn gcd(a: u32, b: u32) -> u32 {
                if b == 0 {
                    a
                } else {
                    gcd(b, a % b)
                }
            }

            gcd(from_sample_rate, to_sample_rate)
        };

        LinearResampler {
            from_rate: from_sample_rate / sample_rate_gcd,
            to_rate: to_sample_rate / sample_rate_gcd,
            current_from: (0.0, 0.0),
            next_from: (0.0, 0.0),
            from_fractional_pos: 0,
            current_frame_channel: 0,
        }
    }

    /// Generates a new sample from the given `input` samples `Iterator` object.
    /// Uses linear interpolation to either upsample or downsample from the input
    fn next(&mut self, input: &mut dyn Iterator<Item = f32>) -> f32 {
        // Helper function for interpolating between values
        fn interpolate(a: f32, b: f32, num: u32, denom: u32) -> f32 {
            (a * ((denom - num) as f32) + b * (num as f32)) / (denom as f32)
        }

        // Check which channel to process of the current frame
        let ret = match self.current_frame_channel {
            0 => interpolate(
                self.current_from.0,
                self.next_from.0,
                self.from_fractional_pos,
                self.to_rate,
            ),
            _ => interpolate(
                self.current_from.1,
                self.next_from.1,
                self.from_fractional_pos,
                self.to_rate,
            ),
        };
        self.current_frame_channel += 1;

        // Check if both channels are processed
        if self.current_frame_channel >= 2 {
            // Set up next frame to resample
            self.current_frame_channel = 0;

            self.from_fractional_pos += self.from_rate;

            // Check if it's time to get another frame
            while self.from_fractional_pos > self.to_rate {
                self.from_fractional_pos -= self.to_rate;
                self.current_from = self.next_from;

                let left = input.next().unwrap_or(0.0);
                let right = input.next().unwrap_or(0.0);
                self.next_from = (left, right);
            }
        }
        ret
    }
}
