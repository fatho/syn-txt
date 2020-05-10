//! List operations

use crate::lang::heap::*;
use crate::lang::interpreter::*;
use crate::lang::value::*;

/// Create a list value from its arguments.
pub fn list(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let mut elems = Vec::new();

    // TODO: find a way to build lists without intermediate vector
    while let Value::Cons(head, tail) = &*args.pin() {
        let value = int.eval(head.pin())?;
        elems.push(value);
        args = Gc::clone(tail);
    }

    rev_list_from_iter(int, elems.into_iter().rev())
}

/// Call a function for each element of a list.
pub fn for_each(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let fun = int.pop_argument_eval(&mut args)?;

    let nil = int.heap_alloc_value(Value::Nil);

    while let Value::Cons(head, tail) = &*args.pin() {
        let fun_args = int.heap_alloc_value(Value::Cons(Gc::clone(head), Gc::clone(&nil)));
        int.eval_call(Gc::clone(&fun), fun_args)?;
        args = Gc::clone(tail);
    }

    Ok(int.heap_alloc_value(Value::Void))
}

pub fn cons(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let head = int.pop_argument_eval(&mut args)?;
    let tail = int.pop_argument_eval(&mut args)?;
    Ok(int.heap_alloc_value(Value::Cons(head, tail)))
}

pub fn head(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let list = int.pop_argument_eval(&mut args)?;
    if let Value::Cons(head, _) = &*list.pin() {
        Ok(Gc::clone(head))
    } else {
        Err(int.make_error(list.id(), EvalErrorKind::Type))
    }
}

pub fn tail(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let list = int.pop_argument_eval(&mut args)?;
    if let Value::Cons(_, tail) = &*list.pin() {
        Ok(Gc::clone(tail))
    } else {
        Err(int.make_error(list.id(), EvalErrorKind::Type))
    }
}

pub fn is_cons(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let list = int.pop_argument_eval(&mut args)?;
    Ok(int.heap_alloc_value(Value::Bool(list.pin().is_cons())))
}

pub fn is_nil(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let list = int.pop_argument_eval(&mut args)?;
    Ok(int.heap_alloc_value(Value::Bool(list.pin().is_nil())))
}

/// Map over a list with some function return a new value to take its place.
pub fn map(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let fun = int.pop_argument_eval(&mut args)?;
    let mut list = int.pop_argument_eval(&mut args)?;
    int.expect_no_more_arguments(&args)?;

    let mut elems = Vec::new();
    let nil = int.heap_alloc_value(Value::Nil);

    while let Value::Cons(head, tail) = &*list.pin() {
        let value = int.eval(head.pin())?;
        let fun_args = int.heap_alloc_value(Value::Cons(value, Gc::clone(&nil)));
        let result = int.eval_call(Gc::clone(&fun), fun_args)?;
        elems.push(result);
        list = Gc::clone(tail);
    }

    rev_list_from_iter(int, elems.into_iter().rev())
}

/// Concatenate lists.
pub fn concat(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let mut elems = Vec::new();

    while let Value::Cons(head, tail) = &*args.pin() {
        let mut inner = int.eval(head.pin())?.pin();
        while let Value::Cons(inner_head, inner_tail) = &*inner {
            elems.push(Gc::clone(inner_head));
            inner = inner_tail.pin();
        }
        args = Gc::clone(tail);
    }

    rev_list_from_iter(int, elems.into_iter().rev())
}

/// Reverse a list.
pub fn reverse(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let mut reversed = int.heap_alloc_value(Value::Nil);
    let mut list = int.pop_argument_eval(&mut args)?.pin();
    while let Value::Cons(head, tail) = &*list {
        let value = int.eval(head.pin())?;
        reversed = int.heap_alloc_value(Value::Cons(value, reversed));
        list = tail.pin();
    }
    Ok(reversed)
}

/// Build a list from the reverse order of items in the given iterator.
fn rev_list_from_iter<I: Iterator<Item = Gc<Value>>>(
    int: &mut Interpreter,
    values: I,
) -> Result<Gc<Value>> {
    let mut result = int.heap_alloc_value(Value::Nil);
    for val in values {
        result = int.heap_alloc_value(Value::Cons(val, result));
    }
    Ok(result)
}
