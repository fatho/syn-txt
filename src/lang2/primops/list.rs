//! List operations

use crate::lang2::heap::*;
use crate::lang2::interpreter::*;
use crate::lang2::value::*;

/// Create a list value from its arguments.
pub fn list(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let mut elems = Vec::new();

    // TODO: find a way to build lists without intermediate vector
    while let Value::Cons(head, tail) = &*args.pin() {
        let value = int.eval(Gc::clone(head))?;
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
        let value = int.eval(Gc::clone(head))?;
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
        let mut inner = int.eval(Gc::clone(head))?.pin();
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
        let value = int.eval(Gc::clone(head))?;
        reversed = int.heap_alloc_value(Value::Cons(value, reversed));
        list = tail.pin();
    }
    Ok(reversed)
}

// /// Create a list of a range of integers. Possible forms:
// ///  * `(range n)` is equivalent to `(range 0 n 1)`
// ///  * `(range m n)` is equivalent to `(range m n 1)`
// ///  * `(range m n step)` produces a list starting at `m` and
// ///    adding `step` at each step, until it is greater than or equal to `n`.
// pub fn range(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
//     let a: i64 = int.pop_argument_eval(&mut args)?;
//     let b: Option<i64> = if args.is_empty() {
//         None
//     } else {
//         Some(args.extract(int)?)
//     };
//     let c: Option<i64> = if args.is_empty() {
//         None
//     } else {
//         Some(args.extract(int)?)
//     };
//     int.expect_no_more_arguments(&args)?;

//     let (start, end, step) = match (a, b, c) {
//         (start, Some(end), Some(step)) => (start, end, step),
//         (start, Some(end), None) => (start, end, 1),
//         (end, _, _) => (0, end, 1),
//     };

//     let mut values: Vec<Value> = Vec::new();
//     let mut current = start;
//     while (step > 0 && current < end) || (step < 0 && current > end) {
//         values.push(Value::Int(current));
//         current += step;
//     }
//     Ok(Value::List(values.into()))
// }

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
