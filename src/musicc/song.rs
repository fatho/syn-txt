// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! High-level description of a song that can be turned into audio.

use crate::{pianoroll::PianoRoll, synth};

/// A description of a complete song.
pub struct Song {
    /// The speed of the song measured in beats per minute.
    pub bpm: i64,
    /// The tracks of the song, playing simultaneously.
    pub tracks: Vec<Track>,
}

/// The instrument used for playing a track.
pub enum Instrument {
    /// The built-in test synthesizer.
    TestSynth(synth::test::Params),
}

/// A single track generating sound by playing notes on an instrument.
pub struct Track {
    pub instrument: Instrument,
    pub notes: PianoRoll,
}
