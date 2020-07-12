use syn_txt::note;
use syn_txt::play;
use syn_txt::rational::Rational;
use syn_txt::song::*;
use syn_txt::synth;

use std::io;

#[rustfmt::skip]
fn main() -> io::Result<()> {
    play::song_main(|| {
        let velocity = note::Velocity::from_f64(1.0);
        let eigth = Rational::new(1, 8);
        let song = Song {
            bpm: 128,
            tracks: vec![
                Track {
                    instrument: Instrument::TestSynth(synth::test::Params::default()),
                    notes: vec![
                        PlayedNote { start: 0 * eigth, duration: eigth, note: note::Note::from_midi(45),  velocity  },
                        PlayedNote { start: 1 * eigth, duration: eigth, note: note::Note::from_midi(48),  velocity  },
                        PlayedNote { start: 2 * eigth, duration: eigth, note: note::Note::from_midi(45),  velocity  },
                        PlayedNote { start: 3 * eigth, duration: eigth, note: note::Note::from_midi(50),  velocity  },
                        PlayedNote { start: 4 * eigth, duration: eigth, note: note::Note::from_midi(45),  velocity  },
                        PlayedNote { start: 5 * eigth, duration: eigth, note: note::Note::from_midi(52),  velocity  },
                        PlayedNote { start: 6 * eigth, duration: eigth, note: note::Note::from_midi(45),  velocity  },
                        PlayedNote { start: 7 * eigth, duration: eigth, note: note::Note::from_midi(50),  velocity  },
                    ],
                }
            ],
        };
        Ok(song)
    })
}
