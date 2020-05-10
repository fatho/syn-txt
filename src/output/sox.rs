// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! Easy interface for getting sound to play using a sox subprocess.
use std::io;
use std::path::Path;
use std::process::{Command, Stdio};

pub enum SoxTarget<'a> {
    Play,
    File(&'a Path),
}

pub fn with_sox<R, F: FnOnce(&mut dyn io::Write) -> io::Result<R>>(
    sample_rate: i32,
    target: SoxTarget,
    callback: F,
) -> io::Result<R> {
    let sample_rate_str = format!("{}", sample_rate);
    let input_args = &[
        "-R", // make the output reproducible
        "--channels",
        "2",
        "--rate",
        &sample_rate_str,
        "--type",
        "f64",
        "/dev/stdin",
    ];
    let mut player = match target {
        SoxTarget::Play => Command::new("play")
            .args(input_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?,
        SoxTarget::File(outfile) => Command::new("sox")
            .args(input_args)
            .arg(outfile)
            .stdin(Stdio::piped())
            .spawn()?,
    };

    let mut audio_stream = player.stdin.take().expect("Used stdin(Stdio::piped())");

    let result = callback(&mut audio_stream);

    drop(audio_stream);
    player.wait()?;

    result
}
