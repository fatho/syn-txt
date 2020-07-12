// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! Implementation of the music compiler (musicc).

pub mod output;
pub mod song;

use std::io;
use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "musicc", about = "Compiling code into music")]
struct Opt {
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,

    /// Output file (any sox-supported format). Music is played directly if not given.
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,

    /// Dump the description of the song generated from evaluating the code.
    #[structopt(long)]
    #[allow(clippy::option_option)]
    dump_description: Option<Option<PathBuf>>,
}

pub fn song_main<F: FnOnce() -> io::Result<song::Song>>(compose: F) -> io::Result<()> {
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
    output::play(song, opt.output.as_deref())
}
