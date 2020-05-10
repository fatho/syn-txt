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
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    /// The source code of the music.
    #[structopt(parse(from_os_str))]
    source: PathBuf,
}

fn main() -> io::Result<()> {
    let opt = Opt::from_args();

    let level = if opt.verbose {
        log::Level::Trace
    } else {
        log::Level::Info
    };
    simple_logger::init_with_level(level).unwrap();

    let source = std::fs::read_to_string(&opt.source)?;
    let song = musicc::eval::eval(&opt.source.to_string_lossy(), &source)?;
    musicc::output::play(song)
}
