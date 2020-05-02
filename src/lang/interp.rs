use std::collections::HashMap;

use super::ast;
use super::span::Span;
use crate::rational::Rational;

type InterpreterError = String;
type InterpreterResult<T> = Result<T, InterpreterError>;

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
            ast::SymExp::Keyword(_) => Err(format!("cannot evaluate keyword at {:?}", sym.src)),
            ast::SymExp::List(list) => self.eval_list(sym.src, &list),
            ast::SymExp::Str(v) => Ok(Value::Str(v.clone())),
            ast::SymExp::Float(v) => Ok(Value::Float(*v)),
            ast::SymExp::Ratio(v) => Ok(Value::Ratio(*v)),
            ast::SymExp::Int(v) => Ok(Value::Int(*v)),
            ast::SymExp::Variable(var) => {
                if let Some(value) = self.lookup_var(var) {
                    Ok(value.clone())
                } else {
                    Err(format!("no such variable {} at {:?}", var.0, sym.src))
                }
            }
        }
    }

    fn eval_list(
        &mut self,
        span: Span,
        list: &[ast::SymExpSrc],
    ) -> InterpreterResult<Value> {
        let head_exp = list
            .first()
            .ok_or("cannot evaluate empty list".to_owned())?;
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
                    while ! args.is_empty() {
                        let v2 = args.value(self)?;

                        if let Some(result) = v1.add(&v2) {
                            v1 = result;
                        } else {
                            return Err(format!("cannot add {:?} and {:?} at {:?}", v1.get_type(), v2.get_type(), span));
                        }
                    }
                    drop(args);
                    Ok(v1)
                }
                "-" => {
                    let mut v1 = args.value(self)?;
                    while ! args.is_empty() {
                        let v2 = args.value(self)?;

                        if let Some(result) = v1.sub(&v2) {
                            v1 = result;
                        } else {
                            return Err(format!("cannot subtract {:?} and {:?} at {:?}", v1.get_type(), v2.get_type(), span));
                        }
                    }
                    drop(args);
                    Ok(v1)
                }
                "*" => {
                    let mut v1 = args.value(self)?;
                    while ! args.is_empty() {
                        let v2 = args.value(self)?;

                        if let Some(result) = v1.mul(&v2) {
                            v1 = result;
                        } else {
                            return Err(format!("cannot multiply {:?} and {:?} at {:?}", v1.get_type(), v2.get_type(), span));
                        }
                    }
                    drop(args);
                    Ok(v1)
                }
                "/" => {
                    let mut v1 = args.value(self)?;
                    while ! args.is_empty() {
                        let v2 = args.value(self)?;

                        if let Some(result) = v1.div(&v2) {
                            v1 = result;
                        } else {
                            return Err(format!("cannot divide {:?} and {:?} at {:?}", v1.get_type(), v2.get_type(), span));
                        }
                    }
                    drop(args);
                    Ok(v1)
                }
                unknown => Err(format!("unknown function {:?}", unknown)),
            },
            _ => Err(format!("cannot call {:?}", head_exp)),
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
            Err(format!("not enough arguments at {:?}", self.list_span))
        }
    }

    /// The current argument must have a plain variable.
    pub fn variable<'b>(&'b mut self) -> InterpreterResult<&'a ast::Ident> {
        let arg = self.symbolic()?;
        if let ast::SymExp::Variable(ident) = &arg.exp {
            Ok(ident)
        } else {
            Err(format!("expected variable at {:?}", arg.src))
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
            Err(format!(
                "{} arguments expected, but {} given at {:?}",
                self.expected,
                self.args.len() + self.expected,
                self.list_span
            ))
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

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Int(i) => Some(*i as f64),
            Value::Ratio(r) => Some(r.numerator() as f64 / r.denominator() as f64),
            _ => None,
        }
    }

    pub fn as_ratio(&self) -> Option<Rational> {
        match self {
            Value::Int(i) => Some(Rational::from_int(*i)),
            Value::Ratio(r) => Some(*r),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(i) => Some(*i),
            _ => None,
        }
    }

    // Operations

    pub fn add(&self, other: &Value) -> Option<Value> {
        let result = self.get_type().merge_coercible(other.get_type())?;
        match result {
            Type::Float => Some(Value::Float(self.as_float()? + other.as_float()?)),
            Type::Int => Some(Value::Int(self.as_int()? + other.as_int()?)),
            Type::Ratio => Some(Value::Ratio(self.as_ratio()? + other.as_ratio()?)),
            _ => None,
        }
    }

    pub fn sub(&self, other: &Value) -> Option<Value> {
        let result = self.get_type().merge_coercible(other.get_type())?;
        match result {
            Type::Float => Some(Value::Float(self.as_float()? - other.as_float()?)),
            Type::Int => Some(Value::Int(self.as_int()? - other.as_int()?)),
            Type::Ratio => Some(Value::Ratio(self.as_ratio()? - other.as_ratio()?)),
            _ => None,
        }
    }

    pub fn mul(&self, other: &Value) -> Option<Value> {
        let result = self.get_type().merge_coercible(other.get_type())?;
        match result {
            Type::Float => Some(Value::Float(self.as_float()? * other.as_float()?)),
            Type::Int => Some(Value::Int(self.as_int()? * other.as_int()?)),
            Type::Ratio => Some(Value::Ratio(self.as_ratio()? * other.as_ratio()?)),
            _ => None,
        }
    }

    pub fn div(&self, other: &Value) -> Option<Value> {
        let result = self.get_type().merge_coercible(other.get_type())?;
        match result {
            Type::Float => Some(Value::Float(self.as_float()? / other.as_float()?)),
            Type::Int => Some(Value::Int(self.as_int()? / other.as_int()?)),
            Type::Ratio => Some(Value::Ratio(self.as_ratio()? / other.as_ratio()?)),
            _ => None,
        }
    }
}

impl Type {
    /// Return the type, if any exists, to which both types can be coerced.
    pub fn merge_coercible(self, other: Type) -> Option<Type> {
        match (self, other) {
            // Operations involving one float always become all-float
            (Type::Float, Type::Ratio) => Some(Type::Float),
            (Type::Ratio, Type::Float) => Some(Type::Float),
            (Type::Float, Type::Int) => Some(Type::Float),
            (Type::Int, Type::Float) => Some(Type::Float),

            // Operations involving one rational always become all-rational
            (Type::Ratio, Type::Int) => Some(Type::Ratio),
            (Type::Int, Type::Ratio) => Some(Type::Ratio),

            // Reflexivity
            _ if self == other => Some(self),

            _ => None,
        }
    }
}
