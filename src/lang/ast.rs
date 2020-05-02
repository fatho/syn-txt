use super::span::Span;
use crate::rational::Rational;

/// A non-namespaced identifier.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Ident(pub String);

/// An expression with a source annotation
#[derive(Debug, Clone)]
pub struct SymExpSrc {
    /// The source code range that the expression originates from.
    pub src: Span,
    /// The expression itself
    pub exp: SymExp,
}

/// Sybolic expression (S-Expression)
#[derive(Debug, Clone)]
pub enum SymExp {
    /// An abstract expression that cannot be evaluated.
    /// Only used for matching named arguments.
    Keyword(Ident),
    /// A variable, user defined or built-in
    Variable(Ident),
    /// A string
    Str(String),
    /// A float
    Float(f64),
    /// A rational number
    Ratio(Rational),
    /// An integer
    Int(i64),
    /// A list
    List(Vec<SymExpSrc>),
}
