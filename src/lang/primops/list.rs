//! List operations

use crate::lang::interpreter::*;
use std::rc::Rc;

/// Create a list value from its arguments.
pub fn list(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
    let mut elems = Vec::with_capacity(args.remaining());

    while !args.is_empty() {
        let value = args.value(intp)?;
        elems.push(value);
    }

    Ok(Value::List(elems.into()))
}

/// Call a function for each element of a list.
pub fn for_each(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
    let fun = args.value(intp)?;
    let fun_src = args.last_span();

    let list: Rc<[Value]> = args.extract(intp)?;
    let list_src = args.last_span();

    for x in list.iter() {
        let _ = intp.call_values(fun_src, &fun, &[(list_src, x)])?;
    }

    Ok(Value::Unit)
}

/// Map over a list with some function return a new value to take its place.
pub fn map(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
    let fun = args.value(intp)?;
    let fun_src = args.last_span();

    let list: Rc<[Value]> = args.extract(intp)?;
    let list_src = args.last_span();

    let mut new_list = Vec::new();

    for x in list.iter() {
        let new_value = intp.call_values(fun_src, &fun, &[(list_src, x)])?;
        new_list.push(new_value);
    }

    Ok(Value::List(new_list.into()))
}

/// Concatenate lists.
pub fn concat(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
    let mut elems = Vec::new();

    while !args.is_empty() {
        let value: Rc<[Value]> = args.extract(intp)?;
        elems.extend_from_slice(&value);
    }

    Ok(Value::List(elems.into()))
}

/// Reverse a list.
pub fn reverse(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
    let list: Rc<[Value]> = args.extract(intp)?;

    // Copy into a new Rc of which we are the only owner at the moment
    let mut unique_list: Rc<[Value]> = (&*list).into();
    Rc::get_mut(&mut unique_list).unwrap().reverse();

    Ok(Value::List(unique_list))
}

/// Create a list of a range of integers. Possible forms:
///  * `(range n)` is equivalent to `(range 0 n 1)`
///  * `(range m n)` is equivalent to `(range m n 1)`
///  * `(range m n step)` produces a list starting at `m` and
///    adding `step` at each step, until it is greater than or equal to `n`.
pub fn range(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
    let a: i64 = args.extract(intp)?;
    let b: Option<i64> = if args.is_empty() {
        None
    } else {
        Some(args.extract(intp)?)
    };
    let c: Option<i64> = if args.is_empty() {
        None
    } else {
        Some(args.extract(intp)?)
    };
    args.done()?;

    let (start, end, step) = match (a, b, c) {
        (start, Some(end), Some(step)) => (start, end, step),
        (start, Some(end), None) => (start, end, 1),
        (end, _, _) => (0, end, 1),
    };

    let mut values: Vec<Value> = Vec::new();
    let mut current = start;
    while (step > 0 && current < end) || (step < 0 && current > end) {
        values.push(Value::Int(current));
        current += step;
    }
    Ok(Value::List(values.into()))
}
