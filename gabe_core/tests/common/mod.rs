#![allow(dead_code)]

use std::io::Write;

use gabe_core::sink::*;

pub struct MostRecentSink {
    inner: Option<VideoFrame>,
}

impl MostRecentSink {
    pub fn new() -> Self {
        MostRecentSink { inner: None }
    }

    pub fn has_frame(&self) -> bool {
        self.inner.is_some()
    }

    pub fn get_frame(&mut self) -> Option<VideoFrame> {
        if self.inner.is_some() {
            let ret = self.inner.as_ref().unwrap().clone();
            self.inner = None;
            Some(ret)
        } else {
            None
        }
    }

    pub fn into_inner(self) -> Option<VideoFrame> {
        self.inner
    }
}

impl Sink<VideoFrame> for MostRecentSink {
    fn append(&mut self, value: VideoFrame) {
        self.inner = Some(value);
    }
}

pub struct NullSink;

impl Sink<VideoFrame> for NullSink {
    fn append(&mut self, _value: VideoFrame) {}
}

impl Sink<AudioFrame> for NullSink {
    fn append(&mut self, _value: AudioFrame) {}
}

pub fn run_dmg_sound_case(gb: &mut gabe_core::gb::Gameboy) -> bool {
    let mut video_sink = NullSink;
    let mut audio_sink = NullSink;
    let mut output_ptr: usize = 0xA004;
    let mut cycles = 0;
    const CYCLE_TIMEOUT: u32 = 4194304;
    loop {
        cycles += gb.step(&mut video_sink, &mut audio_sink);
        // Get test data from $A000. Signature of $DE, $B0, $61 in $A001-$A003
        let data = gb.get_memory_range(0xA000..0xA004);
        if data[1] == 0xDE && data[2] == 0xB0 && data[3] == 0x61 {
            // 0x80 means the test is still running
            if (data[0] != 0x80) && (cycles > CYCLE_TIMEOUT) {
                return data[0] == 0;
            } else {
                let str_data = gb.get_memory_range(output_ptr .. output_ptr + 5);
                for c in str_data.into_iter() {
                    if *c == 0 {
                        break;
                    } else {
                        print!("{}", *c as char);
                        std::io::stdout().flush().unwrap();
                        output_ptr += 1;
                    }
                }
            }
        }
    }
}