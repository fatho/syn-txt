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

//! Exemplary implementation of a synthesizer, wielding waves like a pro.

use crate::automation::{BuiltInValues, Expr};
use crate::envelope::*;
use crate::filter;
use crate::oscillator::*;
use crate::tuning::*;
use crate::wave::*;
use syntxt_core::note::*;

use super::polyphonic::*;

use log::trace;

pub type Wavinator = Poly<Sampler>;

/// Parameters of the synthesizer.
/// TODO: Add filter settings
#[derive(Debug)]
pub struct Params {
    /// Output gain of the synthesizer
    pub gain: Expr,

    /// Pan of the center unison voice
    pub pan: Expr,

    /// Number of voices per note
    pub unison: usize,
    /// Maximum detune factor the outermost unison voices.
    pub unison_detune_cents: f64,
    /// The larger the spread, the more evenly the unison voices contribute to the final sound,
    /// the smaller the spread, the more the center frequency dominates.
    pub unison_spread: f64,

    /// Oscillator shape
    pub wave_shape: WaveShape,

    /// Evenlope for played notes
    pub envelope: ADSR,

    /// The type of filter to apply to the synthesizer output.
    /// Currently limited to biquadratic filters.
    pub filter: filter::BiquadType,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            gain: Expr::Const(1.0),
            pan: Expr::Const(0.0),
            unison: 1,
            unison_detune_cents: 3.0,
            unison_spread: 1.0,
            wave_shape: WaveShape::Sine,
            envelope: ADSR {
                attack: 0.01,
                decay: 0.0,
                sustain: 1.0,
                release: 0.1,
            },
            filter: filter::BiquadType::Allpass,
        }
    }
}

/// State needed for a playing note.
pub struct Sampler {
    /// The voices producing the sound of the note
    voices: Vec<Phase>,
    /// The envelope defining the volume shape of the note
    envelope: EvalADSR,
    /// Filter for this note
    biquad: Stereo<filter::Biquad>,
    /// Index of the center voice (which may be in between two voices)
    midpoint: f64,
    /// Frequency of the center voice
    center_freq: f64,
    /// The gain resulting from the initial note velocity
    velocity_gain: f64,
    /// Duration of the current note in samples so far
    playtime_samples: usize,
}

impl NoteSampler for Sampler {
    type Params = Params;

    fn new(note: Note, velocity: Velocity, sample_rate: f64, params: &Self::Params) -> Self {
        // NOTE: number of voices can only be determined at note creation at the moment
        Self {
            voices: std::iter::repeat(Phase::ZERO)
                .take(params.unison.max(1))
                .collect(),
            envelope: params.envelope.instantiate(sample_rate),
            biquad: Stereo {
                left: filter::Biquad::new(),
                right: filter::Biquad::new(),
            },
            // Compute the index of the center voice (which may be in between two voices).
            // The number of voices should be odd, so that one voice is playing the actual note frequency.
            midpoint: (params.unison as f64 - 1.0) / 2.0,
            center_freq: Tuning::default().frequency(note),
            // the velocity simply controls the volume of the note
            velocity_gain: velocity.as_f64(),
            playtime_samples: 0,
        }
    }

    fn sample(
        &mut self,
        global_sample_count: usize,
        sample_rate: f64,
        params: &Self::Params,
    ) -> Option<Stereo<f64>> {
        if self.envelope.faded() {
            return None;
        }
        let builtins = BuiltInValues {
            global_time_seconds: global_sample_count as f64 / sample_rate,
            note_time_seconds: self.playtime_samples as f64 / sample_rate,
        };

        let mut value = 0.0;
        let mut value_gain_sum = 0.0;
        let spread_squared = params.unison_spread.max(0.001).powi(2);
        for (index, voice) in self.voices.iter_mut().enumerate() {
            let delta = index as f64 - self.midpoint;

            let gain = (-delta * delta / (2.0 * spread_squared)).exp();

            value += params.wave_shape.eval(*voice) * gain;
            value_gain_sum += gain;

            let detune = syntxt_core::util::from_cents(params.unison_detune_cents * delta);
            let frequency = detune * self.center_freq;
            *voice = voice.step_frequency(frequency, sample_rate);
        }

        let envelope_gain = self.envelope.step();
        let instrument_gain = params.gain.eval(&builtins, &[]).unwrap_or(0.0);
        let correction_gain = value_gain_sum.recip();

        trace!(
            "e = {}, i = {}, c = {}",
            envelope_gain,
            instrument_gain,
            correction_gain
        );

        let final_gain = instrument_gain * envelope_gain * self.velocity_gain * correction_gain;

        let pan = params.pan.eval(&builtins, &[]).unwrap_or(0.0);
        let output = final_gain * Stereo::panned_mono(value, pan);

        // TODO: make filter automatable
        let filter_coeffs = params.filter.to_coefficients(sample_rate);
        let filtered_output = Stereo {
            left: self.biquad.left.step(&filter_coeffs, output.left),
            right: self.biquad.right.step(&filter_coeffs, output.right),
        };
        self.playtime_samples += 1;
        Some(filtered_output)
    }

    fn release(&mut self) {
        self.envelope.release()
    }
}
