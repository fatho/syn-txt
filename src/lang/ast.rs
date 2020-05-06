// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

use super::span::Span;
use crate::rational::Rational;

/// A non-namespaced identifier.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Ident(pub String);

/// An expression with a source annotation
#[derive(Debug, Clone, PartialEq)]
pub struct SymExpSrc {
    /// The source code range that the expression originates from.
    pub src: Span,
    /// The expression itself
    pub exp: SymExp,
}

/// Sybolic expression (S-Expression)
#[derive(Debug, Clone, PartialEq)]
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
