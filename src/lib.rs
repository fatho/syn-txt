// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

// modules for making sounds
pub mod envelope;
pub mod filter;
pub mod instrument;
pub mod note;
pub mod oscillator;
pub mod tuning;
pub mod util;
pub mod wave;

// Building songs
pub mod graph;
pub mod melody;
pub mod play;
pub mod song;

// Utility modules
pub mod rational;
