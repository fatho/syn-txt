// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2021  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use syntxt_core::note::*;

/// Defines the tuning of an instrument by assinging a frequency to a certain note.
/// This defines the frequencies of all other notes at a standard tuning of 12 half-tones per octave.
///
/// # Examples
///
/// ```
/// use syntxt_core::note::*;
/// use syntxt_audio::tuning::*;
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
            reference_note: Note::named(NoteName::A, Accidental::Base, 4),
            reference_frequency: 440.0,
        }
    }
}
