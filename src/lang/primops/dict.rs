// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! Dict operations

use crate::lang::heap::*;
use crate::lang::interpreter::*;
use crate::lang::value::*;
use std::collections::HashMap;

/// Create a dict value from its arguments.
pub fn dict(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let mut dict = HashMap::new();

    while let Value::Cons(head, tail) = &*args.pin() {
        let key = if let Value::Keyword(key) = &*head.pin() {
            key.clone()
        } else {
            return Err(int.make_error(head.id(), EvalErrorKind::IncompatibleArguments));
        };
        args = Gc::clone(tail);
        let value = int.pop_argument_eval(&mut args)?;
        dict.insert(key, value);
    }

    Ok(int.heap_alloc_value(Value::Dict(dict)))
}

/// Return a new dict based on an existing dict where zero or more entries get a new value.
pub fn dict_update(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let dict_arg = int.pop_argument(&mut args)?;
    let dict_id = dict_arg.id();
    let mut dict = if let Value::Dict(d) = &*int.eval(dict_arg.pin())?.pin() {
        d.clone()
    } else {
        return Err(int.make_error(dict_id, EvalErrorKind::Type));
    };

    while let Value::Cons(head, tail) = &*args.pin() {
        let key = if let Value::Keyword(key) = &*head.pin() {
            key.clone()
        } else {
            return Err(int.make_error(head.id(), EvalErrorKind::IncompatibleArguments));
        };
        args = Gc::clone(tail);
        let value = int.pop_argument_eval(&mut args)?;
        dict.insert(key, value);
    }

    Ok(int.heap_alloc_value(Value::Dict(dict)))
}

/// Retrieve the entry of a dict.
pub fn dict_get(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let dict_arg = int.pop_argument(&mut args)?;
    let dict_id = dict_arg.id();
    match &*int.eval(dict_arg.pin())?.pin() {
        Value::Dict(dict) => {
            let arg = int.pop_argument(&mut args)?.pin();
            let key = if let Value::Keyword(key) = &*arg {
                key
            } else {
                return Err(int.make_error(arg.id(), EvalErrorKind::IncompatibleArguments));
            };
            int.expect_no_more_arguments(&args)?;

            let value = dict.get(key).ok_or_else(|| {
                int.make_error(arg.id(), EvalErrorKind::UnknownKeyword(key.to_name()))
            })?;

            Ok(value.clone())
        }
        _ => Err(int.make_error(dict_id, EvalErrorKind::Type)),
    }
}
