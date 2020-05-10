// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! The module responsible for evaluating syn.txt files to an abstract description of a song.

use log::error;
use std::io;

use crate::lang::interpreter::Interpreter;
use crate::lang::value::Value;
use crate::lang::heap::{Heap, GcPin, Gc};
use crate::lang::debug::{DebugTable, SourceLocation};
use crate::lang::compiler;
use crate::lang::span::LineMap;

use super::{langext, output};

/// Evaluate syn.txt source code into a song description.
///
/// TODO: allow including other files.
pub fn eval(input_name: &str, input: &str) -> io::Result<output::Song> {
    let mut heap = Heap::new();
    let mut debug = DebugTable::new();
    let values = compiler::compile_str(&mut heap, &mut debug, input_name, input).unwrap();

    log::info!("evaluating {}", input_name);
    let mut int = Interpreter::new(&mut heap, &mut debug);
    for (name, op) in langext::PRIMOPS {
        int.register_primop(name, op.0)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e.info())))?;
    }

    let mut final_value = int.heap_alloc_value(Value::Void).pin();
    for v in values {
        match int.eval(v) {
            Ok(val) => final_value = val.pin(),
            Err(err) => {
                let source = err.location().and_then(|loc| int.debug_info().get_source(&loc.file));
                let lines = source.map(LineMap::new);
                log_error(lines.as_ref(), err.location(), err.info());
                return Err(io::Error::new(io::ErrorKind::InvalidData, err));
            }
        }
    }

    drop(int);
    heap.gc_cycles();

    build_song(final_value).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "not a song"))
}

fn build_song(value: GcPin<Value>) -> Option<output::Song> {
    todo!()
}

fn log_error<E: std::fmt::Display>(
    lines: Option<&LineMap>,
    location: Option<&SourceLocation>,
    message: E,
) {
    use std::fmt::Write;
    let mut buf = String::new();
    write!(&mut buf, "error: {}", message).unwrap();
    if let Some(location) = location {
        if let Some(lines) = lines {
            let start = lines.offset_to_pos(location.span.begin);
            let end = lines.offset_to_pos(location.span.end);
            writeln!(&mut buf, "({} {}-{})", location.file, start, end).unwrap();
            writeln!(&mut buf, "{}", lines.highlight(start, end, true)).unwrap();
        } else {
            writeln!(&mut buf, "({} {}-{})", location.file, location.span.begin, location.span.end).unwrap();
        }
    }
    error!("{}", buf);
}
