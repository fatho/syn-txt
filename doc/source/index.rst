A text-based synthesizer
========================

.. toctree::
  :maxdepth: 2
  :caption: Contents:


The goal is for this to be a declarative, text-based synthesizer/audio workstation.
At the moment, it is very much work in progress. The repository contains a roadmap_
with a rough outline of what is planned for the near future.

However, there is already a working prototype that includes

However, there is already a working prototype that includes
- a simple textual format for writing melodies
- a small embedded DSL for constructing simple songs
- a synthesizer with multiple oscillators, several waveforms, and simple filters
- a convenient function providing the main function for an executable generating the song (examples_)

Eventually, the idea is that song descriptions get a serialization format,
making it possible to generate them from other programming languages as well.

Example
-------

To give you an example, a simple melody on the built-in synthesizer with the default settings
can be generated with the following code snippet (demo.rs_):

.. code-block:: rust

    use syn_txt::melody::parse_melody;
    use syn_txt::play;
    use syn_txt::song::*;
    use syn_txt::synth;

    fn main() -> std::io::Result<()> {
        play::song_main(|| {
            let song = Song {
                bpm: 128,
                tracks: vec![
                    Track {
                        instrument: Instrument::TestSynth(synth::test::Params::default()),
                        notes: parse_melody(r"
                            a3- c4- a3- d4- a3- e4- a3- d4-
                            a3- c4- a3- d4- a3- e4- a3- d4-
                            { { c4- d4- e4- d4- } a3+ } { { c4- d4- e4- d4- } a3+ }
                            { a3 c4 } { a3 d4 } { a3 c4 } r
                        ").unwrap(),
                    }
                ],
            };
            Ok(song)
        })
    }


Compiling this program to a waveform results in this audio:

.. raw:: html

   <audio controls="controls">
         <source src="_static/demo.ogg" type="audio/ogg">
         Your browser does not support the <code>audio</code> element. 
   </audio>

Build Instructions
------------------

The project is written in Rust_ and all tooling is pulled in via Nix_,
although just having rustc and Cargo available *should* just work.
That said, you can simply run `nix-shell` in the repository root to drop
into a shell where all dependencies are avaiable.
Then, the normal Cargo_ workflow applies.

.. code-block:: bash

   # Play the included example melody
   cargo run --example demo

At the moment, it depends on spawning a sox_ subprocess and piping the audio data to it
for actually playing sound.
If everything worked, it should produce something similar to the following audio snippet:

.. raw:: html

   <audio controls="controls">
         <source src="_static/demo.ogg" type="audio/ogg">
         Your browser does not support the <code>audio</code> element. 
   </audio>

.. _demo.rs: https://github.com/fatho/syn-txt/blob/master/examples/demo.rs
.. _roadmap: https://github.com/fatho/syn-txt/blob/master/planning/roadmap.md
.. _examples: https://github.com/fatho/syn-txt/blob/master/examples/
.. _Rust: https://www.rust-lang.org/
.. _Nix: https://nixos.org/nix/
.. _Cargo: https://doc.rust-lang.org/cargo/
.. _sox: http://sox.sourceforge.net/


License
-------

The project is free software licensed under the `GNU General Public License Version 3`_.

.. _GNU General Public License Version 3: https://www.gnu.org/licenses/gpl-3.0.en.html

..
  Indices and tables
  ==================

  * :ref:`genindex`
  * :ref:`search`
