// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! Exemplary implementation of a synthesizer.

use super::envelope::*;
use super::oscillator::*;
use super::tuning::*;
use super::filter;

use crate::note::*;
use crate::wave::*;

pub struct TestSynth {
    /// Samples per second rate of the generated audio signal.
    sample_rate: f64,
    /// Reference note and frequency, determining the pitch of all other notes.
    tuning: Tuning,

    /// Evenlope for played notes
    envelope: ADSR,

    /// Output gain of the synthesizer
    gain: f64,

    /// Pan of the center unison voice
    pan: f64,

    /// Number of voices per note
    unison: usize,
    /// Maximum detune factor the outermost unison voices.
    unison_detune_cents: f64,
    /// Gain falloff exponent for the more detuned noises.
    unison_falloff: f64,

    // low-pass filter
    filter_coefficients: filter::BiquadCoefficients,
    biquad: Stereo<filter::Biquad>,

    /// Monotoneously increasing id used for identifying playing notes.
    next_play_handle: usize,
    active_notes: Vec<NoteState>,
}

/// Opaque handle indicating a playing voice.
#[derive(Debug, PartialEq, Eq)]
pub struct PlayHandle(usize);

impl TestSynth {
    pub fn new(sample_rate: f64) -> Self {
        let filter_coefficients = filter::BiquadCoefficients::lowpass(sample_rate, 2000.0, 0.7071);
        TestSynth {
            // voice settings
            pan: 0.0,
            unison: 3,
            unison_detune_cents: 15.0,
            unison_falloff: 0.0,
            tuning: Tuning::default(),
            // volume
            envelope: ADSR {
                attack: 0.01,
                decay: 0.0,
                sustain: 1.0,
                release: 0.1,
            },
            gain: 0.5,
            filter_coefficients,
            biquad: Stereo {
                left: filter::Biquad::new(),
                right: filter::Biquad::new(),
            },
            // audio settings
            sample_rate,
            // internal state
            next_play_handle: 0,
            active_notes: vec![],
        }
    }
}

impl TestSynth {
    /// Play a note on the synthesizer, starting `sample_delay` samples into
    /// the next `fill_buffer` call. It will also work if the next `fill_buffer`
    /// call produces fewer samples, but the playing note will already occupy
    /// resources. It is therefore a good idea to only call `play_note` just
    /// before the `fill_buffer` call where the note starts.
    ///
    /// This function returns a `PlayHandle` that can be used for notifying the
    /// synthesizer about a note that has been released.
    /// Any note with a non-zero sustain level in its envelope will keep playing
    /// indefinitely until released with `release_note`.
    pub fn play_note(&mut self, sample_delay: usize, note: Note, velocity: Velocity) -> PlayHandle {
        let frequency = self.tuning.frequency(note);
        let handle = self.next_play_handle();

        let midpoint = (self.unison as f64 - 1.0) / 2.0;
        let mut voices = (0..self.unison)
            .map(|index| {
                let offset = if self.unison > 1 {
                    (index as f64 - midpoint) / midpoint
                } else {
                    0.0
                };
                let detune = crate::util::from_cents(offset * self.unison_detune_cents);
                let pan = self.pan;
                Voice {
                    // normalized in the next step
                    gain: (-self.unison_falloff * offset.powi(2)).exp(),
                    pan,
                    oscillator: Oscillator::new(
                        WaveShape::SuperSaw,
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
            envelope: self.envelope.instantiate(self.sample_rate),
        });
        handle
    }

    /// Release a note that was previously played using `play_note`.
    /// If a note has already been released, this has no effect.
    /// If a note has only been marked for release, the shorter release time is used.
    pub fn release_note(&mut self, sample_delay: usize, handle: PlayHandle) {
        if let Some(voice) = self.active_notes.iter_mut().find(|v| v.handle == handle) {
            voice.release_delay_samples = voice.release_delay_samples.min(sample_delay);
        }
    }

    fn next_play_handle(&mut self) -> PlayHandle {
        let h = PlayHandle(self.next_play_handle);
        self.next_play_handle += 1;
        h
    }

    /// Add the waveforms generated by the currently playing notes onto the buffer.
    pub fn fill_buffer(&mut self, output: &mut [Stereo<f64>]) {
        for out_sample in output.iter_mut() {
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
            wave *= self.gain;
            wave = Stereo {
                left: self.biquad.left.step(&self.filter_coefficients, wave.left),
                right: self.biquad.right.step(&self.filter_coefficients, wave.right),
            };

            *out_sample += wave;
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
