// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

use crate::lang::heap::*;
use crate::lang::interpreter::*;
use crate::{lang::value::*, rational::Rational};

use std::cmp::Ordering;

pub fn gt(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let a = int.pop_argument_eval(&mut args)?;
    let b = int.pop_argument_eval(&mut args)?;
    int.expect_no_more_arguments(&args)?;
    Ok(int.heap_alloc_value(Value::Bool(
        partial_cmp_impl(a.pin(), b.pin()) == Some(Ordering::Greater),
    )))
}

pub fn geq(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let a = int.pop_argument_eval(&mut args)?;
    let b = int.pop_argument_eval(&mut args)?;
    int.expect_no_more_arguments(&args)?;
    let order = partial_cmp_impl(a.pin(), b.pin());
    Ok(int.heap_alloc_value(Value::Bool(
        order == Some(Ordering::Greater) || order == Some(Ordering::Equal),
    )))
}

pub fn lt(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let a = int.pop_argument_eval(&mut args)?;
    let b = int.pop_argument_eval(&mut args)?;
    int.expect_no_more_arguments(&args)?;
    Ok(int.heap_alloc_value(Value::Bool(
        partial_cmp_impl(a.pin(), b.pin()) == Some(Ordering::Less),
    )))
}

pub fn leq(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let a = int.pop_argument_eval(&mut args)?;
    let b = int.pop_argument_eval(&mut args)?;
    int.expect_no_more_arguments(&args)?;
    let order = partial_cmp_impl(a.pin(), b.pin());
    Ok(int.heap_alloc_value(Value::Bool(
        order == Some(Ordering::Less) || order == Some(Ordering::Equal),
    )))
}

pub fn eq(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let a = int.pop_argument_eval(&mut args)?;
    let b = int.pop_argument_eval(&mut args)?;
    int.expect_no_more_arguments(&args)?;
    Ok(int.heap_alloc_value(Value::Bool(
        partial_cmp_impl(a.pin(), b.pin()) == Some(Ordering::Equal),
    )))
}

pub fn neq(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let a = int.pop_argument_eval(&mut args)?;
    let b = int.pop_argument_eval(&mut args)?;
    int.expect_no_more_arguments(&args)?;
    Ok(int.heap_alloc_value(Value::Bool(
        partial_cmp_impl(a.pin(), b.pin()) != Some(Ordering::Equal),
    )))
}

pub fn partial_cmp_impl(a: GcPin<Value>, b: GcPin<Value>) -> Option<Ordering> {
    match (&*a, &*b) {
        (Value::Str(a), Value::Str(b)) => a.partial_cmp(b),

        (Value::Float(a), Value::Float(b)) => a.partial_cmp(b),
        (Value::Float(a), Value::Int(b)) => a.partial_cmp(&(*b as f64)),
        (Value::Int(a), Value::Float(b)) => (*a as f64).partial_cmp(b),
        (Value::Int(a), Value::Int(b)) => a.partial_cmp(b),

        (Value::Ratio(a), Value::Ratio(b)) => a.partial_cmp(b),
        (Value::Ratio(a), Value::Int(b)) => a.partial_cmp(&Rational::from_int(*b)),
        (Value::Int(a), Value::Ratio(b)) => Rational::from_int(*a).partial_cmp(b),

        (Value::Bool(a), Value::Bool(b)) => a.partial_cmp(b),
        (Value::Nil, Value::Nil) => Some(Ordering::Equal),
        (Value::Nil, Value::Cons(_, _)) => Some(Ordering::Less),
        (Value::Cons(_, _), Value::Nil) => Some(Ordering::Greater),
        (Value::Cons(ahead, atail), Value::Cons(bhead, btail)) => {
            let head_order = partial_cmp_impl(ahead.pin(), bhead.pin())?;
            if head_order == Ordering::Equal {
                partial_cmp_impl(atail.pin(), btail.pin())
            } else {
                Some(head_order)
            }
        }
        _ => None,
    }
}
