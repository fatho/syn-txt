//! This is the namespace for all parts dealing with data in sampled waves.

/// Time measured in samples since some unspecified epoch (e.g. start of the song).
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SampleTime(pub usize);

/// Information about how audio is sampled.
pub struct SamplerInfo {
    /// Number of samples per second.
    pub sample_rate: i32,
    /// Number of samples in the buffer.
    // This determines the minimum delay for feedback loops and
    /// real-time audio processing (e.g. external MIDI events).
    pub buffer_size: usize,
}

/// Convenience type for making things stereo, e.g. individual samples or whole buffers.
#[derive(Copy, Clone, Debug)]
pub struct Stereo<T> {
    pub left: T,
    pub right: T,
}
