// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! Digital filters galore

/// Pre-define types of biquad filters that can be used for deriving
/// various combinations of `BiquadCoefficients`.
#[derive(Debug, Clone)]
pub enum BiquadType {
    /// The identity filter that lets the signal pass unchanged.
    Allpass,
    /// Lowpass filter with the given cutoff frequency and Q factor (controls resonance)
    Lowpass { cutoff: f64, q: f64 },
}

impl BiquadType {
    pub fn to_coefficients(&self, sample_rate: f64) -> BiquadCoefficients {
        match self {
            BiquadType::Allpass => BiquadCoefficients::allpass(),
            BiquadType::Lowpass { cutoff, q } => {
                BiquadCoefficients::lowpass(sample_rate, *cutoff, *q)
            }
        }
    }
}

/// Filter coefficients for a biquadratic filter,
/// based on https://www.w3.org/2011/audio/audio-eq-cookbook.html.
#[derive(Debug, Clone)]
pub struct BiquadCoefficients {
    pub b0: f64,
    pub b1: f64,
    pub b2: f64,
    pub a1: f64,
    pub a2: f64,
}

impl BiquadCoefficients {
    /// The identity filter that lets the signal pass unchanged.
    pub fn allpass() -> Self {
        Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
        }
    }

    /// Lowpass filter with the given cutoff frequency and Q factor
    pub fn lowpass(sample_rate: f64, cutoff: f64, q: f64) -> Self {
        let omega0 = 2.0 * std::f64::consts::PI * cutoff / sample_rate;
        let (sin_omega, cos_omega) = omega0.sin_cos();
        let alpha = sin_omega / (2.0 * q);
        let a0 = 1.0 + alpha;
        let a0_inv = 1.0 / a0;
        Self {
            b0: a0_inv * (1.0 - cos_omega) / 2.0,
            b1: a0_inv * (1.0 - cos_omega),
            b2: a0_inv * (1.0 - cos_omega) / 2.0,
            a1: a0_inv * (-2.0 * cos_omega),
            a2: a0_inv * (1.0 - alpha),
        }
    }
}

/// Biquadratic filter with four delay gates, based on https://www.w3.org/2011/audio/audio-eq-cookbook.html.
#[derive(Debug, Clone)]
pub struct Biquad {
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

impl Biquad {
    pub fn new() -> Self {
        Self {
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    /// Feed the next value through the filter using the given coefficients.
    pub fn step(&mut self, c: &BiquadCoefficients, input: f64) -> f64 {
        let output =
            c.b0 * input + c.b1 * self.x1 + c.b2 * self.x2 - c.a1 * self.y1 - c.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }
}

// Alternative form of the biquad filter:

// #[derive(Debug, Clone)]
// pub struct Biquad {
//     w1: f64,
//     w2: f64,
// }

// impl Biquad {
//     pub fn new() -> Self {
//         Self {
//             w1: 0.0,
//             w2: 0.0,
//         }
//     }

//     pub fn step(&mut self, c: &BiquadCoefficients, input: f64) -> f64 {
//         let w0 = input - c.a1 * self.w1 - c.a2 * self.w2;
//         let output = c.b0 * w0 + c.b1 * self.w1 + c.b2 * self.w2;
//         self.w2 = self.w1;
//         self.w1 = w0;
//         output
//     }
// }
