// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! Translate an abstract description of music into waveforms

use std::io;

use log::{info, trace};

use crate::note::{Note, Velocity};
use crate::output;
use crate::pianoroll::{PianoRoll, Time};
use crate::synth;
use crate::wave;
use std::path::Path;

pub struct Song {
    pub bpm: i64,
    pub notes: PianoRoll,
}

/// Play a song on the default speakers.
pub fn play(song: Song, outfile: Option<&Path>) -> io::Result<()> {
    let bpm = song.bpm;

    // hard-coded denominator of measures (for now)
    // One `1 / measure_denom` counts as one beat for the bpm.
    let measure_denom = 4;

    let sample_rate = 44100;
    let time_to_sample = |time: Time| {
        (time.numerator() * measure_denom * 60 * sample_rate / time.denominator() / bpm) as usize
    };
    let time_to_seconds = |time: Time| {
        time.numerator() as f64 / time.denominator() as f64 * measure_denom as f64 * 60.0 as f64
            / bpm as f64
    };

    let roll = &song.notes;
    // iterate the notes in the piano roll in order, and convert timing to samples
    let mut note_iter: std::iter::Peekable<_> = roll
        .iter()
        .map(|note| QueuedPlay {
            begin_sample: time_to_sample(note.start),
            end_sample: time_to_sample(note.start + note.duration),
            note: note.note,
            velocity: note.velocity,
        })
        .peekable();

    let max_samples = time_to_sample(roll.length()) + 2 * sample_rate as usize;

    info!("playing at {} bpm at {} Hz", bpm, sample_rate);
    info!(
        "total length {} samples ({:.2} seconds)",
        max_samples,
        time_to_seconds(roll.length()) + 2.0
    );

    // 10 ms buffer at 44100 Hz
    let buffer_size = 441;

    let mut synth = synth::test::TestSynth::new(sample_rate as f64);
    let target = match outfile {
        None => output::sox::SoxTarget::Play,
        Some(path) => output::sox::SoxTarget::File(path),
    };
    output::sox::with_sox(sample_rate as i32, target, |audio_stream| {
        let mut audio_buffer = AudioBuffer::new(buffer_size);
        let mut byte_buffer = vec![0u8; audio_buffer.byte_len()];

        // heap to keep track of currently playing notes
        let mut releases = std::collections::BinaryHeap::new();

        let mut samples_total = 0;
        while samples_total < max_samples {
            let window_start = samples_total;
            let window_end = samples_total + audio_buffer.len();

            audio_buffer.fill_zero();

            // process all notes that are due in the current window
            while let Some(note) = next_if(&mut note_iter, |n| n.begin_sample < window_end) {
                let handle =
                    synth.play_note(note.begin_sample - window_start, note.note, note.velocity);
                trace!(
                    "{:7}: play {:?} as {:?}",
                    note.begin_sample,
                    note.note,
                    handle
                );
                releases.push(QueuedRelease {
                    time: note.end_sample,
                    note_handle: handle,
                });
            }
            // process all note releases that are due in the current window
            // NOTE: must be done after processing the notes to play in order
            // to catch notes that last shorter than one buffer window.
            while let Some(release) = releases.peek() {
                if release.time < window_end {
                    trace!("{:7}: release {:?}", release.time, release.note_handle);
                    let release = releases.pop().unwrap();
                    synth.release_note(release.time - window_start, release.note_handle)
                } else {
                    break;
                }
            }

            synth.fill_buffer(audio_buffer.samples_mut());

            let n = audio_buffer.copy_bytes_to(&mut byte_buffer);
            assert_eq!(n, audio_buffer.len());
            audio_stream.write_all(&byte_buffer)?;
            samples_total += audio_buffer.len();
        }

        Ok(())
    })
}

fn next_if<Item, I: Iterator<Item = Item>, F: Fn(&Item) -> bool>(
    iter: &mut std::iter::Peekable<I>,
    predicate: F,
) -> Option<Item> {
    if let Some(preview) = iter.peek() {
        if predicate(preview) {
            return iter.next();
        }
    }
    None
}

pub struct QueuedPlay {
    begin_sample: usize,
    end_sample: usize,
    note: Note,
    velocity: Velocity,
}

pub struct QueuedRelease {
    time: usize,
    note_handle: synth::test::PlayHandle,
}

impl PartialEq for QueuedRelease {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl Eq for QueuedRelease {}

impl PartialOrd for QueuedRelease {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueuedRelease {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // the smallest release time is the largest queued release for the max heap
        other.time.cmp(&self.time)
    }
}

pub struct AudioBuffer {
    samples: Vec<wave::Stereo<f64>>,
}

#[allow(clippy::len_without_is_empty)]
impl AudioBuffer {
    pub fn new(sample_count: usize) -> Self {
        Self {
            samples: vec![
                wave::Stereo {
                    left: 0.0,
                    right: 0.0
                };
                sample_count
            ],
        }
    }

    pub fn fill_zero(&mut self) {
        self.samples
            .iter_mut()
            .for_each(|s| *s = wave::Stereo::new(0.0, 0.0));
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn byte_len(&self) -> usize {
        self.len() * 2 * std::mem::size_of::<f64>()
    }

    pub fn samples(&self) -> &[wave::Stereo<f64>] {
        &self.samples
    }

    pub fn samples_mut(&mut self) -> &mut [wave::Stereo<f64>] {
        &mut self.samples
    }

    /// Copy the stereo `f64` samples to bytes, interleaving the left and right samples.
    ///
    /// Could probably be implemented with some sort of unsafe transmute,
    /// but copying is safe and likely not the bottleneck.
    ///
    /// Returns the number of samples that were actually copied.
    /// Might be less than the number of input samples if the output buffer was not large enough.
    pub fn copy_bytes_to(&self, bytes: &mut [u8]) -> usize {
        let mut processed = 0;
        for (sample, target) in self.samples.iter().zip(bytes.chunks_exact_mut(16)) {
            target[0..8].copy_from_slice(&sample.left.to_le_bytes());
            target[8..16].copy_from_slice(&sample.right.to_le_bytes());
            processed += 1;
        }
        processed
    }
}
