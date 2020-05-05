use log::warn;
use std::collections::HashMap;
use std::{cell::RefCell, fmt, rc::Rc};

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
    /// Keyword was not understood by callee.
    UnknownKeyword(ast::Ident),
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
            IntpErrInfo::UnknownKeyword(var) => write!(f, "unknown keyword `{}`", &var.0),
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
    builtins: ScopeRef,
    /// Points to the current innermost scope
    scope_stack: ScopeRef,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        let mut builtin_scope = Scope::new();

        let prim = vec![
            // syntax
            ("begin", PrimOp(primops::begin)),
            ("define", PrimOp(primops::define)),
            ("lambda", PrimOp(primops::lambda)),
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
            builtin_scope.define(ast::Ident(name.to_owned()), Value::FnPrim(fun));
        }

        let builtins = builtin_scope.into_ref();
        let top_scope = Scope::nest(builtins.clone()).into_ref();

        Self {
            builtins,
            scope_stack: top_scope,
        }
    }

    pub fn register_primop(
        &mut self,
        name: &str,
        op: fn(&mut Interpreter, ArgParser) -> InterpreterResult<Value>,
    ) -> InterpreterResult<()> {
        let var = ast::Ident((*name).to_owned());
        let val = Value::FnPrim(PrimOp(op));
        if let Some((var, _val)) = self.builtins.borrow_mut().define(var, val) {
            // TODO: allow None as location
            Err(IntpErr::new(
                Span { begin: 0, end: 0 },
                IntpErrInfo::Redefinition(var),
            ))
        } else {
            Ok(())
        }
    }

    pub fn register_primop_ext<F>(&mut self, name: &str, op: F) -> InterpreterResult<()>
    where
        F: Fn(&mut Interpreter, ArgParser) -> InterpreterResult<Value> + 'static,
    {
        let var = ast::Ident((*name).to_owned());
        let val = Value::ext_closure(op);
        if let Some((var, _val)) = self.builtins.borrow_mut().define(var, val) {
            // TODO: allow None as location
            Err(IntpErr::new(
                Span { begin: 0, end: 0 },
                IntpErrInfo::Redefinition(var),
            ))
        } else {
            Ok(())
        }
    }

    pub fn scope_stack(&mut self) -> &ScopeRef {
        &self.scope_stack
    }

    /// Create a new topmost scope for bindings.
    /// Any `define`s and `set!`s will target the top-most scope.
    pub fn push_scope(&mut self) {
        let new_scope = Scope::nest(self.scope_stack.clone()).into_ref();
        self.scope_stack = new_scope;
    }

    /// Remove the topmost scope and all its bindings.
    pub fn pop_scope(&mut self) {
        let outer = self.scope_stack.borrow().outer();
        if let Some(outer) = outer {
            self.scope_stack = outer;
        } else {
            warn!("trying to pop outermost scope")
        }
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
                if let Some(value) = self.scope_stack.borrow().lookup(var) {
                    Ok(value)
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
            .ok_or_else(|| IntpErr::new(span, IntpErrInfo::Uncallable))?;
        let head = self.eval(head_exp)?;
        let mut args = ArgParser::new(span, &list[1..]);

        match head {
            Value::FnPrim(PrimOp(prim_fn)) => prim_fn(self, args),
            Value::Ext(val) => val.0.call(self, args),
            Value::Closure(clos) => {
                // Create a new scope inside the captured scope and define the arguments
                let mut scope_stack = Scope::nest(clos.captured_scope.clone());
                for param_var in clos.parameters.iter() {
                    if let Some((var, _)) = scope_stack.define(param_var.clone(), args.value(self)?)
                    {
                        // the `lambda` prim op ensures that the parameter names are unique,
                        // but the interpreter host might have sneaked in an invalid closure.
                        // TODO: ensure invariants in `Closure`
                        return Err(IntpErr::new(
                            args.last_span(),
                            IntpErrInfo::Other(format!(
                                "closure redefined parameter name {}",
                                var.0
                            )),
                        ));
                    }
                }

                let mut closure_interpreter = Self {
                    // the closure has captured the built-ins at creation time,
                    // this built-in scope is just a dummy value
                    builtins: Scope::new().into_ref(),
                    scope_stack: scope_stack.into_ref(),
                };

                closure_interpreter.eval(&clos.code)
            }
            _ => Err(IntpErr::new(head_exp.src, IntpErrInfo::Uncallable)),
        }
    }
}

/// Helper for parsing the arguments of a call.
pub struct ArgParser<'a> {
    /// The span of the whole list whose arguments are parsed.
    list_span: Span,
    /// Span of the most recently parsed argument, of of the whole list,
    /// if no argument has been parsed yet. Used for error attribution.
    last_span: Span,
    args: &'a [ast::SymExpSrc],
}

impl<'a> ArgParser<'a> {
    pub fn new(list_span: Span, args: &'a [ast::SymExpSrc]) -> Self {
        Self {
            list_span,
            last_span: list_span,
            args,
        }
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

    /// The source location of the most recently parsed argument.
    pub fn last_span(&self) -> Span {
        self.last_span
    }

    /// Return the current argument as symbolic expression.
    pub fn symbolic<'b>(&'b mut self) -> InterpreterResult<&'a ast::SymExpSrc> {
        if let Some(sym) = self.args.first() {
            self.args = &self.args[1..];
            self.last_span = sym.src;
            Ok(sym)
        } else {
            Err(IntpErr::new(
                self.list_span,
                IntpErrInfo::NotEnoughArguments,
            ))
        }
    }

    /// The current argument must be a plain variable.
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

    /// The current argument must be a keyword.
    pub fn keyword<'b>(&'b mut self) -> InterpreterResult<&'a ast::Ident> {
        let arg = self.symbolic()?;
        if let ast::SymExp::Keyword(ident) = &arg.exp {
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

    /// Evaluate the current argument and return it as a Rust value.
    pub fn extract<T: FromValue>(&mut self, interp: &mut Interpreter) -> InterpreterResult<T> {
        let arg = self.symbolic()?;
        let result = interp.eval(arg)?;
        T::from_value(result).map_err(|_| IntpErr::new(arg.src, IntpErrInfo::Type))
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

/// A reference to a shared scope.
pub type ScopeRef = Rc<RefCell<Scope>>;

/// A binding scope for variables.
/// Scopes are lexially nested, and inner scopes have precedence before outer scopes.
#[derive(Debug, PartialEq, Clone)]
pub struct Scope {
    bindings: HashMap<ast::Ident, Value>,
    outer: Option<ScopeRef>,
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

impl Scope {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            outer: None,
        }
    }

    /// Wrap this scope into a `ScopeRef`.
    pub fn into_ref(self) -> ScopeRef {
        Rc::new(RefCell::new(self))
    }

    /// Create a nested scope inside the given outer scope.
    pub fn nest(outer: ScopeRef) -> Self {
        Self {
            bindings: HashMap::new(),
            outer: Some(outer),
        }
    }

    /// Return a reference to the lexically outer scope, if this scope is not the outermost.
    pub fn outer(&self) -> Option<ScopeRef> {
        self.outer.clone()
    }

    /// Define a variable in this scope, if possible.
    /// On success, it returns `None`, otherwise it gives the arguments back to the caller.
    /// NOTE: `define`, unlike set, does not operate recursively on outer scopes.
    pub fn define(&mut self, var: ast::Ident, value: Value) -> Option<(ast::Ident, Value)> {
        if self.bindings.get(&var).is_none() {
            self.bindings.insert(var, value);
            None
        } else {
            Some((var, value))
        }
    }

    /// Set a variable in the scope where it was defined.
    /// If the variable was not defined, the `value` argument is returned as `Err`,
    /// otherwise, the previous value is returned as `Ok`.
    pub fn set(&mut self, var: &ast::Ident, value: Value) -> Result<Value, Value> {
        if let Some(slot) = self.bindings.get_mut(var) {
            Ok(std::mem::replace(slot, value))
        } else if let Some(outer) = self.outer.as_ref() {
            outer.borrow_mut().set(var, value)
        } else {
            Err(value)
        }
    }

    /// Return a copy of the value of the given variable, or `None` if it was not defined.
    pub fn lookup(&self, var: &ast::Ident) -> Option<Value> {
        if let Some(value) = self.bindings.get(var) {
            Some(value.clone())
        } else if let Some(outer) = self.outer.as_ref() {
            outer.borrow().lookup(var)
        } else {
            None
        }
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

/// Evaluating expressions results in values.

/// Values should be small enough so that they can be cloned without a big performance hit.
/// Any big values (such as `ExtClosure`) should be packaged behind `Rc`.
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
    /// A value provided by an interpreter extension.
    /// Interpretation of it is up to the extension.
    Ext(ExtVal),
    Closure(Rc<Closure>),
}

impl Value {
    /// Smart constructor for `Self::Ext` that performs the wrapping.
    pub fn ext<T: ExtensionValue>(val: T) -> Self {
        Self::Ext(ExtVal::new(val))
    }

    pub fn ext_closure<F>(fun: F) -> Self
    where
        F: Fn(&mut Interpreter, ArgParser) -> InterpreterResult<Value> + 'static,
    {
        Value::ext(ExtClosure(fun))
    }
}

/// A closure is a piece of code that has captured its environment and
/// can be called with arguments.
#[derive(Debug, PartialEq, Clone)]
pub struct Closure {
    /// The current scope stack at the point when the closure was made.
    /// Note: mutating variables in these scopes affects the closure as well.
    pub captured_scope: ScopeRef,
    /// Name of the parameters that must be passed to the closure when calling it.
    /// The names must be unique.
    pub parameters: Vec<ast::Ident>,
    /// The code to execute when calling the closure
    pub code: ast::SymExpSrc,
}

/// Wrapper for extension values that relays the `PartialEq` implementation to `partial_eq`.
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
    /// Dynamically typed version of the PartialEq trait.
    fn partial_eq(&self, other: &dyn ExtensionValue) -> bool;

    /// Workaround for the lack of trait-upcasting in Rust.
    /// This method allows the self in `partial_eq` to downcast the `other`.
    fn as_any(&self) -> &dyn std::any::Any;

    /// Allows external values to be callable. The default implementation returns an error.
    fn call(&self, _intp: &mut Interpreter, args: ArgParser) -> InterpreterResult<Value> {
        Err(IntpErr::new(args.list_span, IntpErrInfo::Uncallable))
    }
}

/// An extern callable closure.
pub struct ExtClosure<F>(F);

impl<F> fmt::Debug for ExtClosure<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ptr = &self.0 as *const _;
        write!(f, "ExtClosure({:p})", ptr)
    }
}

impl<F> ExtensionValue for ExtClosure<F>
where
    F: Fn(&mut Interpreter, ArgParser) -> InterpreterResult<Value> + 'static,
{
    /// Dynamically typed version of the PartialEq trait.
    fn partial_eq(&self, other: &dyn ExtensionValue) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            std::ptr::eq(self as *const _, other as *const _)
        } else {
            false
        }
    }

    /// Workaround for the lack of trait-upcasting in Rust.
    /// This method allows the self in `partial_eq` to downcast the `other`.
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call(&self, intp: &mut Interpreter, args: ArgParser) -> InterpreterResult<Value> {
        self.0(intp, args)
    }
}

#[macro_export]
macro_rules! declare_extension_value {
    ($value_type: ty) => {
        impl ExtensionValue for $value_type {
            fn partial_eq(&self, other: &dyn ExtensionValue) -> bool {
                if let Some(real_other) = other.as_any().downcast_ref::<Self>() {
                    self == real_other
                } else {
                    false
                }
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };
}

/// Trait for unmarshalling `Value`.
pub trait FromValue: Sized {
    fn from_value(value: Value) -> Result<Self, Value>;
}

impl FromValue for String {
    fn from_value(value: Value) -> Result<String, Value> {
        match value {
            Value::Str(x) => Ok(x),
            Value::Int(x) => Ok(format!("{}", x)),
            Value::Float(x) => Ok(format!("{}", x)),
            Value::Ratio(x) => Ok(format!("{}", x)),
            value => Err(value),
        }
    }
}

impl FromValue for i64 {
    fn from_value(value: Value) -> Result<i64, Value> {
        match value {
            Value::Int(x) => Ok(x),
            value => Err(value),
        }
    }
}

impl FromValue for f64 {
    fn from_value(value: Value) -> Result<f64, Value> {
        match value {
            Value::Int(x) => Ok(x as f64),
            Value::Float(x) => Ok(x),
            value => Err(value),
        }
    }
}

impl FromValue for Rational {
    fn from_value(value: Value) -> Result<Rational, Value> {
        match value {
            Value::Int(x) => Ok(Rational::from_int(x)),
            Value::Ratio(x) => Ok(x),
            value => Err(value),
        }
    }
}

impl<E: ExtensionValue + Clone> FromValue for E {
    fn from_value(value: Value) -> Result<Self, Value> {
        match value {
            Value::Ext(ref ext) => {
                if let Some(e) = ext.0.as_any().downcast_ref::<E>() {
                    Ok(e.clone())
                } else {
                    Err(value)
                }
            }
            value => Err(value),
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

    #[test]
    fn test_closure_stateless() {
        expect_values_or_errors(
            r#"
            (define plus-one
                (lambda (x) (+ x 1)))
            (plus-one 2)
            (plus-one 3)
            "#,
            &[Ok(Value::Unit), Ok(Value::Int(3)), Ok(Value::Int(4))],
        )
    }

    /// Test that closures can capture global state and any mutations
    /// from either inside or outside the closure can be seen elsewhere.
    #[test]
    fn test_closure_global_state() {
        expect_values_or_errors(
            r#"
            (define global-state 0)
            (define get-global
                (lambda ()
                    (begin
                        (define ret global-state)
                        (set! global-state (+ ret 1))
                        ret
                    )
                )
            )
            (get-global)
            (get-global)
            (set! global-state 10)
            (get-global)
            global-state
            "#,
            &[
                Ok(Value::Unit),
                Ok(Value::Unit),
                Ok(Value::Int(0)),
                Ok(Value::Int(1)),
                Ok(Value::Unit),
                Ok(Value::Int(10)),
                Ok(Value::Int(11)),
            ],
        )
    }

    /// Test that closures can capture scopes that are subsequently popped,
    /// never to be seen again.
    #[test]
    fn test_closure_hidden_state() {
        expect_values_or_errors(
            r#"
            ; A closure that, when called, creates a new fresh closure
            (define make-counter
                (lambda (initial-count)
                    (begin
                        (define counter initial-count)
                        (lambda ()
                            (begin
                                (define count counter)
                                (set! counter (+ 1 count))
                                count
                            ))
                    )))

            (define c1 (make-counter 0))
            (define c2 (make-counter 3))
            (c1)
            (c1)
            (c2)
            (c2)
            (c1)
            "#,
            &[
                Ok(Value::Unit),
                Ok(Value::Unit),
                Ok(Value::Unit),
                Ok(Value::Int(0)),
                Ok(Value::Int(1)),
                Ok(Value::Int(3)),
                Ok(Value::Int(4)),
                Ok(Value::Int(2)),
            ],
        )
    }
}
