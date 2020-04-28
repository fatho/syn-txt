//! The glue responsible for turning the description of music into actual waveforms.

use crate::synth::{Synthesizer, Event};
use crate::wave::Stereo;

/// Feeds a synthesizer with events, generating (buffered) audio output.
pub struct SynthPlayer<'e, S> {
    current_sample: usize,
    synth: S,
    events: &'e [Event],
}

impl<'e, S: Synthesizer> SynthPlayer<'e, S> {
    /// The events must be ordered chronologically by their start time.
    pub fn new(synth: S, events: &'e [Event]) -> Self {
        Self { synth, events, current_sample: 0 }
    }

    pub fn remaining_events(&self) -> &'e [Event] {
        self.events
    }

    pub fn generate(&mut self, output: &mut [Stereo<f64>]) {
        self.synth.play(self.events, output);
        self.current_sample += output.len();

        // Skip all events that were processed in this iteration
        let skipped = self.events.iter()
            .position(|e| e.time >= self.current_sample)
            .unwrap_or(self.events.len());
        self.events = &self.events[skipped..];
    }

    pub fn into_synth(self) -> S {
        self.synth
    }
}
