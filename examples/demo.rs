use syn_txt::play;
use syn_txt::song::*;
use syn_txt::melody::parse_melody;
use syn_txt::synth;

use std::io;

#[rustfmt::skip]
fn main() -> io::Result<()> {
    play::song_main(|| {
        let song = Song {
            bpm: 128,
            tracks: vec![
                Track {
                    instrument: Instrument::TestSynth(synth::test::Params::default()),
                    notes: parse_melody(r"
                        a3- c4- a3- d4- a3- e4- a3- d4-
                        a3- c4- a3- d4- a3- e4- a3- d4-
                        a3-. c4-- c4-- r-- a3-. d4-- d4-- r--
                        a3-. e4-- e4-- r-- a3-. c4-- c4-- r--
                        a3-. e4-- d#4-- d4-- c#4-- c4-- b3-- a#3-- a3-- r-- r
                    ").unwrap(),
                }
            ],
        };
        Ok(song)
    })
}
