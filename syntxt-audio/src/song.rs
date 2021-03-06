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

//! High-level description of a song that can be turned into audio.

use crate::instrument;
use syntxt_core::note::{Note, Velocity};
use syntxt_core::rational::Rational;

/// A description of a complete song.
#[derive(Debug)]
pub struct Song {
    /// The speed of the song measured in beats per minute.
    pub bpm: i64,
    /// The tracks of the song, playing simultaneously.
    pub tracks: Vec<Track>,
}

/// The instrument used for playing a track.
#[derive(Debug)]
pub enum Instrument {
    /// The built-in test synthesizer.
    Wavinator(instrument::wavinator::Params),
}

/// A single track generating sound by playing notes on an instrument.
#[derive(Debug)]
pub struct Track {
    pub instrument: Instrument,
    pub notes: Vec<PlayedNote>,
}

/// Time in measures, can be fractional, e.g. a note taking 1/4.
/// The time is relative until the music is put into a song with a specific measure.
pub type Time = Rational;

#[derive(Debug, Clone, PartialEq)]
pub struct PlayedNote {
    /// Which key was pressed
    pub note: Note,
    /// How hard the key was pressed
    pub velocity: Velocity,
    /// Time when the key was pressed
    pub start: Time,
    /// Time when the key was released
    pub duration: Time,
}

/// Time signature of the song, consisting of
/// - the number of beats per minute,
/// - the length of a single beat
/// Note that this omits the number of beats per bar,
/// which is not needed for computing time from beats.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TimeSig {
    /// How many beats per minute
    pub beats_per_minute: i64,
    /// The length of one beat is `1 / beat_unit`.
    pub beat_unit: i64,
}

impl TimeSig {
    pub fn seconds(&self, note_time: Rational) -> Rational {
        note_time * self.beat_unit * 60 / self.beats_per_minute
    }

    pub fn samples(&self, note_time: Rational, samples_per_second: i64) -> i64 {
        (self.seconds(note_time) * samples_per_second).round()
    }
}
