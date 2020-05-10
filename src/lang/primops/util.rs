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
