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

## Now

- implement useful combinators as built-ins
- allow configuring the synthesizer from within the language
- refactor piano roll interface to be less ad-hoc
- add list value and combinators to language (useful for defining chords on the fly)
- add record accessors

## Later

- expose mixer channels, tracks and playlists (what to play when on which track) in DSL

## Eventually

- design more complex audio routing as DAG, including effects and a mixer