//! Translate an abstract description of music into waveforms

use std::io;

use log::info;

use crate::note::NoteAction;
use crate::output;
use crate::pianoroll::{PianoRoll, Time};
use crate::render;
use crate::synth;
use crate::wave;

/// Play a piano roll on the default speakers.
pub fn play(roll: PianoRoll) -> io::Result<()> {
    // TODO: make bpm configurable
    let bpm = 120;

    // hard-coded denominator of measures (for now)
    // One `1 / measure_denom` counts as one beat for the bpm.
    let measure_denom = 4;

    let sample_rate = 44100;
    let time_to_sample = |time: Time| {
        (time.numerator() * measure_denom * 60 * sample_rate / time.denominator() / bpm) as usize
    };
    let time_to_seconds = |time: Time| {
        time.numerator() as f64 / time.denominator() as f64 * measure_denom as f64 * 60.0 as f64 / bpm as f64
    };

    let mut events = Vec::new();

    for pn in roll.iter() {
        let note = pn.note;
        let velocity = pn.velocity;
        let play = synth::Event {
            time: time_to_sample(pn.start),
            action: NoteAction::Play { note, velocity },
        };
        let release = synth::Event {
            time: time_to_sample(pn.start + pn.duration),
            action: NoteAction::Release { note },
        };

        events.push(play);
        events.push(release);
    }

    events.sort_by_key(|evt| evt.time);

    let max_samples = time_to_sample(roll.length()) + sample_rate as usize;

    info!("playing {} events at {} bpm at {} Hz", events.len(), bpm, sample_rate);
    info!("total length {} samples ({:.2} seconds)", max_samples, time_to_seconds(roll.length()) + 1.0);

    // 10 ms buffer at 44100 Hz
    let buffer_size = 441;

    let synth = synth::test::TestSynth::new(0, sample_rate as f64);
    let mut player = render::SynthPlayer::new(synth, &events);

    crate::output::sox::with_sox_player(sample_rate as i32, |audio_stream| {
        let mut audio_buffer: Vec<wave::Stereo<f64>> = vec![
            wave::Stereo {
                left: 0.0,
                right: 0.0
            };
            buffer_size
        ];
        let mut byte_buffer = vec![0u8; buffer_size * 2 * 8];

        let mut samples_total = 0;
        while samples_total < max_samples {
            audio_buffer
                .iter_mut()
                .for_each(|s| *s = wave::Stereo::new(0.0, 0.0));
            player.generate(&mut audio_buffer);
            let n = output::copy_f64_bytes(&audio_buffer, &mut byte_buffer);
            assert_eq!(n, audio_buffer.len());
            audio_stream.write_all(&byte_buffer)?;
            samples_total += audio_buffer.len();
        }

        Ok(())
    })
}
