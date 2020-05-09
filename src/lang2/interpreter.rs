use std::fmt;

use super::ast;
use super::span::*;
use super::value::*;

pub type InterpreterResult<T> = Result<T, EvalError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalError {
    location: Option<SourceLocation>,
    info: EvalErrorKind,
}

impl EvalError {
    pub fn new(location: Option<SourceLocation>, info: EvalErrorKind) -> Self {
        Self { location, info }
    }

    pub fn location(&self) -> Option<&SourceLocation> {
        self.location.as_ref()
    }

    pub fn info(&self) -> &EvalErrorKind {
        &self.info
    }
}

/// Location referring to a source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub file: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalErrorKind {
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

impl fmt::Display for EvalErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EvalErrorKind::Unevaluatable => write!(f, "unevaluatable"),
            EvalErrorKind::NoSuchVariable(var) => write!(f, "no such variable `{}`", &var.0),
            EvalErrorKind::Uncallable => write!(f, "uncallable"),
            EvalErrorKind::IncompatibleArguments => write!(f, "incompatible arguments"),
            EvalErrorKind::NotEnoughArguments => write!(f, "not enough arguments in function call"),
            EvalErrorKind::TooManyArguments => write!(f, "too many arguments in function call"),
            EvalErrorKind::UnknownKeyword(var) => write!(f, "unknown keyword `{}`", &var.0),
            EvalErrorKind::DivisionByZero => write!(f, "division by zero"),
            EvalErrorKind::Redefinition(var) => write!(f, "redefined variable `{}`", &var.0),
            EvalErrorKind::Type => write!(f, "type error"),
            EvalErrorKind::Other(msg) => write!(f, "{}", msg),
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
