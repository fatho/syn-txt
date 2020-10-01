# Roadmap

The goal is to have a DSL for

- specifying the audio rack (synthesizers, effects, mixer, routing) as a DAG (cycles might be possible, but introduce latency depending on the buffer size)
- specifying music as notes (piano-roll) in a composable way
- specifying which piano rolls are played when on which track

## Done

- implement a basic synthesizer
- allow multiple tracks to play at once on separate instruments
- implement audio graph routing

## Soon

- implement instrument-local automation
  (e.g. adjust filter cut-off based on note duration)
- implement song-global automation (global LFOs, automation tracks, sidechaining)

## Later

- document existing functionality
- implement sample tracks (playing samples directly, without going through notes)
- implement more instruments
- implement more filters
- define serialization format
