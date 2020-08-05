// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

use crate::{
    instrument::Instrument,
    note::{Note, Velocity},
    song::PlayedNote,
};
use std::collections::BinaryHeap;

use log::trace;

pub struct InstrumentSource<I: Instrument> {
    instrument: I,
    /// The notes that are played on this track
    play_queue: Vec<QueuedPlay>,
    /// The next note to be played
    next_note: usize,
    /// The currently active notes that are to be released in the future.
    note_releases: BinaryHeap<QueuedRelease<I::PlayHandle>>,
    /// How many samples were already played.
    samples_processed: usize,
}

impl<I: Instrument> InstrumentSource<I> {
    pub fn new(
        sample_rate: i64,
        time_sig: crate::song::TimeSig,
        instrument: I,
        notes: Vec<PlayedNote>,
    ) -> Self {
        let mut play_queue: Vec<_> = notes
            .into_iter()
            .map(|note| QueuedPlay {
                begin_sample: time_sig.samples(note.start, sample_rate) as usize,
                end_sample: time_sig.samples(note.start + note.duration, sample_rate) as usize,
                note: note.note,
                velocity: note.velocity,
            })
            .collect();
        // The notes must be sorted in the order they are played for `fill_buffer` to work correctly.
        play_queue.sort_by_key(|n| n.begin_sample);

        Self {
            instrument,
            play_queue,
            next_note: 0,
            note_releases: BinaryHeap::new(),
            samples_processed: 0,
        }
    }
}

impl<I: Instrument> super::Node for InstrumentSource<I> {
    fn num_inputs(&self) -> usize {
        0
    }
    fn num_outputs(&self) -> usize {
        1
    }
    fn render(&mut self, rio: &super::RenderIo) {
        let mut audio_buffer = rio.output(0);
        audio_buffer.fill_zero();

        // Compute start and end time of this buffer in samples
        let buffer_start = self.samples_processed;
        let buffer_end = self.samples_processed + audio_buffer.len();

        // process all notes that are due in the current window
        while self.next_note < self.play_queue.len()
            && self.play_queue[self.next_note].begin_sample < buffer_end
        {
            let note = &self.play_queue[self.next_note];
            let handle = self.instrument.play_note(
                note.begin_sample - buffer_start,
                note.note,
                note.velocity,
            );
            trace!(
                "{:7}: play {:?} as {:?}",
                note.begin_sample,
                note.note,
                handle
            );
            self.note_releases.push(QueuedRelease {
                end_sample: note.end_sample,
                note_handle: handle,
            });
            self.next_note += 1;
        }
        // process all note releases that are due in the current window
        // NOTE: must be done after processing the notes to play in order
        // to catch notes that last shorter than one buffer window.
        while let Some(release) = self.note_releases.peek() {
            if release.end_sample < buffer_end {
                trace!(
                    "{:7}: release {:?}",
                    release.end_sample,
                    release.note_handle
                );
                let release = self.note_releases.pop().unwrap();
                self.instrument
                    .release_note(release.end_sample - buffer_start, release.note_handle)
            } else {
                break;
            }
        }

        self.samples_processed = buffer_end;
        self.instrument.fill_buffer(audio_buffer.samples_mut());
    }
}

/// A note that is queued to be played in the future.
struct QueuedPlay {
    /// The sample number where the note starts playing.
    begin_sample: usize,
    /// The sample number where the note stops playing.
    /// If the end sample lies before the begin sample, the note is note played.
    end_sample: usize,
    /// The note that is played.
    note: Note,
    /// How fast the note is played.
    velocity: Velocity,
}

/// A note that is currently played and scheduled to be released in the future.
struct QueuedRelease<H> {
    /// The number of the sample where the note stops playing.
    end_sample: usize,
    /// The handle to the note that is playing, needed for releasing the note.
    note_handle: H,
}

/// Queued releases are compared by their scheduled time.
impl<H> PartialEq for QueuedRelease<H> {
    fn eq(&self, other: &Self) -> bool {
        self.end_sample == other.end_sample
    }
}

impl<H> Eq for QueuedRelease<H> {}

impl<H> PartialOrd for QueuedRelease<H> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Queued releases are compared by their scheduled time,
/// the higher the release time, the smaller the QueuedRelease (in order to use them in the standard binary heap).
impl<H> Ord for QueuedRelease<H> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // the smallest release time is the largest queued release for the max heap
        other.end_sample.cmp(&self.end_sample)
    }
}
