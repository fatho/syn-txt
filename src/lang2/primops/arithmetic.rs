//! Arithmetic primitive operations.

use crate::lang2::interpreter::*;
use crate::lang2::heap::*;
use crate::lang2::value::*;
use crate::rational::Rational;

/// Add all the arguments. If no arguments are passed, a zero int is returned.
pub fn add(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let mut accum = Number::Int(0);
    while let Some((next, id)) = Number::try_pop(int, &mut args)? {
        accum = match widen(accum, next) {
            (Number::Int(x), Number::Int(y)) => Number::Int(x + y),
            (Number::Ratio(x), Number::Ratio(y)) => Number::from_rational(x + y),
            (Number::Float(x), Number::Float(y)) => Number::Float(x + y),
            _ => return Err(int.make_error(id, EvalErrorKind::Type)),
        };
    }
    Ok(int.heap_alloc_value(accum.to_value()))
}

/// Add subtract all arguments after the first from the first argument.
pub fn sub(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let mut accum = Number::pop(int, &mut args)?.0;
    let mut has_more = false;
    while let Some((next, id)) = Number::try_pop(int, &mut args)? {
        has_more = true;
        accum = match widen(accum, next) {
            (Number::Int(x), Number::Int(y)) => Number::Int(x - y),
            (Number::Ratio(x), Number::Ratio(y)) => Number::from_rational(x - y),
            (Number::Float(x), Number::Float(y)) => Number::Float(x - y),
            _ => return Err(int.make_error(id, EvalErrorKind::Type)),
        };
    }
    if ! has_more {
        accum = match accum {
            Number::Float(x) => Number::Float(-x),
            Number::Ratio(x) => Number::from_rational(-x),
            Number::Int(x) => Number::Int(-x),
        };
    }
    Ok(int.heap_alloc_value(accum.to_value()))
}

/// Add all the arguments. If no arguments are passed, a `1` int is returned.
pub fn mul(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let mut accum = Number::Int(1);
    while let Some((next, id)) = Number::try_pop(int, &mut args)? {
        accum = match widen(accum, next) {
            (Number::Int(x), Number::Int(y)) => Number::Int(x * y),
            (Number::Ratio(x), Number::Ratio(y)) => Number::from_rational(x * y),
            (Number::Float(x), Number::Float(y)) => Number::Float(x * y),
            _ => return Err(int.make_error(id, EvalErrorKind::Type)),
        };
    }
    Ok(int.heap_alloc_value(accum.to_value()))
}

/// Divide the first argument by all other arguments.
/// If just one argument is given, the reciprocal is returned.
pub fn div(int: &mut Interpreter, mut args: Gc<Value>) -> Result<Gc<Value>> {
    let (mut accum, initial_id) = Number::pop(int, &mut args)?;
    let mut has_more = false;
    while let Some((next, id)) = Number::try_pop(int, &mut args)? {
        has_more = true;
        if next.is_zero() {
            return Err(int.make_error(id, EvalErrorKind::DivisionByZero));
        }
        accum = match widen(accum, next) {
            (Number::Int(x), Number::Int(y)) => Number::from_rational(Rational::new(x, y)),
            (Number::Ratio(x), Number::Ratio(y)) => Number::from_rational(x / y),
            (Number::Float(x), Number::Float(y)) => Number::Float(x / y),
            _ => return Err(int.make_error(args.id(), EvalErrorKind::Type)),
        };
    }
    if ! has_more {
        if accum.is_zero() {
            return Err(int.make_error(initial_id, EvalErrorKind::DivisionByZero));
        }
        accum = match accum {
            Number::Float(x) => Number::Float(1.0 / x),
            Number::Ratio(x) => Number::Ratio(x.recip()),
            Number::Int(x) => Number::Ratio(Rational::new(1, x)),
        };
    }
    Ok(int.heap_alloc_value(accum.to_value()))
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Number {
    /// A float
    Float(f64),
    /// A rational number
    Ratio(Rational),
    /// An integer
    Int(i64),
}

impl Number {
    /// Convert from rational, downgrading a rational with denominator one back to int.
    pub fn from_rational(r: Rational) -> Self {
        if r.denominator() == 1 {
            Number::Int(r.numerator())
        } else {
            Number::Ratio(r)
        }
    }

    pub fn to_value(self) -> Value {
        match self {
            Number::Float(x) => Value::Float(x),
            Number::Ratio(x) => Value::Ratio(x),
            Number::Int(x) => Value::Int(x),
        }
    }

    pub fn try_from_value(value: &Value) -> Option<Number> {
        match value {
            Value::Int(i) => Some(Number::Int(*i)),
            Value::Float(i) => Some(Number::Float(*i)),
            Value::Ratio(i) => Some(Number::Ratio(*i)),
            _ => None,
        }
    }

    pub fn pop(int: &mut Interpreter, args: &mut Gc<Value>) -> Result<(Number, Id)> {
        let arg = int.pop_argument(args)?;
        let arg_id = arg.id();
        let value = int.eval(arg)?;
        if let Some(number) = Number::try_from_value(&*value.pin()) {
            Ok((number, arg_id))
        } else {
            Err(int.make_error(arg_id, EvalErrorKind::Type))
        }
    }

    pub fn try_pop(int: &mut Interpreter, args: &mut Gc<Value>) -> Result<Option<(Number, Id)>> {
        if let Value::Nil = &*args.pin() {
            Ok(None)
        } else {
            Number::pop(int, args).map(Some)
        }
    }

    pub fn is_zero(&self) -> bool {
        match self {
            Number::Int(0) => true,
            Number::Float(f) => *f == 0.0,
            Number::Ratio(r) => r.is_zero(),
            _ => false,
        }
    }
}

/// Widen numeric types if necessary if the values don't have the same type.
/// The only case where this happens is if one of the values is an integer
/// and the other is a float or rational.
fn widen(v1: Number, v2: Number) -> (Number, Number) {
    match (v1, v2) {
        (Number::Int(x), y @ Number::Ratio(_)) => (Number::Ratio(Rational::from_int(x)), y),
        (x @ Number::Ratio(_), Number::Int(y)) => (x, Number::Ratio(Rational::from_int(y))),
        (Number::Int(x), y @ Number::Float(_)) => (Number::Float(x as f64), y),
        (x @ Number::Float(_), Number::Int(y)) => (x, Number::Float(y as f64)),
        other => other,
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
        widen(Number::Int(ix), Number::Int(iy)),
        (Number::Int(ix), Number::Int(iy))
    );
    assert_eq!(
        widen(Number::Ratio(rx), Number::Ratio(ry)),
        (Number::Ratio(rx), Number::Ratio(ry))
    );
    assert_eq!(
        widen(Number::Float(fx), Number::Float(fy)),
        (Number::Float(fx), Number::Float(fy))
    );

    // Widens where necessary
    assert_eq!(
        widen(Number::Int(ix), Number::Ratio(ry)),
        (Number::Ratio(rx), Number::Ratio(ry))
    );
    assert_eq!(
        widen(Number::Float(fx), Number::Int(iy)),
        (Number::Float(fx), Number::Float(fy))
    );
}
