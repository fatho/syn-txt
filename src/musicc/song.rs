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


use crate::{synth, pianoroll::PianoRoll};

pub struct Song {
    pub bpm: i64,
    pub notes: PianoRoll,
    pub instrument: Instrument,
}

pub enum Instrument {
    TestSynth(synth::test::Params),
}

pub struct Track {
    pub instrument: Instrument,
    pub notes: PianoRoll,
}
