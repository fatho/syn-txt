//! Arithmetic primitive operations.

use crate::lang::interpreter::*;
use crate::lang::span::*;
use crate::rational::Rational;


/// Add all the arguments. If no arguments are passed, a zero int is returned.
pub fn add(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
    let mut accum = Value::Int(0);
    while !args.is_empty() {
        accum = match widen(accum, args.value(intp)?) {
            (Value::Int(x), Value::Int(y)) => Value::Int(x + y),
            (Value::Ratio(x), Value::Ratio(y)) => Value::Ratio(x + y),
            (Value::Float(x), Value::Float(y)) => Value::Float(x + y),
            _ => return Err(IntpErr::new(args.list_span(), IntpErrInfo::Type)),
        };
    }
    Ok(accum)
}

/// Add subtract all arguments after the first from the first argument.
pub fn sub(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
    let mut accum = args.value(intp)?;
    while !args.is_empty() {
        accum = match widen(accum, args.value(intp)?) {
            (Value::Int(x), Value::Int(y)) => Value::Int(x - y),
            (Value::Ratio(x), Value::Ratio(y)) => Value::Ratio(x - y),
            (Value::Float(x), Value::Float(y)) => Value::Float(x - y),
            _ => return Err(IntpErr::new(args.list_span(), IntpErrInfo::Type)),
        };
    }
    Ok(accum)
}

/// Add all the arguments. If no arguments are passed, a `1` int is returned.
pub fn mul(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
    let mut accum = Value::Int(1);
    while !args.is_empty() {
        accum = match widen(accum, args.value(intp)?) {
            (Value::Int(x), Value::Int(y)) => Value::Int(x * y),
            (Value::Ratio(x), Value::Ratio(y)) => Value::Ratio(x * y),
            (Value::Float(x), Value::Float(y)) => Value::Float(x * y),
            _ => return Err(IntpErr::new(args.list_span(), IntpErrInfo::Type)),
        };
    }
    Ok(accum)
}

/// Divide the first argument by all other arguments.
/// If just one argument is given, the reciprocal is returned.
pub fn div(intp: &mut Interpreter, mut args: ArgParser) -> InterpreterResult<Value> {
    let mut accum = if args.remaining() == 1 {
        // compute reciprocal by using `1` as initial numerator
        Value::Int(1)
    } else {
        args.value(intp)?
    };

    while !args.is_empty() {
        let rhs = args.value(intp)?;
        div_by_zero_check(args.list_span(), &rhs)?;

        accum = match widen(accum, rhs) {
            // NOTE: contrary to the other arithmetic operations, int and int is not an int again.
            (Value::Int(x), Value::Int(y)) => Value::Ratio(Rational::new(x, y)),
            (Value::Ratio(x), Value::Ratio(y)) => Value::Ratio(x / y),
            (Value::Float(x), Value::Float(y)) => Value::Float(x / y),
            _ => return Err(IntpErr::new(args.list_span(), IntpErrInfo::Type)),
        };
    }
    Ok(accum)
}


/// Widen numeric types if necessary if the values don't have the same type.
/// The only case where this happens is if one of the values is an integer
/// and the other is a float or rational.
fn widen(v1: Value, v2: Value) -> (Value, Value) {
    match (v1, v2) {
        (Value::Int(x), y @ Value::Ratio(_)) => (Value::Ratio(Rational::from_int(x)), y),
        (x @ Value::Ratio(_), Value::Int(y)) => (x, Value::Ratio(Rational::from_int(y))),
        (Value::Int(x), y @ Value::Float(_)) => (Value::Float(x as f64), y),
        (x @ Value::Float(_), Value::Int(y)) => (x, Value::Float(y as f64)),
        other => other,
    }
}

fn div_by_zero_check(location: Span, denom: &Value) -> InterpreterResult<()> {
    let is_zero = match denom {
        Value::Int(0) => true,
        Value::Float(f) => *f == 0.0,
        Value::Ratio(r) => r.is_zero(),
        _ => false,
    };
    if is_zero {
        Err(IntpErr::new(location, IntpErrInfo::DivisionByZero))
    } else {
        Ok(())
    }
}

#[test]
fn test_widening() {
    let ix = 1;
    let iy = 2;
    let fx = 1.0;
    let fy: f64 = 2.0;
    let rx = Rational::from_int(1);
    let ry = Rational::from_int(2);

    // behaves as identity when types are the same
    assert_eq!(
        widen(Value::Int(ix), Value::Int(iy)),
        (Value::Int(ix), Value::Int(iy))
    );
    assert_eq!(
        widen(Value::Ratio(rx), Value::Ratio(ry)),
        (Value::Ratio(rx), Value::Ratio(ry))
    );
    assert_eq!(
        widen(Value::Float(fx), Value::Float(fy)),
        (Value::Float(fx), Value::Float(fy))
    );

    // Widens where necessary
    assert_eq!(
        widen(Value::Int(ix), Value::Ratio(ry)),
        (Value::Ratio(rx), Value::Ratio(ry))
    );
    assert_eq!(
        widen(Value::Float(fx), Value::Int(iy)),
        (Value::Float(fx), Value::Float(fy))
    );
}
