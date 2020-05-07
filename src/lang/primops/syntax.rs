//! Primitive operations that define special syntax with deep integration in how evaluation is handled.

use std::rc::Rc;

use crate::lang::ast;
use crate::lang::interpreter::*;

/// Interprets the `(begin ...)` construct that creates a new scope and executes a series of expressions,
/// returning the value of the last one.
pub fn begin(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
    intp.push_scope();
    let mut result = Ok(Value::Unit);
    while !args.is_empty() {
        result = args.value(intp);
        if result.is_err() {
            break;
        }
    }
    // ensure that we always pop the scope, even if the evaluation errored out
    intp.pop_scope();
    result
}

/// Interprets the `(lambda (args) expr)` construct for creating a closure.
pub fn lambda(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
    // Parse the parameter list
    let arg_parameters = args.symbolic()?;

    let param_parser = match &arg_parameters.exp {
        ast::SymExp::List(arg_list) => ArgParser::new(args.last_span(), &arg_list),
        _ => {
            return Err(IntpErr::new(
                args.last_span(),
                IntpErrInfo::IncompatibleArguments,
            ))
        }
    };

    // Parse the lambda expression
    let lambda_expr = args.symbolic()?;

    lambda_impl(intp, param_parser, lambda_expr)
}

pub fn lambda_impl(
    intp: &mut Interpreter,
    mut params: ArgParser,
    expr: &ast::SymExpSrc,
) -> InterpreterResult<Value> {
    let mut parameters = Vec::new();
    while !params.is_empty() {
        let ident = params.variable()?;
        if parameters.contains(ident) {
            return Err(IntpErr::new(
                params.last_span(),
                IntpErrInfo::Redefinition(ident.clone()),
            ));
        } else {
            parameters.push(ident.clone());
        }
    }
    let closure = Closure {
        captured_scope: intp.scope_stack().clone(),
        parameters,
        code: expr.clone(),
    };
    Ok(Value::Closure(Rc::new(closure)))
}

/// Define a variable in the current top-most scope.
pub fn define(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
    // The thing being defined
    let defined = args.symbolic()?;

    // Transform `(define (f ...) v)` into `(define f (lambda (...) v))`
    let (var, value) = match &defined.exp {
        ast::SymExp::List(elems) => {
            let mut lambda_args = ArgParser::new(defined.src, &elems);
            let lambda_name = lambda_args.variable()?;
            let lambda_expr = args.symbolic()?;
            let closure_value = lambda_impl(intp, lambda_args, lambda_expr)?;

            (lambda_name, closure_value)
        }
        ast::SymExp::Variable(var) => {
            let value = args.value(intp)?;
            args.done()?;
            (var, value)
        }
        _ => {
            return Err(IntpErr::new(
                args.last_span(),
                IntpErrInfo::IncompatibleArguments,
            ))
        }
    };

    if let Some((var, _)) = intp.scope_stack().borrow_mut().define(var.clone(), value) {
        Err(IntpErr::new(
            args.list_span(),
            IntpErrInfo::Redefinition(var),
        ))
    } else {
        Ok(Value::Unit)
    }
}

/// Set a variable in the scope it was defined in.
pub fn set(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
    let var = args.variable()?;
    let value = args.value(intp)?;
    args.done()?;

    // Traverse the scopes from top to bottom
    match intp.scope_stack().borrow_mut().set(var, value) {
        Ok(_) => Ok(Value::Unit),
        Err(_) => Err(IntpErr::new(
            args.list_span(),
            IntpErrInfo::NoSuchVariable(var.clone()),
        )),
    }
}
