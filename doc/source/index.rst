A text-based synthesizer
========================

.. toctree::
  :maxdepth: 2
  :caption: Contents:


The goal is for this to be a declarative, text-based synthesizer/audio workstation.
At the moment, it is very much work in progress. The repository contains a roadmap_
with a rough outline of what is planned for the near future.

However, there is already a working prototype that includes

- an interpreter for a very small subset of a scheme-like language
  that can be used for defining piano rolls (examples_)
- a synthesizer turning notes into waveforms
- an interpreter for that language to play piano rolls on the built-in synthesizer

Example
-------

To give you an example, a few simple chords can be generated with the
following code snippet (chords.syn_):

.. code-block:: scheme

  (define melody
      (begin
          (define (chord length a b c)
              (piano-roll/stack
                  (note :pitch a :length length :velocity (/ 1.0 3))
                  (note :pitch b :length length :velocity (/ 1.0 3))
                  (note :pitch c :length length :velocity (/ 1.0 3))
              )
          )

          (piano-roll
              (chord 1/1 "a2"  "c3" "e3")
              (chord 1/1 "a2"  "d3" "f3")
              (chord 1/2 "g#2" "b3" "e3")
              (chord 1/2 "g#2" "b3" "d3")
              (chord 1/1 "a2"  "c3" "e3")
          )
      )
  )

  (song :bpm 120 :notes melody)

Compiling this program to a waveform results in this audio:

.. raw:: html

   <audio controls="controls">
         <source src="_static/chords.ogg" type="audio/ogg">
         Your browser does not support the <code>audio</code> element. 
   </audio>

Notes and melodies are first-class entities in syn.txt,
and can be freely recombined and passed around.

Build Instructions
------------------

The project is written in Rust_ and all tooling is pulled in via Nix_,
although just having rustc and Cargo available *should* just work.
That said, you can simply run `nix-shell` in the repository root to drop
into a shell where all dependencies are avaiable.
Then, the normal Cargo_ workflow applies.

.. code-block:: bash

   # Play the included example melody
   cargo run --bin musicc test-data/demo.syn

At the moment, it depends on spawning a sox_ subprocess and piping the audio data to it
for actually playing sound.
If everything worked, it should produce something similar to the following audio snippet:

.. raw:: html

   <audio controls="controls">
         <source src="_static/demo.ogg" type="audio/ogg">
         Your browser does not support the <code>audio</code> element. 
   </audio>

.. _chords.syn: https://github.com/fatho/syn-txt/blob/master/test-data/chords.syn
.. _roadmap: https://github.com/fatho/syn-txt/blob/master/planning/roadmap.md
.. _examples: https://github.com/fatho/syn-txt/blob/master/test-data/
.. _Rust: https://www.rust-lang.org/
.. _Nix: https://nixos.org/nix/
.. _Cargo: https://doc.rust-lang.org/cargo/
.. _sox: http://sox.sourceforge.net/



Indices and tables
==================

* :ref:`genindex`
* :ref:`search`
