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
use std::process::{Command, Stdio};

pub fn with_sox_player<R, F: FnOnce(&mut dyn io::Write) -> io::Result<R>>(
    sample_rate: i32,
    callback: F,
) -> io::Result<R> {
    let mut player = Command::new("play")
        .arg("--channels")
        .arg("2")
        .arg("--rate")
        .arg(format!("{}", sample_rate))
        .arg("--type")
        .arg("f64")
        .arg("/dev/stdin")
        .arg("stats")
        .arg("spectrogram")
        .arg("-y")
        .arg("513")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    let mut audio_stream = player.stdin.take().expect("Used stdin(Stdio::piped())");

    let result = callback(&mut audio_stream);

    drop(audio_stream);
    player.wait()?;

    result
}

pub fn with_sox_wav<R, F: FnOnce(&mut dyn io::Write) -> io::Result<R>>(
    sample_rate: i32,
    callback: F,
) -> io::Result<R> {
    let mut player = Command::new("sox")
        .arg("-R")
        .arg("--channels")
        .arg("2")
        .arg("--rate")
        .arg(format!("{}", sample_rate))
        .arg("--type")
        .arg("f64")
        .arg("/dev/stdin")
        .arg("--type")
        .arg("wavpcm")
        .arg("out.wav")
        .stdin(Stdio::piped())
        .spawn()?;

    let mut audio_stream = player.stdin.take().expect("Used stdin(Stdio::piped())");

    let result = callback(&mut audio_stream);

    drop(audio_stream);
    player.wait()?;

    result
}
