//! This namespace contains all the parts converting from note data to wave data.

pub mod test;
pub mod oscillator;
pub mod envelope;

use crate::note::*;

/// Defines the tuning of an instrument by assinging a frequency to a certain note.
/// This defines the frequencies of all other notes at a standard tuning of 12 half-tones per octave.
///
/// # Examples
///
/// ```
/// use syn_txt::note::*;
/// use syn_txt::synth::*;
/// assert_eq!(Tuning::default().frequency(Note::from_midi(57)), 220.0);
/// assert_eq!(Tuning::default().frequency(Note::from_midi(81)), 880.0);
/// ```
pub struct Tuning {
    pub reference_note: Note,
    pub reference_frequency: f64,
}

impl Tuning {
    /// Return the frequency of a note relative to this tuning.
    pub fn frequency(&self, other: Note) -> f64 {
        let semitones = other.index() - self.reference_note.index();
        let octaves = semitones as f64 / 12.0;
        self.reference_frequency * 2.0f64.powf(octaves)
    }
}

/// Default concert tuning, where A4 corresponds to 440 Hz.
impl Default for Tuning {
    fn default() -> Self {
        Tuning {
            reference_note: Note::named(NoteName::A, NoteOffset::Base, 4).unwrap(),
            reference_frequency: 440.0,
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
