// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.


pub struct Gain {
    gain: f64,
}

impl Gain {
    /// Create a gain node that applies a linear gain.
    pub fn from_linear(gain: f64) -> Self {
        Self { gain }
    }

    /// Create a gain node that applies a logarithmic gain measured in decibels.
    pub fn from_decibels(gain_db: f64) -> Self {
        Self::from_linear(crate::util::from_decibels(gain_db))
    }
}

impl super::Node for Gain {
    fn num_inputs(&self) -> usize {
        1
    }
    fn num_outputs(&self) -> usize {
        1
    }
    fn render(&mut self, rio: &super::RenderIo) {
        let input = rio.input(0);
        let mut output = rio.output(0);

        for (i, o) in input.iter().zip(output.iter_mut()) {
            *o = *i * self.gain;
        }
    }

}
