//! Useful functions that don't have any other place to be.

use crate::lang::interpreter::*;

/// Debug-print all arguments that are given
pub fn print(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
    while !args.is_empty() {
        let val = args.value(intp)?;
        println!("{:?}", val);
    }
    Ok(Value::Unit)
}
