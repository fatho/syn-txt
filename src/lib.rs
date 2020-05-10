// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

// Language implementation
pub mod lang;
pub mod musicc;

// modules for making sounds
pub mod note;
pub mod pianoroll;
pub mod synth;
pub mod util;
pub mod wave;

// audio I/O
pub mod output;

// Utility modules
pub mod rational;
