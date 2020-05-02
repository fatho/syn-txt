//! This namespace contains all the parts converting from note data to wave data.

pub mod envelope;
pub mod event;
pub mod oscillator;
pub mod test;
pub mod tuning;

// Also re-export everything necessary for defining the trait.
pub use event::*;

use crate::wave::Stereo;

/// A synthesizer turns events into sound.
pub trait Synthesizer {
    fn play(&mut self, events: &[Event], output: &mut [Stereo<f64>]);
}
