use crate::rational::Rational;

/// A non-namespaced identifier.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Ident(pub String);

/// Sybolic expression (S-Expression)
#[derive(Debug, Clone, PartialEq)]
pub enum SymExp {
    /// An abstract expression that cannot be evaluated.
    /// Only used for matching named arguments.
    Keyword(Ident),
    Variable(Ident),
    Literal(Value),
    List(Vec<SymExp>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// An opaque internal value with the given id
    Internal(usize),
    /// A string
    Str(String),
    /// A float
    Float(f64),
    /// A rational number
    Ratio(Rational),
    /// An integer
    Int(i64),
}
