//! Example demonstrating how to use the low-level synth interface.

use std::io;
use std::iter;

use syn_txt;
use syn_txt::note::*;
use syn_txt::synth;
use syn_txt::wave;

struct Composer {
    events: Vec<synth::Event>,
    last_note_start: f64,
    last_note_end: f64,
    bpm: i32,
    measure: i32,
    sample_rate: i32,
}

impl Composer {
    fn at(&self, t: f64) -> usize {
        ((t * self.measure as f64) / self.bpm as f64 * 60.0 * self.sample_rate as f64).floor() as usize
    }

    fn play_at(&mut self, t: f64, duration: f64, note: Note) {
        let play = synth::Event {
            time: self.at(t),
            action: NoteAction::Play { note, velocity: Velocity(127) },
        };
        let release = synth::Event {
            time: self.at(t + duration),
            action: NoteAction::Release { note },
        };
        self.events.push(play);
        self.events.push(release);
        self.last_note_start = t;
        self.last_note_end = t + duration;
    }

    fn play_after(&mut self, duration: f64, note: Note) {
        self.play_at(self.last_note_end, duration, note)
    }
}

fn main() -> io::Result<()> {
    let mut comp = Composer {
        events: Vec::new(),
        last_note_start: 0.0,
        last_note_end: 0.0,
        bpm: 128,
        measure: 4,
        sample_rate: 44100,
    };

    for _ in 0..8 {
        comp.play_after(0.125, Note::named(NoteName::E, NoteOffset::Base, 4).unwrap());
        comp.play_after(0.125, Note::named(NoteName::A, NoteOffset::Base, 3).unwrap());
    }
    for _ in 0..8 {
        comp.play_after(0.125, Note::named(NoteName::F, NoteOffset::Base, 4).unwrap());
        comp.play_after(0.125, Note::named(NoteName::A, NoteOffset::Base, 3).unwrap());
    }

    comp.events.sort_by_key(|evt| evt.time);

    for evt in comp.events.iter() {
        println!("{:?}", evt);
    }

    let max_samples = comp.at(comp.last_note_end) + comp.sample_rate as usize;

    let mut synth = synth::test::TestSynth::new(0, wave::SamplerInfo {
        sample_rate: comp.sample_rate,
        buffer_size: 44100,
    });

    syn_txt::output::sox::with_sox_player(comp.sample_rate, |audio_stream| {
        let mut audio: Vec<wave::Stereo<f64>> = vec![wave::Stereo { left: 0.0, right: 0.0 }; max_samples];

        synth.play(&comp.events, &mut audio);

        let bytes: Vec<u8> = audio.iter()
            .flat_map(|frame| iter::once(frame.left).chain(iter::once(frame.right)))
            .flat_map(|sample|{
                let bytes: Vec<u8> = sample.to_le_bytes()[..].into();
                bytes.into_iter()
            })
            .collect();

        audio_stream.write(&bytes)?;
        Ok(())
    })
}
