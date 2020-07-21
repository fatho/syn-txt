// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

use syn_txt::melody::parse_melody;
use syn_txt::play;
use syn_txt::song::*;
use syn_txt::synth;

use std::io;

#[rustfmt::skip]
fn main() -> io::Result<()> {
    play::song_main(|| {
        let song = Song {
            bpm: 128,
            tracks: vec![
                Track {
                    instrument: Instrument::TestSynth(synth::test::Params::default()),
                    notes: parse_melody(r"
                        a3- c4- a3- d4- a3- e4- a3- d4-
                        a3- c4- a3- d4- a3- e4- a3- d4-
                        { { c4- d4- e4- d4- } a3+ } { { c4- d4- e4- d4- } a3+ }
                        { a3 c4 } { a3 d4 } { a3 c4 } r
                    ").unwrap(),
                }
            ],
        };
        Ok(song)
    })
}
