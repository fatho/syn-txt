# Automation

Goal: Change instrument/filter parameters dynamically over the course of the song.

## Inputs

Which inputs can be used in automation depends on the context.

In all automation contexts, the following **global inputs** are available:
- time since song started (in seconds and bars)
- maybe some global settings like bpm?

For automation of per-note parameters (e.g. in an instrument that has per-note state):

- note velocity
- time since note started (in seconds and bars)

Addionally, in a node that has automation inputs, these inputs can be referenced in
all automation expressions inside that node.

## Output

A single floating point number indicating the parameter value.

TODO: are there automable integer parameters, and if so, how to compute those? (maybe rounding)
