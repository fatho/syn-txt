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
use std::path::PathBuf;

use log::{info, trace};
use structopt::StructOpt;

use crate::note::{Note, Velocity};
use crate::output;
use crate::song::{Instrument, Song, Track};
use crate::synth;
use crate::{rational::Rational, wave};
use std::{collections::BinaryHeap, path::Path};
use wave::{AudioBuffer, Stereo};

#[derive(Debug, StructOpt)]
#[structopt(name = "musicc", about = "Compiling code into music")]
struct Opt {
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,

    /// Output file (any sox-supported format). Music is played directly if not given.
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,

    /// Dump the description of the song generated from evaluating the code.
    #[structopt(long)]
    #[allow(clippy::option_option)]
    dump_description: Option<Option<PathBuf>>,
}

pub fn song_main<F: FnOnce() -> io::Result<crate::song::Song>>(compose: F) -> io::Result<()> {
    let opt: Opt = Opt::from_args();

    let level = match opt.verbose {
        0 => log::Level::Info,
        1 => log::Level::Debug,
        _ => log::Level::Trace,
    };
    simple_logger::init_with_level(level).unwrap();

    let dump_out = opt
        .dump_description
        .map(|path| path.unwrap_or_else(|| "/dev/stdout".into()));
    let song = compose()?;
    if let Some(dump_out_path) = dump_out {
        use std::io::Write;
        let mut f = std::fs::File::create(dump_out_path)?;
        writeln!(f, "{:?}", song)?;
    }
    play(song, opt.output.as_deref())
}

/// Play a song on the default speakers.
pub fn play(song: Song, outfile: Option<&Path>) -> io::Result<()> {
    let sample_rate = 44100;

    let sig = TimeSig {
        beats_per_minute: song.bpm,
        beat_unit: 4,
    };

    let mut players: Vec<_> = song
        .tracks
        .into_iter()
        .map(|track| TrackPlayer::new(sample_rate, sig, track))
        .collect();

    let max_samples =
        players.iter().map(|p| p.samples_total()).max().unwrap_or(0) + 2 * sample_rate as usize;

    info!("playing at {} bpm at {} Hz", song.bpm, sample_rate);
    info!(
        "total length {} samples ({:.2} seconds)",
        max_samples,
        max_samples as f64 / sample_rate as f64
    );

    // 10 ms buffer at 44100 Hz
    let buffer_size = 441;

    let target = match outfile {
        None => output::sox::SoxTarget::Play,
        Some(path) => output::sox::SoxTarget::File(path),
    };
    output::sox::with_sox(sample_rate as i32, target, |audio_stream| {
        let mut audio_buffer = AudioBuffer::new(buffer_size);
        let mut byte_buffer = vec![0u8; audio_buffer.byte_len()];

        let mut samples_total = 0;
        while samples_total < max_samples {
            audio_buffer.fill_zero();
            for player in players.iter_mut() {
                player.fill_buffer(audio_buffer.samples_mut());
            }

            let n = audio_buffer.copy_bytes_to(&mut byte_buffer);
            assert_eq!(n, audio_buffer.len());
            audio_stream.write_all(&byte_buffer)?;
            samples_total += audio_buffer.len();
        }

        Ok(())
    })
}

/// Time signature of the song, consisting of
/// - the number of beats per minute,
/// - the length of a single beat
/// Note that this omits the number of beats per bar,
/// which is not needed for computing time from beats.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TimeSig {
    /// How many beats per minute
    pub beats_per_minute: i64,
    /// The length of one beat is `1 / beat_unit`.
    pub beat_unit: i64,
}

impl TimeSig {
    pub fn seconds(&self, note_time: Rational) -> Rational {
        note_time * self.beat_unit * 60 / self.beats_per_minute
    }

    pub fn samples(&self, note_time: Rational, samples_per_second: i64) -> i64 {
        (self.seconds(note_time) * samples_per_second).round()
    }
}

pub struct TrackPlayer {
    /// The instrument on this track, there will be more choice eventually.
    instrument: synth::test::TestSynth,
    /// The notes that are played on this track
    notes: Vec<QueuedPlay>,
    /// The next note to be played
    next_note: usize,
    /// The currently active notes that are to be released in the future.
    note_releases: BinaryHeap<QueuedRelease>,
    /// How many samples were already played.
    samples_processed: usize,
    /// How long this track is in samples (measured until the end of the last note)
    samples_total: usize,
}

impl TrackPlayer {
    pub fn new(sample_rate: i64, time_sig: TimeSig, track: Track) -> Self {
        let instrument = match track.instrument {
            Instrument::TestSynth(params) => {
                synth::test::TestSynth::with_params(sample_rate as f64, params)
            }
        };
        let mut notes: Vec<_> = track
            .notes
            .into_iter()
            .map(|note| QueuedPlay {
                begin_sample: time_sig.samples(note.start, sample_rate) as usize,
                end_sample: time_sig.samples(note.start + note.duration, sample_rate) as usize,
                note: note.note,
                velocity: note.velocity,
            })
            .collect();
        // The notes must be sorted in the order they are played for `fill_buffer` to work correctly.
        notes.sort_by_key(|n| n.begin_sample);

        let samples_total = notes.iter().map(|p| p.end_sample).max().unwrap_or(0);

        Self {
            instrument,
            notes,
            next_note: 0,
            note_releases: BinaryHeap::new(),
            samples_processed: 0,
            samples_total,
        }
    }

    /// Sample where the last note is released.
    /// The instrument might still generate sound after this point.
    pub fn samples_total(&self) -> usize {
        self.samples_total
    }

    pub fn fill_buffer(&mut self, buffer: &mut [Stereo<f64>]) {
        // Compute start and end time of this buffer in samples
        let buffer_start = self.samples_processed;
        let buffer_end = self.samples_processed + buffer.len();

        // process all notes that are due in the current window
        while self.next_note < self.notes.len()
            && self.notes[self.next_note].begin_sample < buffer_end
        {
            let note = &self.notes[self.next_note];
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
        self.instrument.fill_buffer(buffer);
    }
}

/// A note that is queued to be played in the future.
pub struct QueuedPlay {
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
pub struct QueuedRelease {
    /// The number of the sample where the note stops playing.
    end_sample: usize,
    /// The handle to the note that is playing, needed for releasing the note.
    note_handle: synth::test::PlayHandle,
}

/// Queued releases are compared by their scheduled time.
impl PartialEq for QueuedRelease {
    fn eq(&self, other: &Self) -> bool {
        self.end_sample == other.end_sample
    }
}

impl Eq for QueuedRelease {}

impl PartialOrd for QueuedRelease {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Queued releases are compared by their scheduled time,
/// the higher the release time, the smaller the QueuedRelease (in order to use them in the standard binary heap).
impl Ord for QueuedRelease {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // the smallest release time is the largest queued release for the max heap
        other.end_sample.cmp(&self.end_sample)
    }
}
