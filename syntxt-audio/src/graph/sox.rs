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
use std::io::Write;
use std::path::Path;
use std::process::{ChildStdin, Command, Stdio};

use log::error;
pub enum SoxTarget<'a> {
    Play,
    File(&'a Path),
}

pub struct SoxSink {
    audio_stream: ChildStdin,
    buffer: Vec<u8>,
    error: bool,
}

impl SoxSink {
    pub fn new(sample_rate: i32, target: SoxTarget) -> io::Result<Self> {
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

        // For properly recording the sox dependency on nix:
        let (play, sox) = if let Some(sox_bin) = option_env!("NIX_SOX_BIN") {
            log::debug!("using sox from nix store {}", sox_bin);
            let play = Path::new(sox_bin).join("play");
            let sox = Path::new(sox_bin).join("sox");
            (play, sox)
        } else {
            ("play".into(), "sox".into())
        };

        let mut player = match target {
            SoxTarget::Play => Command::new(&play)
                .args(input_args)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?,
            SoxTarget::File(outfile) => Command::new(&sox)
                .args(input_args)
                .arg(outfile)
                .stdin(Stdio::piped())
                .spawn()?,
        };

        let audio_stream = player.stdin.take().expect("Used stdin(Stdio::piped())");

        // sox should exit automatically once the input stream is closed,
        // there's no special precaution required.

        Ok(Self {
            audio_stream,
            buffer: Vec::new(),
            error: false,
        })
    }
}

impl super::Node for SoxSink {
    fn num_inputs(&self) -> usize {
        1
    }
    fn num_outputs(&self) -> usize {
        0
    }
    fn render(&mut self, rio: &super::RenderIo) {
        if self.error {
            return;
        }

        let inp = rio.input(0);
        if self.buffer.len() < inp.byte_len() {
            self.buffer.resize(inp.byte_len(), 0);
        }
        inp.copy_bytes_to(&mut self.buffer);

        let status = self
            .audio_stream
            .write_all(&self.buffer)
            .and_then(|_| self.audio_stream.flush());
        if let Err(err) = status {
            error!("Failed to write audio to sox stream: {}", err);
            self.error = true;
        }
    }
}
