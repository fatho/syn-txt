# Roadmap

The goal is to have a DSL based on S-Expressions for

- specifying the audio rack (synthesizers, effects, mixer, routing) as a DAG (cycles might be possible, but introduce latency depending on the buffer size)
- specifying music as notes (piano-roll) in a composable way
- specifying which piano rolls are played when on which track

## Done

- implement lexer and parser for DSL
- implement very basic interpreter for DSL
- make interpreter extensible with extra built-in functions and types
  (for example, for piano rolls)
- make executable that takes a DSL file, evaluates it to a piano roll and plays that on a synth
- organize `musicc` code so that it is more maintainable
- allow configuring the synthesizer from within the language
- add list value and combinators to language (useful for defining chords on the fly)
- add record accessors (in the form of dicts)

## Now

- document existing functionality
- implement useful combinators as built-ins
- refactor syn_txt::pianoroll interface to be less ad-hoc (or get rid of it completely?)
- allow multiple tracks to play at once on separate instruments

## Later

- design interface for composing tracks both vertically (playing multiple tracks at once)
  and horizontally (sequencing tracks and stacks of tracks), like it is possible with notes already.
- implement sample tracks (playing samples directly, without going through notes)
- implement more instruments


## Eventually

- design more complex audio routing as DAG, including effects and a mixer
- implement song-global automation (global LFOs, automation tracks, sidechaining)