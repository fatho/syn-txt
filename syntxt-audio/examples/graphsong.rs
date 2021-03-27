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

//! Evaluating how the graph interface could be used

use syntxt_audio::melody::parse_melody;
use syntxt_audio::song;
use syntxt_audio::{graph::*, wave::Stereo};
use syntxt_audio::{instrument::wavinator, song::Time};

fn main() {
    let instrument = wavinator::Wavinator::with_params(44100.0, wavinator::Params::default());
    let notes = parse_melody(
        r"
            a3- c4- a3- d4- a3- e4- a3- d4-
            a3- c4- a3- d4- a3- e4- a3- d4-
            { { c4- d4- e4- d4- } a3+ } { { c4- d4- e4- d4- } a3+ }
            { a3 c4 } { a3 d4 } { a3 c4 } r
        ",
    )
    .unwrap();
    let last_note_end = notes
        .iter()
        .map(|n| n.start + n.duration)
        .max()
        .unwrap_or(Time::int(0));
    let sig = song::TimeSig {
        beats_per_minute: 128,
        beat_unit: 4,
    };

    let mut builder = GraphBuilder::new();

    let source = builder
        .add_node(InstrumentSource::new(44100, sig, instrument, notes))
        .build();

    let _sink = builder
        .add_node(SoxSink::new(44100, SoxTarget::Play).unwrap())
        .input_from(0, source.output(0))
        .build();

    let buffer_size = 1024;
    let mut graph = builder.build(1024).unwrap();
    let total_samples = sig.samples(last_note_end + Time::int(2), 44100) + buffer_size - 1;
    for _ in 0..(total_samples / buffer_size) {
        graph.step();
    }
}

/// Add audio streams together.
pub struct Sum {
    num_inputs: usize,
}

impl Node for Sum {
    fn num_inputs(&self) -> usize {
        self.num_inputs
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn render(&mut self, rio: &RenderIo) {
        let mut out = rio.output(0);
        out.fill_zero();
        let outsamples = out.samples_mut();

        for i in 0..self.num_inputs {
            let in_ref = rio.input(i);
            let in_samples = in_ref.samples();

            for (i, o) in in_samples.iter().zip(outsamples.iter_mut()) {
                *o += *i;
            }
        }
    }
}

/// Render all the incoming audio data on the terminal.
pub struct DebugSink;

impl Node for DebugSink {
    fn num_outputs(&self) -> usize {
        0
    }

    fn num_inputs(&self) -> usize {
        1
    }

    fn render(&mut self, rio: &RenderIo) {
        let input = rio.input(0);

        println!("[{}]", rio.start());
        println!("|{: ^41}|{: ^41}|", "left", "right");
        println!(
            "|-----------------------------------------|-----------------------------------------|"
        );
        let half_width = 20;
        let width = half_width * 2;
        for s in input.samples() {
            let lpos = half_width + (s.left * half_width as f64).round() as i64;
            let rpos = half_width + (s.right * half_width as f64).round() as i64;

            print!("|");
            (0..lpos).for_each(|_| print!(" "));
            print!("*");
            (0..width - lpos).for_each(|_| print!(" "));
            print!("|");
            (0..rpos).for_each(|_| print!(" "));
            print!("*");
            (0..width - rpos).for_each(|_| print!(" "));
            println!("|");
        }
    }
}

/// Continuously generate a simple sine wave.
pub struct Sine {
    samples_per_second: Sample,
    amplitude: f64,
    frequency: f64,
}

impl Node for Sine {
    fn num_outputs(&self) -> usize {
        1
    }

    fn num_inputs(&self) -> usize {
        0
    }

    fn render(&mut self, rio: &RenderIo) {
        let mut out = rio.output(0);
        let buf = out.samples_mut();
        for i in 0..rio.length() {
            let t = (i + rio.start()) as f64 / self.samples_per_second as f64;
            buf[i] = Stereo::mono(
                (2.0 * std::f64::consts::PI * t * self.frequency).sin() * self.amplitude,
            );
        }
    }
}
