// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2021  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::sync::Arc;

use crate::lexer::Span;
use syntxt_core::{nonnan::F64N, rational::Rational};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node<T> {
    pub span: Span,
    pub data: T,
}

pub type NodePtr<T> = Arc<Node<T>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Root {
    pub objects: Vec<Node<Object>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Object {
    pub name: Node<String>,
    pub lbrace: Node<()>,
    pub attrs: Vec<Node<Attribute>>,
    pub children: Vec<Node<Object>>,
    pub rbrace: Node<()>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: Node<String>,
    pub colon: Node<()>,
    pub value: Node<Expr>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    String(String),
    Int(i64),
    Ratio(Rational),
    Float(F64N),
    Bool(bool),
    Unary {
        operator: Node<UnaryOp>,
        operand: NodePtr<Expr>,
    },
    Binary {
        left: NodePtr<Expr>,
        operator: Node<BinaryOp>,
        right: NodePtr<Expr>,
    },
    Paren {
        lparen: Node<()>,
        expr: NodePtr<Expr>,
        rparen: Node<()>,
    },
    Object(NodePtr<Object>),
    Var(String),
    Accessor {
        expr: NodePtr<Expr>,
        dot: Node<()>,
        attribute: Node<String>,
    },
    Call {
        callee: NodePtr<Expr>,
        lparen: Node<()>,
        // TODO: Also keep track of commas in argument list
        arguments: Vec<Node<Expr>>,
        rparen: Node<()>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mult,
    Div,
    And,
    Or,
}
