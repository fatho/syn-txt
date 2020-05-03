use std::collections::HashMap;
use std::fmt;

use super::ast;
use super::span::Span;
use crate::rational::Rational;

pub type InterpreterResult<T> = Result<T, IntpErr>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntpErr {
    location: Span,
    info: IntpErrInfo,
}

impl IntpErr {
    pub fn new(location: Span, info: IntpErrInfo) -> Self {
        Self { location, info }
    }

    pub fn location(&self) -> Span {
        self.location
    }

    pub fn info(&self) -> &IntpErrInfo {
        &self.info
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntpErrInfo {
    /// Some expressions, such as keywords or empty lists, cannot be evaluated
    Unevaluatable,
    /// Variable/function was not found
    NoSuchVariable(ast::Ident),
    /// Tried to call something that cannot be called, such as the int in `(1 a b)`.
    Uncallable,
    /// There was a problem with the arguments in a call
    Arguments,
    DivisionByZero,
    /// Type error (e.g. trying to add two incompatible types).
    Type,
}

impl fmt::Display for IntpErrInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntpErrInfo::Unevaluatable => write!(f, "unevaluatable"),
            IntpErrInfo::NoSuchVariable(var) => write!(f, "no such variable `{}`", &var.0),
            IntpErrInfo::Uncallable => write!(f, "uncallable"),
            IntpErrInfo::Arguments => write!(f, "incompatible arguments"),
            IntpErrInfo::DivisionByZero => write!(f, "division by zero"),
            IntpErrInfo::Type => write!(f, "type error"),
        }
    }
}

pub struct Interpreter {
    scopes: Vec<Scope>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
        }
    }

    pub fn lookup_var(&self, var: &ast::Ident) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.lookup_var(var) {
                return Some(val);
            }
        }
        None
    }

    pub fn eval(&mut self, sym: &ast::SymExpSrc) -> InterpreterResult<Value> {
        match &sym.exp {
            ast::SymExp::Keyword(_) => Err(IntpErr::new(sym.src, IntpErrInfo::Unevaluatable)),
            ast::SymExp::List(list) => self.eval_list(sym.src, &list),
            ast::SymExp::Str(v) => Ok(Value::Str(v.clone())),
            ast::SymExp::Float(v) => Ok(Value::Float(*v)),
            ast::SymExp::Ratio(v) => Ok(Value::Ratio(*v)),
            ast::SymExp::Int(v) => Ok(Value::Int(*v)),
            ast::SymExp::Variable(var) => {
                if let Some(value) = self.lookup_var(var) {
                    Ok(value.clone())
                } else {
                    Err(IntpErr::new(
                        sym.src,
                        IntpErrInfo::NoSuchVariable(var.clone()),
                    ))
                }
            }
        }
    }

    fn eval_list(&mut self, span: Span, list: &[ast::SymExpSrc]) -> InterpreterResult<Value> {
        let head_exp = list
            .first()
            .ok_or(IntpErr::new(span, IntpErrInfo::Uncallable))?;
        let mut args = ArgParser::new(span, &list[1..]);

        // TODO: allow arbitrary expression as head and evaluate it
        match &head_exp.exp {
            ast::SymExp::Variable(fun) => match fun.0.as_str() {
                "define" | "set!" => {
                    let var = args.variable()?;
                    let value = args.value(self)?;
                    args.done()?;

                    // TODO: disallow defining defined variables
                    // TODO: disallow setting undefined variables
                    self.scopes.last_mut().unwrap().set_var(var.clone(), value);
                    Ok(Value::Unit)
                }
                "+" => {
                    let mut v1 = args.value(self)?;
                    while !args.is_empty() {
                        v1 = v1
                            .add(&args.value(self)?)
                            .map_err(|e| IntpErr::new(span, e))?;
                    }
                    drop(args);
                    Ok(v1)
                }
                "-" => {
                    let mut v1 = args.value(self)?;
                    while !args.is_empty() {
                        v1 = v1
                            .sub(&args.value(self)?)
                            .map_err(|e| IntpErr::new(span, e))?;
                    }
                    drop(args);
                    Ok(v1)
                }
                "*" => {
                    let mut v1 = args.value(self)?;
                    while !args.is_empty() {
                        v1 = v1
                            .mul(&args.value(self)?)
                            .map_err(|e| IntpErr::new(span, e))?;
                    }
                    drop(args);
                    Ok(v1)
                }
                "/" => {
                    let mut v1 = args.value(self)?;
                    while !args.is_empty() {
                        v1 = v1
                            .div(&args.value(self)?)
                            .map_err(|e| IntpErr::new(span, e))?;
                    }
                    drop(args);
                    Ok(v1)
                }
                _ => Err(IntpErr::new(
                    head_exp.src,
                    IntpErrInfo::NoSuchVariable(fun.clone()),
                )),
            },
            _ => Err(IntpErr::new(head_exp.src, IntpErrInfo::Uncallable)),
        }
    }
}

struct ArgParser<'a> {
    /// The span of the whole list whose arguments are parsed.
    list_span: Span,
    args: &'a [ast::SymExpSrc],
    expected: usize,
}

impl<'a> ArgParser<'a> {
    pub fn new(list_span: Span, args: &'a [ast::SymExpSrc]) -> Self {
        Self {
            list_span,
            args,
            expected: 0,
        }
    }

    /// Returns if there are no more arguments.
    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    /// Return the current argument as symbolic expression.
    pub fn symbolic<'b>(&'b mut self) -> InterpreterResult<&'a ast::SymExpSrc> {
        if let Some(sym) = self.args.first() {
            self.args = &self.args[1..];
            self.expected += 1;
            Ok(sym)
        } else {
            // TODO: more specific error
            Err(IntpErr::new(self.list_span, IntpErrInfo::Arguments))
        }
    }

    /// The current argument must have a plain variable.
    pub fn variable<'b>(&'b mut self) -> InterpreterResult<&'a ast::Ident> {
        let arg = self.symbolic()?;
        if let ast::SymExp::Variable(ident) = &arg.exp {
            Ok(ident)
        } else {
            // TODO: more specific error
            Err(IntpErr::new(self.list_span, IntpErrInfo::Arguments))
        }
    }

    /// Evaluate the current argument.
    pub fn value(&mut self, interp: &mut Interpreter) -> InterpreterResult<Value> {
        let arg = self.symbolic()?;
        interp.eval(arg)
    }

    pub fn done(self) -> InterpreterResult<()> {
        if self.args.is_empty() {
            Ok(())
        } else {
            Err(IntpErr::new(self.list_span, IntpErrInfo::Arguments))
            // TODO: more specific error
            // Err(format!(
            //     "{} arguments expected, but {} given at {:?}",
            //     self.expected,
            //     self.args.len() + self.expected,
            //     self.list_span
            // ))
        }
    }
}

struct Scope {
    bindings: HashMap<ast::Ident, Value>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn set_var(&mut self, var: ast::Ident, value: Value) {
        self.bindings.insert(var, value);
    }

    pub fn lookup_var(&self, var: &ast::Ident) -> Option<&Value> {
        self.bindings.get(var)
    }
}

/// Evaluating expressions results in values.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A string
    Str(String),
    /// A float
    Float(f64),
    /// A rational number
    Ratio(Rational),
    /// An integer
    Int(i64),
    /// The unit value
    Unit,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Type {
    Str,
    Float,
    Ratio,
    Int,
    Unit,
}

impl Value {
    pub fn get_type(&self) -> Type {
        match self {
            Value::Str(_) => Type::Str,
            Value::Float(_) => Type::Float,
            Value::Ratio(_) => Type::Ratio,
            Value::Int(_) => Type::Int,
            Value::Unit => Type::Unit,
        }
    }

    // Extracting values

    pub fn as_float(&self) -> Result<f64, IntpErrInfo> {
        match self {
            Value::Float(f) => Ok(*f),
            Value::Int(i) => Ok(*i as f64),
            Value::Ratio(r) => Ok(r.numerator() as f64 / r.denominator() as f64),
            _ => Err(IntpErrInfo::Type),
        }
    }

    pub fn as_ratio(&self) -> Result<Rational, IntpErrInfo> {
        match self {
            Value::Int(i) => Ok(Rational::from_int(*i)),
            Value::Ratio(r) => Ok(*r),
            _ => Err(IntpErrInfo::Type),
        }
    }

    pub fn as_int(&self) -> Result<i64, IntpErrInfo> {
        match self {
            Value::Int(i) => Ok(*i),
            _ => Err(IntpErrInfo::Type),
        }
    }

    // Operations

    pub fn add(&self, other: &Value) -> Result<Value, IntpErrInfo> {
        let result = self.get_type().merge_coercible(other.get_type())?;
        match result {
            Type::Float => Ok(Value::Float(self.as_float()? + other.as_float()?)),
            Type::Int => Ok(Value::Int(self.as_int()? + other.as_int()?)),
            Type::Ratio => Ok(Value::Ratio(self.as_ratio()? + other.as_ratio()?)),
            _ => Err(IntpErrInfo::Type),
        }
    }

    pub fn sub(&self, other: &Value) -> Result<Value, IntpErrInfo> {
        let result = self.get_type().merge_coercible(other.get_type())?;
        match result {
            Type::Float => Ok(Value::Float(self.as_float()? - other.as_float()?)),
            Type::Int => Ok(Value::Int(self.as_int()? - other.as_int()?)),
            Type::Ratio => Ok(Value::Ratio(self.as_ratio()? - other.as_ratio()?)),
            _ => Err(IntpErrInfo::Type),
        }
    }

    pub fn mul(&self, other: &Value) -> Result<Value, IntpErrInfo> {
        let result = self.get_type().merge_coercible(other.get_type())?;
        match result {
            Type::Float => Ok(Value::Float(self.as_float()? * other.as_float()?)),
            Type::Int => Ok(Value::Int(self.as_int()? * other.as_int()?)),
            Type::Ratio => Ok(Value::Ratio(self.as_ratio()? * other.as_ratio()?)),
            _ => Err(IntpErrInfo::Type),
        }
    }

    pub fn div(&self, other: &Value) -> Result<Value, IntpErrInfo> {
        let result = self.get_type().merge_coercible(other.get_type())?;
        match result {
            Type::Float => Ok(Value::Float(self.as_float()? / other.as_float()?)),
            // NOTE: dividing two ints always results in a rational
            Type::Int => {
                let denom = other.as_int()?;
                if denom == 0 {
                    return Err(IntpErrInfo::DivisionByZero);
                }
                Ok(Value::Ratio(Rational::new(self.as_int()?, denom)))
            }
            Type::Ratio => {
                let denom = other.as_ratio()?;
                if denom.is_zero() {
                    return Err(IntpErrInfo::DivisionByZero);
                }
                Ok(Value::Ratio(self.as_ratio()? / denom))
            }
            _ => Err(IntpErrInfo::Type),
        }
    }
}

impl Type {
    /// Return the type, if any exists, to which both types can be coerced.
    pub fn merge_coercible(self, other: Type) -> Result<Type, IntpErrInfo> {
        match (self, other) {
            // Operations involving one float always become all-float
            (Type::Float, Type::Ratio) => Ok(Type::Float),
            (Type::Ratio, Type::Float) => Ok(Type::Float),
            (Type::Float, Type::Int) => Ok(Type::Float),
            (Type::Int, Type::Float) => Ok(Type::Float),

            // Operations involving one rational always become all-rational
            (Type::Ratio, Type::Int) => Ok(Type::Ratio),
            (Type::Int, Type::Ratio) => Ok(Type::Ratio),

            // Reflexivity
            _ if self == other => Ok(self),

            _ => Err(IntpErrInfo::Type),
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::{lexer::*, parser::*};
    use super::*;

    fn expect_values(input: &str, expected: &[Value]) {
        let tokens = Lexer::new(input)
            .collect::<Result<Vec<(Span, Token)>, _>>()
            .unwrap();
        let expressions = Parser::new(input, &tokens).parse().unwrap();
        let mut interp = Interpreter::new();

        for (e, val) in expressions.iter().zip(expected) {
            let result = interp.eval(e).unwrap();
            assert_eq!(&result, val);
        }
    }

    #[test]
    fn test_arithmetic() {
        expect_values("(+ 1 2)", &[Value::Int(3)]);
        expect_values("(- 8 12)", &[Value::Int(-4)]);

        expect_values("(- -4 -9)", &[Value::Int(5)]);

        expect_values("(* 2 (/ 8 12))", &[Value::Ratio(Rational::new(4, 3))]);
        expect_values("(/ 5/4 8/7)", &[Value::Ratio(Rational::new(35, 32))]);
    }

    #[test]
    fn test_defines() {
        expect_values(
            r#"
            (define pi 3.14)
            (define r (/ 5 2))
            (define result
                (* r (* 2 pi)))
            result
            (set! result
                (* pi (* r r)))
            result"#,
            &[
                Value::Unit,
                Value::Unit,
                Value::Unit,
                Value::Float(15.700000000000001),
                Value::Unit,
                Value::Float(19.625),
            ],
        );
    }
}
