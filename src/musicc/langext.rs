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
use crate::lang::{marshal, value::*};
use crate::note::Note;

pub static PRIMOPS: &[(&str, PrimOp)] = &[
    // Transposing notes represented as string or midi index
    ("transpose", PrimOp(prim_transpose)),
];

pub fn prim_transpose(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let note = int.pop_argument_eval_parse(&mut args, note_parser())?;
    let amount = int.pop_argument_eval_parse(&mut args, marshal::int())?;
    int.expect_no_more_arguments(&args)?;

    if let Some(new_note) = Note::try_from_midi(note.to_midi() as i64 + amount) {
        Ok(int.heap_alloc_value(Value::Int(new_note.to_midi() as i64)))
    } else {
        Err(int.make_error(
            args.id(),
            EvalErrorKind::Other("transposed pitch exceeds MIDI range".to_string()),
        ))
    }
}

pub fn note_parser() -> impl marshal::ParseValue<Repr = Note> {
    use marshal::ParseValue;
    marshal::string()
        .and_then(|s| Note::named_str(&s))
        .or(marshal::int().and_then(Note::try_from_midi))
}
