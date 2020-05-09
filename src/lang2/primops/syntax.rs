//! Primitive operations that define special syntax with deep integration in how evaluation is handled.

use crate::lang2::interpreter::*;
use crate::lang2::value::*;
use crate::lang2::heap::*;

/// Interprets the `(begin ...)` construct that creates a new scope and executes a series of expressions,
/// returning the value of the last one.
pub fn begin(int: &mut Interpreter, args: Gc<Value>) -> Result<Gc<Value>> {
    int.push_scope();
    let mut result = Ok(int.heap_alloc_value(Value::Void));
    let mut current = args.pin();
    while let Value::Cons(head, tail) = &*current {
        result = int.eval(Gc::clone(head));
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
    let mut parameters = Vec::new();
    while let Value::Cons(param, tail) = &*params.pin() {
        let ident = int.as_symbol(param)?;
        if parameters.contains(&ident) {
            return Err(int.make_error(param.id(), EvalErrorKind::Redefinition(ident.clone())));
        } else {
            parameters.push(ident.clone());
        }
        params = Gc::clone(tail);
    }
    let closure = Closure {
        captured_scope: int.scope_stack().clone(),
        parameters,
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
        Value::Symbol(sym) =>  {
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

    if let Some((var, _)) = int.scope_stack().pin().define(var.clone(), value) {
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
    int.expect_no_more_arguments(&mut args)?;

    // Traverse the scopes from top to bottom
    match int.scope_stack().pin().set(&var, value) {
        Ok(_) => Ok(int.heap_alloc_value(Value::Void)),
        Err(_) => Err(int.make_error(var_arg.id(), EvalErrorKind::NoSuchVariable(var.clone()))),
    }
}
