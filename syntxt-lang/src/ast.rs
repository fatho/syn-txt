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

use std::{
    ops::{Deref, Range},
    sync::Arc,
};

use crate::{lexer::Span, line_map::Pos};
use syntxt_core::{nonnan::F64N, note::Note, rational::Rational};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node<T> {
    pub span: Span,
    pub pos: Range<Pos>,
    pub data: T,
}

impl<T> Node<T> {
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Node<U> {
        Node {
            span: self.span,
            pos: self.pos,
            data: f(self.data),
        }
    }

    pub fn nest<U, F: FnOnce(NodePtr<T>) -> U>(self, f: F) -> Node<U> {
        Node {
            span: self.span.clone(),
            pos: self.pos.clone(),
            data: f(Arc::new(self)),
        }
    }

    pub fn replace<U>(&self, data: U) -> Node<U> {
        Node {
            span: self.span.clone(),
            pos: self.pos.clone(),
            data,
        }
    }
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
    pub value: NodePtr<Expr>,
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
    Sequence(NodePtr<Sequence>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sequence {
    pub llbracket: Node<()>,
    pub symbols: Vec<Node<SeqSym>>,
    pub rrbracket: Node<()>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeqSym {
    Note {
        note: Note,
        duration: Rational,
    },
    Rest {
        duration: Rational,
    },
    /// A nested expression evaluating to a sequence again that is spliced in its place
    Expr(NodePtr<Expr>),
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

pub trait Visit {
    fn visit(&self, visitor: &mut dyn Visitor);
}

pub trait Walk {
    fn walk(&self, visitor: &mut dyn Visitor);
}

impl<T: Visit> Visit for &[T] {
    fn visit(&self, visitor: &mut dyn Visitor) {
        for node in self.iter() {
            node.visit(visitor);
        }
    }
}

impl<T: Visit> Visit for Vec<T> {
    fn visit(&self, visitor: &mut dyn Visitor) {
        for node in self.iter() {
            node.visit(visitor);
        }
    }
}

impl<T: Walk> Walk for Arc<T> {
    fn walk(&self, visitor: &mut dyn Visitor) {
        self.deref().walk(visitor)
    }
}

impl<T: Visit> Visit for Arc<T> {
    fn visit(&self, visitor: &mut dyn Visitor) {
        self.deref().visit(visitor)
    }
}

impl Visit for Node<Root> {
    fn visit(&self, visitor: &mut dyn Visitor) {
        visitor.root(self)
    }
}

impl Visit for Node<Object> {
    fn visit(&self, visitor: &mut dyn Visitor) {
        visitor.object(self)
    }
}

impl Visit for Node<Attribute> {
    fn visit(&self, visitor: &mut dyn Visitor) {
        visitor.attribute(self)
    }
}

impl Visit for Node<Expr> {
    fn visit(&self, visitor: &mut dyn Visitor) {
        visitor.expr(self)
    }
}

impl Visit for Node<Sequence> {
    fn visit(&self, visitor: &mut dyn Visitor) {
        visitor.sequence(self);
    }
}

impl Visit for Node<SeqSym> {
    fn visit(&self, visitor: &mut dyn Visitor) {
        visitor.seq_sym(self);
    }
}

impl<T: Walk> Walk for Node<T> {
    fn walk(&self, visitor: &mut dyn Visitor) {
        self.data.walk(visitor);
    }
}

impl Walk for Root {
    fn walk(&self, visitor: &mut dyn Visitor) {
        self.objects.visit(visitor)
    }
}

impl Walk for Object {
    fn walk(&self, visitor: &mut dyn Visitor) {
        self.attrs.visit(visitor);
        self.children.visit(visitor);
    }
}

impl Walk for Attribute {
    fn walk(&self, visitor: &mut dyn Visitor) {
        self.value.visit(visitor);
    }
}

impl Walk for Expr {
    #[allow(unused_variables)]
    fn walk(&self, visitor: &mut dyn Visitor) {
        match self {
            Expr::String(_) => {}
            Expr::Int(_) => {}
            Expr::Ratio(_) => {}
            Expr::Float(_) => {}
            Expr::Bool(_) => {}
            Expr::Var(_) => {}
            Expr::Unary { operator, operand } => {
                operand.visit(visitor);
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                left.visit(visitor);
                right.visit(visitor);
            }
            Expr::Paren {
                lparen,
                expr,
                rparen,
            } => {
                expr.visit(visitor);
            }
            Expr::Object(obj) => {
                obj.visit(visitor);
            }
            Expr::Accessor {
                expr,
                dot,
                attribute,
            } => {
                expr.visit(visitor);
            }
            Expr::Call {
                callee,
                lparen,
                arguments,
                rparen,
            } => {
                callee.visit(visitor);
                arguments.visit(visitor);
            }
            Expr::Sequence(seq) => {
                seq.visit(visitor);
            }
        }
    }
}

impl Walk for Sequence {
    fn walk(&self, visitor: &mut dyn Visitor) {
        self.symbols.visit(visitor);
    }
}

impl Walk for SeqSym {
    fn walk(&self, visitor: &mut dyn Visitor) {
        match self {
            // These are leaves that cannot be walked further
            SeqSym::Note { .. } => {}
            SeqSym::Rest { .. } => {}
            // Visit the nested parts
            SeqSym::Expr(expr) => {
                expr.visit(visitor);
            }
        }
    }
}

#[allow(unused_variables)]
pub trait Visitor {
    fn root(&mut self, node: &Node<Root>) {}
    fn object(&mut self, node: &Node<Object>) {}
    fn attribute(&mut self, node: &Node<Attribute>) {}
    fn expr(&mut self, node: &Node<Expr>) {}
    fn sequence(&mut self, node: &Node<Sequence>) {}
    fn seq_sym(&mut self, node: &Node<SeqSym>) {}
}
