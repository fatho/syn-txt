// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! `musicc` - pronounced *music-c*, is the compiler for syntxt files to wav files.

use std::io;
use std::path::PathBuf;

use simple_logger;
use structopt::StructOpt;

use syn_txt::musicc;

#[derive(Debug, StructOpt)]
#[structopt(name = "musicc", about = "Compiling code into music")]
struct Opt {
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,

    /// The source code of the music.
    #[structopt(parse(from_os_str))]
    source: PathBuf,

    /// Output file (any sox-supported format). Music is played directly if not given.
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,

    /// Dump the description of the song generated from evaluating the code.
    #[structopt(long)]
    dump_description: Option<Option<PathBuf>>,
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    let level = match opt.verbose {
        0 => log::Level::Info,
        1 => log::Level::Debug,
        _ => log::Level::Trace,
    };
    simple_logger::init_with_level(level).unwrap();

    let source = std::fs::read_to_string(&opt.source)?;
    let dump_out = opt
        .dump_description
        .map(|path| path.unwrap_or("/dev/stdout".into()));
    let song = musicc::eval::eval(&opt.source.to_string_lossy(), &source, dump_out.as_deref())?;
    musicc::output::play(song, opt.output.as_deref())
}
