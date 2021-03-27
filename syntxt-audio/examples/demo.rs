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

use syntxt_audio::instrument::wavinator;
use syntxt_audio::melody::parse_melody;
use syntxt_audio::play;
use syntxt_audio::song::*;
use syntxt_audio::{automation::Expr, filter::BiquadType, oscillator::WaveShape};

use std::io;

#[rustfmt::skip]
fn main() -> io::Result<()> {
    play::song_main(|| {
        let song = Song {
            bpm: 128,
            tracks: vec![
                Track {
                    instrument: Instrument::Wavinator(
                        wavinator::Params {
                            gain: Expr::Const(0.5),
                            pan: Expr::parse("sin time").unwrap(),
                            unison: 16,
                            unison_detune_cents: 0.1,
                            unison_spread: 1.0,
                            filter: BiquadType::Lowpass { cutoff: 8000.0, q: 2.0f64.sqrt().recip() },
                            wave_shape: WaveShape::SuperSaw,
                            ..wavinator::Params::default()
                        }),
                    notes: parse_melody(r"
                        r+++
                        c3-- d3-- e3-- g3-- a3--
                        d3-- e3-- g3-- a3-- c4--
                        e3-- g3-- a3-- c4-- d4--
                        g3-- a3-- c4-- d4-- e4--
                        a3-- c4-- d4-- e4-- g4--
                        c4-- d4-- e4-- g4-- a4--
                        a3- c4- a3- d4- a3- e4- a3- d4-
                        a3- c4- a3- d4- a3- e4- a3- d4-
                        { { c4- d4- e4- d4- } a3+ } { { c4- d4- e4- d4- } a3+ }
                        { a3 c4 } { a3 d4 } { a3 c4 } r
                    ").unwrap(),
                },
                Track {
                    instrument: Instrument::Wavinator(
                        wavinator::Params {
                            gain: Expr::Const(0.5),
                            unison: 2,
                            wave_shape: WaveShape::Rectangle,
                            filter: BiquadType::Lowpass { cutoff: 1000.0, q: 2.0f64.sqrt().recip() },
                            ..wavinator::Params::default()
                        }),
                    notes: parse_melody(r"
                        a1 a2- a1- a1- a1- a2
                        e1 e2- e1 e1- e2
                        d1 d2- d1- d1- d1- d2
                        a1 a2- a1 a1- a2
                        a1 a2- a1- a1- a1- a2
                        e1 e2- e1 e1- e2
                    ").unwrap(),
                },
            ],
        };
        Ok(song)
    })
}
