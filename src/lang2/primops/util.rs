//! Useful functions that don't have any other place to be.

use crate::lang2::interpreter::*;
use crate::lang2::value::*;
use crate::lang2::heap::*;

/// Debug-print all arguments that are given
pub fn print(int: &mut Interpreter, args: Gc<Value>) -> Result<Gc<Value>> {
    let mut current = args.pin();
    while let Value::Cons(arg, tail) = &*current {
        println!("{:?}", arg);
        current = tail.pin();
    }
    Ok(int.heap_alloc(Value::Void))
}
