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
