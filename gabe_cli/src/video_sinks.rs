#![allow(dead_code)]

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

pub struct BlendVideoSink {
    inner: Option<VideoFrame>,
    frames_blended: u16,
}

impl BlendVideoSink {
    pub fn new() -> Self {
        BlendVideoSink {
            inner: None,
            frames_blended: 0,
        }
    }

    pub fn into_inner(self) -> Option<VideoFrame> {
        self.inner
    }
}

impl Sink<VideoFrame> for BlendVideoSink {
    fn append(&mut self, value: VideoFrame) {
        if self.inner.is_some() {
            let new_frame: VideoFrame = self.inner.as_mut().unwrap().iter().zip(value.iter()).map(|(x1, x2)| {
                ((*x2 as u16 + (self.frames_blended * *x1 as u16)) / (self.frames_blended + 1)) as u8
            }).collect();
            self.inner = Some(new_frame);
            self.frames_blended += 1;
        } else {
            self.inner = Some(value);
            self.frames_blended += 1;
        }
    }
}
