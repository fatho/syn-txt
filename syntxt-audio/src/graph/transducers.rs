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
        Self::from_linear(syntxt_core::util::from_decibels(gain_db))
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
