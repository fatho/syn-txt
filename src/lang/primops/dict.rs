//! List operations

use crate::lang::ast;
use crate::lang::interpreter::*;
use std::{collections::HashMap, rc::Rc};

/// Create a dict value from its arguments.
pub fn dict(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
    let mut dict = HashMap::new();
    while !args.is_empty() {
        let key = args.keyword()?;
        let value = args.value(intp)?;

        dict.insert(key.clone(), value);
    }

    Ok(Value::Dict(Rc::new(dict)))
}

/// Return a new dict based on an existing dict where zero or more entries get a new value.
pub fn dict_update(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
    let mut dict: Rc<HashMap<ast::Ident, Value>> = args.extract(intp)?;

    let updated_dict = Rc::make_mut(&mut dict);

    while !args.is_empty() {
        let key = args.keyword()?;
        let value = args.value(intp)?;

        updated_dict.insert(key.clone(), value);
    }

    Ok(Value::Dict(dict))
}

/// Retrieve the entry of a dict.
pub fn dict_get(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
    let dict: Rc<HashMap<ast::Ident, Value>> = args.extract(intp)?;
    let key = args.keyword()?;

    let value = dict
        .get(key)
        .ok_or_else(|| IntpErr::new(args.last_span(), IntpErrInfo::UnknownKeyword(key.clone())))?;

    args.done()?;

    Ok(value.clone())
}
