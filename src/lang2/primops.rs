// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! Primitive operations exposed in the interpreter.

mod syntax;
pub use syntax::*;

mod arithmetic;
pub use arithmetic::*;

mod util;
pub use util::*;

mod list;
pub use list::*;

mod relational;
pub use relational::*;

// mod dict;
// pub use dict::*;
