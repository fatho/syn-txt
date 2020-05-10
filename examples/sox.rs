// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! Example demonstrating how to use the sox-based audio output.

use std::io;
use syn_txt;

fn main() -> io::Result<()> {
    let sample_rate = 44100;

    syn_txt::output::sox::with_sox(sample_rate, syn_txt::output::sox::SoxTarget::Play, |audio_stream| {
        let mut test_buffer: Vec<u8> = Vec::new();
        for i in 0..sample_rate {
            let phase = (i as f64 / sample_rate as f64) * 440.0 * 2.0 * std::f64::consts::PI;
            let amp = phase.sin();
            let bytes = amp.to_le_bytes();
            // Once for each channel
            test_buffer.extend_from_slice(&bytes);
            test_buffer.extend_from_slice(&bytes);
        }
        audio_stream.write(&test_buffer)?;
        Ok(())
    })
}
