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

//! Translate an abstract description of music into waveforms

use std::io;
use std::path::PathBuf;

use log::info;
use structopt::StructOpt;

use crate::graph;
use crate::instrument;
use crate::song::{Instrument, Song, Time, TimeSig};
use std::path::Path;

#[derive(Debug, StructOpt)]
#[structopt(name = "musicc", about = "Compiling code into music")]
struct Opt {
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,

    /// Final gain applied to the output of the song.
    #[structopt(short = "g", long = "gain", default_value = "1.0")]
    gain: f64,

    /// Output file (any sox-supported format). Music is played directly if not given.
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,

    /// Dump the description of the song generated from evaluating the code.
    #[structopt(long)]
    #[allow(clippy::option_option)]
    dump_description: Option<Option<PathBuf>>,
}

pub fn song_main<F: FnOnce() -> io::Result<crate::song::Song>>(compose: F) -> io::Result<()> {
    let opt: Opt = Opt::from_args();

    let level = match opt.verbose {
        0 => log::Level::Info,
        1 => log::Level::Debug,
        _ => log::Level::Trace,
    };
    simple_logger::init_with_level(level).unwrap();

    let dump_out = opt
        .dump_description
        .map(|path| path.unwrap_or_else(|| "/dev/stdout".into()));
    let song = compose()?;
    if let Some(dump_out_path) = dump_out {
        use std::io::Write;
        let mut f = std::fs::File::create(dump_out_path)?;
        writeln!(f, "{:?}", song)?;
    }
    play(song, opt.gain, opt.output.as_deref())
}

/// Play a song on the default speakers.
pub fn play(song: Song, output_gain: f64, outfile: Option<&Path>) -> io::Result<()> {
    let sample_rate = 44100;

    let sig = TimeSig {
        beats_per_minute: song.bpm,
        beat_unit: 4,
    };

    let mut graph_builder = graph::GraphBuilder::new();

    let last_note_end = song
        .tracks
        .iter()
        .flat_map(|t| t.notes.iter())
        .map(|n| n.start + n.duration)
        .max()
        .unwrap_or(Time::int(0));

    let players: Vec<_> = song
        .tracks
        .into_iter()
        .map(|track| match track.instrument {
            Instrument::Wavinator(ps) => graph_builder
                .add_node(graph::InstrumentSource::new(
                    sample_rate,
                    sig,
                    instrument::wavinator::Wavinator::with_params(sample_rate as f64, ps),
                    track.notes,
                ))
                .build(),
        })
        .collect();

    let target = match outfile {
        None => graph::SoxTarget::Play,
        Some(path) => graph::SoxTarget::File(path),
    };

    let mixer = players
        .iter()
        .enumerate()
        .fold(
            graph_builder.add_node(graph::Sum::new(players.len())),
            |accum, (index, item)| accum.input_from(index, item.output(0)),
        )
        .build();

    let output_gain = graph_builder
        .add_node(graph::Gain::from_decibels(output_gain))
        .input_from(0, mixer.output(0))
        .build();

    let _sink = graph_builder
        .add_node(graph::SoxSink::new(44100, target).unwrap())
        .input_from(0, output_gain.output(0))
        .build();

    // 10 ms buffer at 44100 Hz
    let buffer_size = 441;
    let max_samples = sig.samples(last_note_end + Time::int(2), sample_rate) + buffer_size - 1;

    let mut graph = graph_builder
        .build(buffer_size as usize)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    info!("playing at {} bpm at {} Hz", song.bpm, sample_rate);
    info!(
        "total length {} samples ({:.2} seconds)",
        max_samples,
        max_samples as f64 / sample_rate as f64
    );

    for _ in 0..(max_samples / buffer_size) {
        graph.step();
    }

    Ok(())
}
