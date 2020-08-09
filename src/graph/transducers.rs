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

/// A node with an arbitrary but static number of inputs.
pub struct Sum {
    /// How many inputs to sum.
    count: usize,
}

impl Sum {
    pub fn new(count: usize) -> Self {
        Self { count }
    }
}

impl super::Node for Sum {
    fn num_inputs(&self) -> usize {
        self.count
    }
    fn num_outputs(&self) -> usize {
        1
    }
    fn render(&mut self, rio: &super::RenderIo) {
        let mut out = rio.output(0);
        out.fill_zero();
        let outsamples = out.samples_mut();

        for i in 0..self.count {
            let in_ref = rio.input(i);
            let in_samples = in_ref.samples();

            for (i, o) in in_samples.iter().zip(outsamples.iter_mut()) {
                *o += *i;
            }
        }
    }

}
