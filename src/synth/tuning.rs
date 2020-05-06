// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

use crate::note::*;

/// Defines the tuning of an instrument by assinging a frequency to a certain note.
/// This defines the frequencies of all other notes at a standard tuning of 12 half-tones per octave.
///
/// # Examples
///
/// ```
/// use syn_txt::note::*;
/// use syn_txt::synth::tuning::*;
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
            reference_note: Note::named(NoteName::A, NoteOffset::Base, 4),
            reference_frequency: 440.0,
        }
    }
}
