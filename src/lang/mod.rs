//! Parsing etc. for the syn.txt language

pub mod lexer;

use crate::rational::Rational;

#[derive(Debug)]
pub struct Node {
    /// The type of the node, called "kind" because type is a reserved word in rust.
    kind: String,
    attributes: Vec<(String, Value)>,
    children: Vec<Node>,
}

#[derive(Debug)]
pub enum Value {
    Str(String),
    Float(f64),
    Ratio(Rational),
    Node(Node),
}
