//! Primitive operations that define special syntax with deep integration in how evaluation is handled.

use crate::lang2::heap::*;
use crate::lang2::interpreter::*;
use crate::lang2::value::*;
use std::collections::{HashSet, HashMap};

/// Interprets the `(begin ...)` construct that creates a new scope and executes a series of expressions,
/// returning the value of the last one.
pub fn begin(int: &mut Interpreter, args: Gc<Value>) -> Result<Gc<Value>> {
    int.push_scope();
    let mut result = Ok(int.heap_alloc_value(Value::Void));
    let mut current = args.pin();
    while let Value::Cons(head, tail) = &*current {
        result = int.eval(head.pin());
        if result.is_err() {
            break;
        }
        current = tail.pin();
    }
    // ensure that we always pop the scope, even if the evaluation errored out
    int.pop_scope();
    result
}

/// Interprets the `(lambda (args) expr)` construct for creating a closure.
pub fn lambda(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    // Parse the parameter list
    let arg_parameters = int.pop_argument(&mut args)?;
    lambda_impl(int, arg_parameters, args)
}


pub fn lambda_impl(
    int: &mut Interpreter,
    mut params: Gc<Value>,
    body: Gc<Value>,
) -> Result<Gc<Value>> {
    let mut seen_vars: HashSet<Symbol> = HashSet::new();
    let mut parameters = Vec::new();

    // Positional arguments must always come before keyword arguments
    while let Value::Cons(param, tail) = &*params.pin() {
        match &*param.pin() {
            Value::Symbol(var) => {
                let var = var.clone();
                if ! seen_vars.insert(var.clone()) {
                    return Err(int.make_error(param.id(), EvalErrorKind::Redefinition(var)));
                }
                parameters.push(var);
            }
            _ => break,
        }
        params = Gc::clone(tail);
    }

    let mut named_parameters: HashMap<Symbol, (Symbol, Option<Gc<Value>>)> = HashMap::new();
    while let Value::Cons(param, tail) = &*params.pin() {
        params = Gc::clone(tail);
        let result = match &*param.pin() {
            Value::Keyword(key) => {
                // Named arguments either look like this:
                //   (... :name var ...)
                // or this
                //   (... :name (var default) ...)

                let next = int.pop_argument(&mut params)?.pin();
                match &*next {
                    Value::Symbol(var) => Ok((key.clone(), var.clone(), None)),
                    // TODO: find a suitable abstraction for parsing complex syntactic forms
                    // without producing a mess like this.
                    Value::Cons(var, default_tail) => match (&*var.pin(), &*default_tail.pin()) {
                        (Value::Symbol(var), Value::Cons(default, arg_tail)) => match &*arg_tail.pin() {
                            Value::Nil => Ok((key.clone(), var.clone(), Some(Gc::clone(default)))),
                            _ => Err(next.id()),
                        },
                        _ => Err(next.id()),
                    },
                    _ => Err(next.id()),
                }
            }
            _ => Err(param.id()),
        };
        match result {
            Ok((key, var, default)) => {
                if ! seen_vars.insert(var.clone()) {
                    return Err(int.make_error(param.id(), EvalErrorKind::Redefinition(var)));
                }
                if named_parameters.insert(key.clone(), (var, default)).is_some() {
                    return Err(int.make_error(param.id(), EvalErrorKind::DuplicateKeyword(key)));
                }
            }
            Err(id) => return Err(int.make_error(id, EvalErrorKind::IncompatibleArguments)),
        }
    }

    int.expect_no_more_arguments(&params)?;


    let closure = Closure {
        captured_scope: int.scope_stack().clone(),
        parameters,
        named_parameters,
        body,
    };
    let heap_closure = int.heap_alloc(closure);
    Ok(int.heap_alloc_value(Value::Closure(heap_closure)))
}

/// Define a variable in the current top-most scope.
pub fn define(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    // The thing being defined
    let defined = int.pop_argument(&mut args)?;

    // Transform `(define (f ...) v)` into `(define f (lambda (...) v))`
    let (var, value) = match &*defined.pin() {
        Value::Symbol(sym) => {
            let value = int.pop_argument_eval(&mut args)?;
            int.expect_no_more_arguments(&args)?;
            (sym.clone(), value)
        }
        Value::Cons(head, tail) => {
            let fun_name = int.as_symbol(head)?;
            let closure_value = lambda_impl(int, Gc::clone(tail), args)?;

            (fun_name, closure_value)
        }
        _ => {
            return Err(int.make_error(defined.id(), EvalErrorKind::IncompatibleArguments));
        }
    };

    if let Some((var, _)) = int.scope_stack().pin().define(var, value) {
        Err(int.make_error(defined.id(), EvalErrorKind::Redefinition(var)))
    } else {
        Ok(int.heap_alloc_value(Value::Void))
    }
}

/// Set a variable in the scope it was defined in.
pub fn set(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let var_arg = int.pop_argument(&mut args)?;
    let var = int.as_symbol(&var_arg)?;
    let value = int.pop_argument_eval(&mut args)?;
    int.expect_no_more_arguments(&args)?;

    // Traverse the scopes from top to bottom
    match int.scope_stack().pin().set(&var, value) {
        Ok(_) => Ok(int.heap_alloc_value(Value::Void)),
        Err(_) => Err(int.make_error(var_arg.id(), EvalErrorKind::NoSuchVariable(var.clone()))),
    }
}

/// If expression
pub fn if_(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let cond = int.pop_argument_eval(&mut args)?;
    let then = int.pop_argument(&mut args)?.pin();
    let else_ = int.pop_argument(&mut args)?.pin();
    int.expect_no_more_arguments(&args)?;

    if is_true(cond) {
        int.eval(then)
    } else {
        int.eval(else_)
    }
}

/// cond expression (evaluates first matching branch)
pub fn cond(int: &mut Interpreter, args: Gc<Value>) -> Result<Gc<Value>> {
    let mut current = args.pin();
    while let Value::Cons(branch, rest_branches) = &*current {
        let mut branch_current = Gc::clone(branch);
        let test = int.pop_argument_eval(&mut branch_current)?;
        let branch = int.pop_argument(&mut branch_current)?;
        int.expect_no_more_arguments(&branch_current)?;
        if is_true(test) {
            return int.eval(branch.pin());
        }
        current = rest_branches.pin();
    }
    Err(int.make_error(args.id(), EvalErrorKind::Other("no match".to_string())))
}


fn is_true(value: Gc<Value>) -> bool {
    match &*value.pin() {
        Value::Bool(b) => *b,
        Value::Nil => false,
        Value::Void => false,
        _ => true,
    }
}
