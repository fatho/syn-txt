use std::collections::HashMap;
use std::{fmt, rc::Rc};
use std::cell::RefCell;

use super::ast;
use super::primops;
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
    IncompatibleArguments,
    NotEnoughArguments,
    TooManyArguments,
    DivisionByZero,
    /// Type error (e.g. trying to add two incompatible types).
    Type,
    /// Tried to redefine a variable in the scope it was originally defined.
    /// (Shadowing variables in a new scope is fine).
    Redefinition(ast::Ident),
    /// Miscellaneous errors that shouldn't happen, but might.
    Other(String),
}

impl fmt::Display for IntpErrInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntpErrInfo::Unevaluatable => write!(f, "unevaluatable"),
            IntpErrInfo::NoSuchVariable(var) => write!(f, "no such variable `{}`", &var.0),
            IntpErrInfo::Uncallable => write!(f, "uncallable"),
            IntpErrInfo::IncompatibleArguments => write!(f, "incompatible arguments"),
            IntpErrInfo::NotEnoughArguments => write!(f, "not enough arguments in function call"),
            IntpErrInfo::TooManyArguments => write!(f, "too many arguments in function call"),
            IntpErrInfo::DivisionByZero => write!(f, "division by zero"),
            IntpErrInfo::Redefinition(var) => write!(f, "redefined variable `{}`", &var.0),
            IntpErrInfo::Type => write!(f, "type error"),
            IntpErrInfo::Other(msg) => write!(f, "{}", msg),
        }
    }
}

pub struct Interpreter {
    /// Read-only scope (from the perspective of the language)
    /// providing all the built-in primops.
    builtins: Scope,
    scopes: Vec<Scope>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut builtin_scope = Scope::new();

        let prim = vec![
            // syntax
            ("begin", PrimOp(primops::begin)),
            ("define", PrimOp(primops::define)),
            ("set!", PrimOp(primops::set)),
            // operators
            ("+", PrimOp(primops::add)),
            ("-", PrimOp(primops::sub)),
            ("*", PrimOp(primops::mul)),
            ("/", PrimOp(primops::div)),
            // util
            ("print", PrimOp(primops::print)),
        ];

        for (name, fun) in prim {
            builtin_scope.set_var(ast::Ident(name.to_owned()), Value::FnPrim(fun));
        }

        Self {
            builtins: builtin_scope,
            scopes: vec![Scope::new()],
        }
    }

    pub fn register_stateful_primop(&mut self, name: &str, op: PrimOpExt) {
        self.builtins
            .set_var(ast::Ident((*name).to_owned()), Value::FnExt(op));
    }

    /// Read a variable from the topmost scope that defines it.
    pub fn lookup_var(&self, var: &ast::Ident) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.lookup_var(var) {
                return Some(val);
            }
        }
        self.builtins.lookup_var(var)
    }

    pub fn scopes_mut(&mut self) -> &mut [Scope] {
        &mut self.scopes
    }

    /// Create a new topmost scope for bindings.
    /// Any `define`s and `set!`s will target the top-most scope.
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new())
    }

    /// Remove the topmost scope and all its bindings.
    pub fn pop_scope(&mut self) {
        debug_assert!(self.scopes.len() > 1, "cannot pop the last scope");
        self.scopes.pop();
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
        let head = self.eval(head_exp)?;
        let args = ArgParser::new(span, &list[1..]);

        match head {
            Value::FnPrim(PrimOp(prim_fn)) => prim_fn(self, args),
            Value::FnExt(PrimOpExt(prim_fn_ext)) => prim_fn_ext(self, args),
            _ => Err(IntpErr::new(head_exp.src, IntpErrInfo::Uncallable)),
        }
    }
}

/// Helper for parsing the arguments of a call.
pub struct ArgParser<'a> {
    /// The span of the whole list whose arguments are parsed.
    list_span: Span,
    args: &'a [ast::SymExpSrc],
}

impl<'a> ArgParser<'a> {
    pub fn new(list_span: Span, args: &'a [ast::SymExpSrc]) -> Self {
        Self { list_span, args }
    }

    /// Return the number of unparsed arguments
    pub fn remaining(&self) -> usize {
        self.args.len()
    }

    /// The source location of the whole list.
    pub fn list_span(&self) -> Span {
        self.list_span
    }

    /// Returns if there are no more arguments.
    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    /// Return the current argument as symbolic expression.
    pub fn symbolic<'b>(&'b mut self) -> InterpreterResult<&'a ast::SymExpSrc> {
        if let Some(sym) = self.args.first() {
            self.args = &self.args[1..];
            Ok(sym)
        } else {
            Err(IntpErr::new(
                self.list_span,
                IntpErrInfo::NotEnoughArguments,
            ))
        }
    }

    /// The current argument must have a plain variable.
    pub fn variable<'b>(&'b mut self) -> InterpreterResult<&'a ast::Ident> {
        let arg = self.symbolic()?;
        if let ast::SymExp::Variable(ident) = &arg.exp {
            Ok(ident)
        } else {
            Err(IntpErr::new(
                self.list_span,
                IntpErrInfo::IncompatibleArguments,
            ))
        }
    }

    /// Evaluate the current argument.
    pub fn value(&mut self, interp: &mut Interpreter) -> InterpreterResult<Value> {
        let arg = self.symbolic()?;
        interp.eval(arg)
    }

    /// End the argument parsing process. There must not be any more arguments remaining.
    pub fn done(&self) -> InterpreterResult<()> {
        if self.args.is_empty() {
            Ok(())
        } else {
            Err(IntpErr::new(self.list_span, IntpErrInfo::TooManyArguments))
        }
    }
}

/// A binding scope for variables.
/// Scopes are lexially nested, and inner scopes have precedence before outer scopes.
pub struct Scope {
    bindings: HashMap<ast::Ident, Value>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    /// Set a variable in this scope and return its previous value, if there was one.
    pub fn set_var(&mut self, var: ast::Ident, value: Value) -> Option<Value> {
        self.bindings.insert(var, value)
    }

    pub fn lookup_var(&self, var: &ast::Ident) -> Option<&Value> {
        self.bindings.get(var)
    }

    pub fn lookup_var_mut(&mut self, var: &ast::Ident) -> Option<&mut Value> {
        self.bindings.get_mut(var)
    }
}
/// A primitive operation exposed to the interpreted language.
#[derive(Copy, Clone)]
pub struct PrimOp(pub for<'a> fn(&mut Interpreter, ArgParser<'a>) -> InterpreterResult<Value>);

impl fmt::Debug for PrimOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ptr = self.0 as *const ();
        write!(f, "PrimOp({:p})", ptr)
    }
}

impl PartialEq for PrimOp {
    fn eq(&self, other: &Self) -> bool {
        let self_ptr = self.0 as *const ();
        let other_ptr = other.0 as *const ();
        self_ptr == other_ptr
    }
}
impl Eq for PrimOp {}

/// A primitive operation exposed to the interpreted language by an extension.
/// It has access to user-defined state, but the cost is that it is a closure
/// behind a shared reference, instead of being a pure function pointer.
#[derive(Clone)]
pub struct PrimOpExt(
    pub Rc<dyn for<'a> Fn(&mut Interpreter, ArgParser<'a>) -> InterpreterResult<Value>>,
);

impl fmt::Debug for PrimOpExt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ptr = &(*self.0) as *const _;
        write!(f, "PrimOpExt({:p})", ptr)
    }
}

impl PartialEq for PrimOpExt {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for PrimOpExt {}

impl PrimOpExt {
    /// Boundle a primop with its corresponding state
    pub fn with_shared_state<S, F>(state: Rc<RefCell<S>>, fun: F) -> PrimOpExt
    where
        S: 'static,
        F: Fn(
                &mut S,
                &mut Interpreter,
                ArgParser,
            ) -> InterpreterResult<Value>
            + 'static,
    {
        PrimOpExt(Rc::new(move |intp, args| {
            let mut state_mut = state.borrow_mut();
            fun(&mut *state_mut, intp, args)
        }))
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
    /// A primitive operation
    FnPrim(PrimOp),
    /// A primitive operation provided by an extension
    FnExt(PrimOpExt),
    /// A value provided by an interpreter extension.
    /// Interpretation of it is up to the extension.
    Ext(ExtVal),
}

impl Value {
    pub fn from_extension<T: ExtensionValue>(val: T) -> Self {
        Self::Ext(ExtVal::new(val))
    }
}

#[derive(Debug, Clone)]
pub struct ExtVal(Rc<dyn ExtensionValue>);

impl ExtVal {
    pub fn new<T: ExtensionValue>(val: T) -> Self {
        Self(Rc::new(val))
    }
}

impl PartialEq for ExtVal {
    fn eq(&self, other: &ExtVal) -> bool {
        self.0.partial_eq(&*other.0)
    }
}

/// Trait to be implemented by values that are provided by interpreter extensions.
pub trait ExtensionValue: std::any::Any + fmt::Debug {
    /// Dynamically typed version of the PartialEq trait
    fn partial_eq(&self, other: &dyn ExtensionValue) -> bool;

    /// Workaround for the lack of trait-upcasting in Rust.
    /// This method allows the self in `partial_eq` to downcast the `other`.
    fn as_any(&self) -> &dyn std::any::Any;
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

    fn expect_values_or_errors(input: &str, expected: &[Result<Value, IntpErrInfo>]) {
        let tokens = Lexer::new(input)
            .collect::<Result<Vec<(Span, Token)>, _>>()
            .unwrap();
        let expressions = Parser::new(input, &tokens).parse().unwrap();
        let mut interp = Interpreter::new();

        for (i, (e, expected_result)) in expressions.iter().zip(expected).enumerate() {
            let result = interp.eval(e).map_err(|e| e.info);
            assert_eq!(&result, expected_result, "\nmismatch in result {}", i);
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
            (define r (/ 5. 2))
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

    #[test]
    fn test_scopes() {
        expect_values_or_errors(
            r#"
            (define pi 3.14)
            (define val 5)
            val
            (define area
                (begin
                    (define r 1.0)
                    (set! val (* pi (* r r)))
                    val
                ))
            r
            val
            ; ensure that an error inside a nested scope pops that scope
            (begin
                (define foo 1) ; we expect this definition to be cleaned up
                (set! bar 1) ; the error occurs here
            )
            foo
            "#,
            &[
                Ok(Value::Unit),
                Ok(Value::Unit),
                Ok(Value::Int(5)),
                Ok(Value::Unit),
                Err(IntpErrInfo::NoSuchVariable(ast::Ident("r".to_owned()))),
                Ok(Value::Float(3.14)),
                Err(IntpErrInfo::NoSuchVariable(ast::Ident("bar".to_owned()))),
                Err(IntpErrInfo::NoSuchVariable(ast::Ident("foo".to_owned()))),
            ],
        )
    }
}
