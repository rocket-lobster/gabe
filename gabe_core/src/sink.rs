/// A trait that accepts input data for later processing
pub trait Sink<T> {
    fn append(&mut self, value: T);
}

/// A Sink type for accepting value references
pub trait SinkRef<T: ?Sized> {
    fn append(&mut self, value: &T);
}

/// A frame of video data, consisting of pixel data in an RGB format
pub type VideoFrame = Box<[u8]>;

/// A frame of audio data, consisting of (Left, Right) sample data of i16
pub type AudioFrame = (f32, f32);