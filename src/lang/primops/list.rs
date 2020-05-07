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
