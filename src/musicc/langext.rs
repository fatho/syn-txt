// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! Music-specific language extensions for the underlying scheme-like language.

use crate::lang::heap::*;
use crate::lang::interpreter::*;
use crate::lang::value::*;
use crate::note::Note;

pub static PRIMOPS: &[(&str, PrimOp)] = &[
    // Transposing notes represented as string or midi index
    ("transpose", PrimOp(prim_transpose)),
];

pub fn prim_transpose(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    todo!()
    // let note = args.extract::<NotePitch>(intp)?;
    // let amount = args.extract::<i64>(intp)?;
    // if let Some(new_note) = Note::try_from_midi(note.0.to_midi() as i64 + amount) {
    //     Ok(NotePitch(new_note).to_value())
    // } else {
    //     Err(IntpErr::new(
    //         args.list_span(),
    //         IntpErrInfo::Other(format!("transposed pitch exceeds MIDI range")),
    //     ))
    // }
}
