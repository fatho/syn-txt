// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! Useful functions that don't have any other place to be.

use crate::lang::heap::*;
use crate::lang::interpreter::*;
use crate::lang::pretty::pretty;
use crate::lang::value::*;

/// Debug-print all arguments that are given
pub fn print(int: &mut Interpreter, args: Gc<Value>) -> Result<Gc<Value>> {
    let mut current = args.pin();
    while let Value::Cons(arg, tail) = &*current {
        let value = int.eval(arg.pin())?;
        println!("{}", pretty(&value.pin()));
        current = tail.pin();
    }
    Ok(int.heap_alloc_value(Value::Void))
}
