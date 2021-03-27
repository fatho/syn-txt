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

#[derive(Debug, Copy, Clone)]
pub struct Phase(f64);

impl Phase {
    pub const ZERO: Phase = Phase(0.0);

    pub fn new(mut offset: f64) -> Phase {
        if offset >= 1.0 {
            while offset >= 1.0 {
                offset -= 1.0;
            }
        } else if offset < 0.0 {
            while offset < 0.0 {
                offset += 1.0;
            }
        }
        Phase(offset)
    }

    pub fn offset(self) -> f64 {
        self.0
    }

    pub fn step(self, amount: f64) -> Phase {
        Phase::new(self.0 + amount)
    }

    pub fn step_frequency(self, frequency: f64, sample_rate: f64) -> Phase {
        self.step(frequency / sample_rate)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum WaveShape {
    Sine,
    Rectangle,
    Triangle,
    Saw,
    SuperSaw,
    TwoSidedSaw,
    AlternatingSaw,
}

impl WaveShape {
    pub fn eval(self, phase: Phase) -> f64 {
        let offset = phase.offset();
        use std::f64::consts::PI;
        match self {
            WaveShape::Sine => (offset * 2.0 * PI).sin(),
            WaveShape::Rectangle => {
                if offset < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            WaveShape::Triangle => {
                if offset < 0.25 {
                    4.0 * offset
                } else if offset < 0.75 {
                    2.0 - 4.0 * offset
                } else {
                    4.0 * offset - 4.0
                }
            }
            WaveShape::Saw => 2.0 * offset - 1.0,
            WaveShape::SuperSaw => {
                let slope = 3.0;
                if offset < 0.5 {
                    slope * offset - 1.0
                } else {
                    1.0 + slope * (offset - 1.0)
                }
            }
            WaveShape::TwoSidedSaw => {
                if offset < 0.5 {
                    2.0 * offset
                } else {
                    -2.0 * (offset - 0.5)
                }
            }
            WaveShape::AlternatingSaw => {
                let upsaw = 2.0 * offset - 1.0;
                let downsaw = -upsaw;
                let breaks = 5;
                let piece = (offset * (breaks + 1) as f64).trunc() as i32;
                if piece % 2 == 0 {
                    upsaw
                } else {
                    downsaw
                }
            }
        }
    }
}

/// An oscillator sampling a wave of some shape at a fixed sample rate.
#[derive(Debug)]
pub struct Oscillator {
    shape: WaveShape,
    sample_rate: f64,
    frequency: f64,
    phase: Phase,
}

impl Oscillator {
    pub fn new(shape: WaveShape, sample_rate: f64, frequency: f64) -> Self {
        Self {
            shape,
            sample_rate,
            frequency,
            phase: Phase::ZERO,
        }
    }

    pub fn next_sample(&mut self) -> f64 {
        let result = self.shape.eval(self.phase);
        self.phase = self.phase.step_frequency(self.frequency, self.sample_rate);
        result
    }
}
