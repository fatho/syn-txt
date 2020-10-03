// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! Exemplary implementation of a synthesizer, wielding waves like a pro.

use crate::automation::Expr;
use crate::envelope::*;
use crate::filter;
use crate::note::*;
use crate::oscillator::*;
use crate::tuning::*;
use crate::wave::*;

pub struct Wavinator {
    /// Samples per second rate of the generated audio signal.
    sample_rate: f64,
    /// Reference note and frequency, determining the pitch of all other notes.
    tuning: Tuning,

    /// The public parameters influencing the sound.
    /// TODO: these are eventually automatable
    parameters: Params,

    biquad: Stereo<filter::Biquad>,

    /// Number of samples already processed
    samples_processed: usize,

    /// Monotoneously increasing id used for identifying playing notes.
    next_play_handle: usize,
    active_notes: Vec<NoteState>,
}

/// Parameters of the synthesizer.
/// TODO: Add filter settings
#[derive(Debug)]
pub struct Params {
    /// Output gain of the synthesizer
    pub gain: Expr,

    /// Pan of the center unison voice
    pub pan: f64,

    /// Number of voices per note
    pub unison: usize,
    /// Maximum detune factor the outermost unison voices.
    pub unison_detune_cents: f64,
    /// Gain falloff exponent for the more detuned noises.
    pub unison_falloff: f64,

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
            pan: 0.0,
            unison: 3,
            unison_detune_cents: 3.0,
            unison_falloff: 0.0,
            wave_shape: WaveShape::SuperSaw,
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

/// Opaque handle indicating a playing voice.
#[derive(Debug, PartialEq, Eq)]
pub struct PlayHandle(usize);

impl Wavinator {
    pub fn new(sample_rate: f64) -> Self {
        Self::with_params(sample_rate, Params::default())
    }
    pub fn with_params(sample_rate: f64, params: Params) -> Self {
        Wavinator {
            parameters: params,
            tuning: Tuning::default(),
            biquad: Stereo {
                left: filter::Biquad::new(),
                right: filter::Biquad::new(),
            },
            // audio settings
            sample_rate,
            // internal state
            samples_processed: 0,
            next_play_handle: 0,
            active_notes: vec![],
        }
    }
}

impl Wavinator {
    fn next_play_handle(&mut self) -> PlayHandle {
        let h = PlayHandle(self.next_play_handle);
        self.next_play_handle += 1;
        h
    }
}

impl super::Instrument for Wavinator {
    type PlayHandle = PlayHandle;

    fn play_note(
        &mut self,
        sample_delay: usize,
        note: Note,
        velocity: Velocity,
    ) -> Self::PlayHandle {
        let frequency = self.tuning.frequency(note);
        let handle = self.next_play_handle();

        let midpoint = (self.parameters.unison as f64 - 1.0) / 2.0;
        let mut voices = (0..self.parameters.unison)
            .map(|index| {
                let offset = if self.parameters.unison > 1 {
                    (index as f64 - midpoint) / midpoint
                } else {
                    0.0
                };
                let detune = crate::util::from_cents(offset * self.parameters.unison_detune_cents);
                let pan = self.parameters.pan;
                Voice {
                    // normalized in the next step
                    gain: (-self.parameters.unison_falloff * offset.powi(2)).exp(),
                    pan,
                    oscillator: Oscillator::new(
                        self.parameters.wave_shape,
                        self.sample_rate,
                        frequency * detune,
                    ),
                }
            })
            .collect::<Vec<Voice>>();
        let total_gain: Stereo<f64> = voices
            .iter()
            .map(|v| Stereo::panned_mono(v.gain, v.pan))
            .sum();
        let normalized_gain = 1.0 / total_gain.left.max(total_gain.right);
        log::trace!("total unnormalized unison gain {:?}", total_gain);
        for voice in voices.iter_mut() {
            voice.gain *= velocity.as_f64() * normalized_gain;
        }

        self.active_notes.push(NoteState {
            handle: PlayHandle(handle.0),
            // state
            play_delay_samples: sample_delay,
            release_delay_samples: std::usize::MAX,
            voices,
            // volume
            envelope: self.parameters.envelope.instantiate(self.sample_rate),
        });
        handle
    }

    fn release_note(&mut self, sample_delay: usize, handle: Self::PlayHandle) {
        if let Some(voice) = self.active_notes.iter_mut().find(|v| v.handle == handle) {
            voice.release_delay_samples = voice.release_delay_samples.min(sample_delay);
        }
    }

    fn fill_buffer(&mut self, output: &mut [Stereo<f64>]) {
        for out_sample in output.iter_mut() {
            // Environment for automation
            let time_seconds = self.samples_processed as f64 / self.sample_rate;
            self.samples_processed += 1;
            let eval_env = &[time_seconds];

            let mut wave = Stereo::mono(0.0);
            let voice_count = self.active_notes.len();
            for voice_index in (0..voice_count).rev() {
                wave += self.active_notes[voice_index].sample();
                if self.active_notes[voice_index].envelope.faded() {
                    log::trace!(
                        "removing faded voice {:?}",
                        self.active_notes[voice_index].handle
                    );
                    self.active_notes.swap_remove(voice_index);
                }
            }

            // It's inefficient to compute these every sample, but once
            // we get to automation, we'd have to do that anyway.
            let filter_coefficients = self.parameters.filter.to_coefficients(self.sample_rate);
            wave = Stereo {
                left: self.biquad.left.step(&filter_coefficients, wave.left),
                right: self.biquad.right.step(&filter_coefficients, wave.right),
            };

            *out_sample += wave * self.parameters.gain.eval(eval_env).unwrap_or(0.0);
        }
    }
}

/// State needed for a playing note.
struct NoteState {
    /// Handle that has been handed out to the host of the synthesizer when
    /// this note was played, used for releasing it.
    handle: PlayHandle,
    /// Number of samples until the note starts
    play_delay_samples: usize,
    /// Number of samples until the note ends
    release_delay_samples: usize,
    /// The voices producing the sound of the note
    voices: Vec<Voice>,
    /// The envelope defining the volume shape of the note
    envelope: EvalADSR,
}

#[derive(Debug)]
struct Voice {
    pan: f64,
    gain: f64,
    oscillator: Oscillator,
}

impl Voice {
    fn sample(&mut self) -> Stereo<f64> {
        let mono = self.oscillator.next_sample() * self.gain;
        Stereo::panned_mono(mono, self.pan)
    }
}

impl NoteState {
    fn sample(&mut self) -> Stereo<f64> {
        if self.play_delay_samples > 0 {
            // the note has not started yet
            self.play_delay_samples -= 1;
            Stereo::mono(0.0)
        } else {
            if self.release_delay_samples > 0 {
                self.release_delay_samples -= 1;
            } else if !self.envelope.released() {
                log::trace!("released {:?}", self.handle);
                self.envelope.release();
            }

            let result: Stereo<f64> = self.voices.iter_mut().map(Voice::sample).sum();
            let envelope = self.envelope.step();

            result * envelope
        }
    }
}
