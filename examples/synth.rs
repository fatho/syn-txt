//! Example demonstrating how to use the low-level synth interface.

use std::io;
use std::iter;

use syn_txt;
use syn_txt::note::*;
use syn_txt::pianoroll::{PianoRoll, Time};
use syn_txt::synth::{self, Synthesizer};
use syn_txt::wave;
use syn_txt::render;

fn main() -> io::Result<()> {
    let eigth = Time::nth(8);
    let vel = Velocity::from_f64(0.5);

    let mut roll1 = PianoRoll::new();
    let mut roll2 = PianoRoll::new();

    for _ in 0..8 {
        roll1.add_after(Note::named(NoteName::E, NoteOffset::Base, 4), eigth, vel);
        roll1.add_stack(Note::named(NoteName::E, NoteOffset::Base, 5), eigth, vel);
        roll1.add_after(Note::named(NoteName::A, NoteOffset::Base, 3), eigth, vel);
        roll1.add_stack(Note::named(NoteName::A, NoteOffset::Base, 4), eigth, vel);
    }
    for _ in 0..8 {
        roll2.add_after(Note::named(NoteName::F, NoteOffset::Base, 4), eigth, vel);
        roll2.add_stack(Note::named(NoteName::F, NoteOffset::Base, 5), eigth, vel);
        roll2.add_after(Note::named(NoteName::A, NoteOffset::Base, 3), eigth, vel);
        roll2.add_stack(Note::named(NoteName::A, NoteOffset::Base, 4), eigth, vel);
    }

    let mut roll = PianoRoll::new();
    roll.append(&roll1);
    roll.append(&roll2);
    roll.append(&roll1);

    let bpm = 120;
    let measure = 4;
    let sample_rate = 44100;
    let time_to_sample = |time: Time| {
        (time.numerator() * measure * 60 * sample_rate / time.denominator() / bpm) as usize
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

    for evt in events.iter() {
        println!("{:?}", evt);
    }

    let max_samples = time_to_sample(roll.length()) + sample_rate as usize;

    let buffer_size = 441;

    let synth = synth::test::TestSynth::new(0, sample_rate as f64);
    let mut player = render::SynthPlayer::new(synth, &events);


    syn_txt::output::sox::with_sox_player(sample_rate, |audio_stream| {
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
            audio_buffer.iter_mut().for_each(|s| *s = wave::Stereo::new(0.0, 0.0));
            player.generate(&mut audio_buffer);
            copy_audio_bytes(&audio_buffer, &mut byte_buffer);
            audio_stream.write(&byte_buffer)?;
            samples_total += audio_buffer.len();
        }
        Ok(())
    })
}

fn copy_audio_bytes(audio: &[wave::Stereo<f64>], bytes: &mut [u8]) {
    assert!(audio.len() * 2 * 8 <= bytes.len());
    let mut offset = 0;
    for sample in audio {
        bytes[offset..offset + 8].copy_from_slice(&sample.left.to_le_bytes());
        bytes[offset + 8..offset + 16].copy_from_slice(&sample.left.to_le_bytes());
        offset += 16;
    }
}
