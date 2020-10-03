// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! Prototype for a polyphonic instrument where each note can be played individually.

use crate::note::*;
use crate::wave::*;

pub struct Poly<Sampler: NoteSampler> {
    /// Samples per second rate of the generated audio signal.
    sample_rate: f64,

    /// The public parameters influencing the sound.
    parameters: Sampler::Params,

    /// Number of samples already processed
    samples_processed: usize,

    /// Monotoneously increasing id used for identifying playing notes.
    next_play_handle: usize,
    active_notes: Vec<NoteState<Sampler>>,
}

pub trait NoteSampler {
    type Params: Default;

    /// Initialize the sampler for a single note.
    fn new(note: Note, velocity: Velocity, sample_rate: f64, params: &Self::Params) -> Self;

    /// Generate the next sample for a note.
    /// Return `None` if the note has faded.
    /// Once `None` was returned, `sample` with never be called again.
    fn sample(&mut self, sample_rate: f64, params: &Self::Params) -> Option<Stereo<f64>>;

    /// Called before the sample when the note is first released.
    fn release(&mut self);
}

/// Opaque handle indicating a playing voice.
#[derive(Debug, PartialEq, Eq)]
pub struct PlayHandle(usize);

impl<Sampler: NoteSampler> Poly<Sampler> {
    pub fn new(sample_rate: f64) -> Self {
        Self::with_params(sample_rate, Sampler::Params::default())
    }
    pub fn with_params(sample_rate: f64, params: Sampler::Params) -> Self {
        Poly {
            parameters: params,
            // audio settings
            sample_rate,
            // internal state
            samples_processed: 0,
            next_play_handle: 0,
            active_notes: vec![],
        }
    }
}

impl<Sampler: NoteSampler> Poly<Sampler> {
    fn next_play_handle(&mut self) -> PlayHandle {
        let h = PlayHandle(self.next_play_handle);
        self.next_play_handle += 1;
        h
    }
}

impl<Sampler: NoteSampler> super::Instrument for Poly<Sampler> {
    type PlayHandle = PlayHandle;

    fn play_note(
        &mut self,
        sample_delay: usize,
        note: Note,
        velocity: Velocity,
    ) -> Self::PlayHandle {
        let handle = self.next_play_handle();

        self.active_notes.push(NoteState {
            handle: PlayHandle(handle.0),
            // state
            play_delay_samples: sample_delay,
            release_delay_samples: std::usize::MAX,
            sampler: NoteSampler::new(note, velocity, self.sample_rate, &self.parameters),
            released: false,
        });
        handle
    }

    fn release_note(&mut self, sample_delay: usize, handle: Self::PlayHandle) {
        if let Some(voice) = self.active_notes.iter_mut().find(|v| v.handle == handle) {
            voice.release_delay_samples = voice
                .release_delay_samples
                .min(sample_delay)
                .max(voice.play_delay_samples);
        }
    }

    fn fill_buffer(&mut self, output: &mut [Stereo<f64>]) {
        for out_sample in output.iter_mut() {
            let mut wave = Stereo::mono(0.0);
            let voice_count = self.active_notes.len();
            for voice_index in (0..voice_count).rev() {
                if let Some(value) =
                    self.active_notes[voice_index].sample(self.sample_rate, &self.parameters)
                {
                    wave += value;
                } else {
                    log::trace!(
                        "removing faded voice {:?}",
                        self.active_notes[voice_index].handle
                    );
                    self.active_notes.swap_remove(voice_index);
                }
            }

            *out_sample += wave
        }
        self.samples_processed += output.len();
    }
}

/// State needed for a playing note.
struct NoteState<Sampler> {
    /// Handle that has been handed out to the host of the synthesizer when
    /// this note was played, used for releasing it.
    handle: PlayHandle,
    /// Number of samples until the note starts
    play_delay_samples: usize,
    /// Number of samples until the note ends
    release_delay_samples: usize,
    /// The voices producing the sound of the note
    sampler: Sampler,
    /// Whether the note was already released
    released: bool,
}

impl<Sampler: NoteSampler> NoteState<Sampler> {
    fn sample(&mut self, sample_rate: f64, params: &Sampler::Params) -> Option<Stereo<f64>> {
        if self.play_delay_samples > 0 {
            // the note has not started yet
            self.play_delay_samples -= 1;
            self.release_delay_samples -= 1;
            Some(Stereo::mono(0.0))
        } else {
            if self.release_delay_samples > 0 {
                self.release_delay_samples -= 1;
            } else if !self.released {
                log::trace!("released {:?}", self.handle);
                self.released = true;
                self.sampler.release();
            }

            self.sampler.sample(sample_rate, params)
        }
    }
}
