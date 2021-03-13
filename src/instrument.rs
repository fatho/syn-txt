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
    note::{Note, Velocity},
    wave::Stereo,
};

pub mod polyphonic;
pub mod wavinator;

/// Interface of an interactive instrument.
///
/// TODO: Add support for automation.
pub trait Instrument {
    type PlayHandle: std::fmt::Debug;

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
    fn play_note(
        &mut self,
        sample_delay: usize,
        note: Note,
        velocity: Velocity,
    ) -> Self::PlayHandle;

    /// Release a note that was previously played using `play_note`.
    /// If a note has already been released, this has no effect.
    /// If a note has only been marked for release, the shorter release time is used.
    fn release_note(&mut self, sample_delay: usize, handle: Self::PlayHandle);

    /// Add the waveforms generated by the currently playing notes onto the buffer.
    fn fill_buffer(&mut self, output: &mut [Stereo<f64>]);
}
