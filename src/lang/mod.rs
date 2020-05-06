// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! Implementation of the underlying scheme-like language of syn.txt.

pub mod ast;
pub mod interpreter;
pub mod lexer;
pub mod parser;
pub mod primops;
pub mod span;
