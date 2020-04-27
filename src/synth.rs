//! This namespace contains all the parts converting from note data to wave data.

pub mod test;

use crate::note::*;
use crate::wave::*;

/// Defines the tuning of an instrument by assinging a frequency to a certain note.
/// This defines the frequencies of all other notes at a standard tuning of 12 half-tones per octave.
pub struct Tuning {
    pub note: Note,
    pub frequency: f64,
}

impl Default for Tuning {
    fn default() -> Self {
        Tuning {
            note: Note::named(NoteName::A, NoteOffset::Base, 4).unwrap(),
            frequency: 440.0,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    /// Time of the event in samples since playback started.
    pub time: usize,
    /// What kind of event happened at this time.
    pub action: NoteAction,
}
