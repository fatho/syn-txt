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

//! Evaluating the AST into a more concrete description of the song.

use std::{collections::HashMap, fmt::Debug, ops::Range};

use syntxt_core::{rational::Rational, sequence::{SeqItem, Sequence}};

use crate::{
    ast::{self, Node, NodePtr},
    lexer,
    line_map::Pos,
};

#[derive(Debug, PartialEq, Eq)]
pub struct Object {
    kind: String,
    id: Option<String>,
    attrs: HashMap<String, ThunkId>,
    children: Vec<ObjectId>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ObjectId(usize);

#[derive(Debug)]
pub enum Thunk {
    Unevaluated {
        scope: ScopeId,
        expr: NodePtr<ast::Expr>,
    },
    ObjectPrimitive {
        obj: Node<ObjectId>,
        prim: Primitive,
    },
    Evaluating {
        entry_point: Node<()>,
    },
    Evaluated(Value),
    Failed,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ThunkId(usize);

#[derive(Debug)]
pub struct Context {
    thunks: Vec<Thunk>,
    objects: Vec<Object>,
    scopes: Vec<Scope>,
    errors: Vec<EvalError>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            thunks: Vec::new(),
            objects: Vec::new(),
            scopes: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn create_object(&mut self, kind: String) -> ObjectId {
        let id = ObjectId(self.objects.len());
        self.objects.push(Object {
            kind,
            id: None,
            attrs: HashMap::new(),
            children: Vec::new(),
        });
        id
    }

    pub fn object_mut(&mut self, id: ObjectId) -> &mut Object {
        &mut self.objects[id.0]
    }

    pub fn create_thunk(&mut self, thunk: Thunk) -> ThunkId {
        let id = ThunkId(self.thunks.len());
        self.thunks.push(thunk);
        id
    }

    pub fn create_scope(&mut self, parent: Option<ScopeId>) -> ScopeId {
        let id = ScopeId(self.scopes.len());
        self.scopes.push(Scope::new(parent));
        id
    }

    pub fn bind(&mut self, scope: ScopeId, var: &Node<String>, value: ThunkId) {
        if self.scopes[scope.0]
            .bindings
            .insert(var.data.clone(), value)
            .is_some()
        {
            self.error(
                var,
                format!("'{}' defined more than once in scope {}", var.data, scope.0),
            );
        }
    }

    pub fn lookup(&mut self, scope: ScopeId, var: &Node<&str>) -> Option<ThunkId> {
        let mut current = Some(scope);
        while let Some(here) = current {
            if let Some(value) = self.scopes[here.0].bindings.get(var.data) {
                return Some(*value);
            }
            current = self.scopes[here.0].parent;
        }
        self.error(var, format!("Variable {} not defined", var.data));
        None
    }

    pub fn error<T, S: Into<String>>(&mut self, source: &Node<T>, message: S) {
        self.errors.push(EvalError {
            span: source.span.clone(),
            pos: source.pos.clone(),
            message: message.into(),
        })
    }

    pub fn errors(&self) -> &[EvalError] {
        &self.errors
    }

    pub fn eval(&mut self, node: &Node<ast::Root>) -> Vec<ObjectId> {
        let root_scope = self.create_scope(None);
        let objects = node
            .data
            .objects
            .iter()
            .map(|child| self.build_hierarchy(child, root_scope))
            .collect();
        self.force_all();
        objects
    }

    /// Traverse the object tree and register all id'd objects in the given scope.
    fn build_hierarchy(&mut self, node: &Node<ast::Object>, scope: ScopeId) -> ObjectId {
        let obj = self.create_object(node.data.name.data.clone());

        let mut id: Option<Node<String>> = None;
        let mut attrs: HashMap<String, ThunkId> = HashMap::new();

        // Insert user-defined attributes
        for attr in &node.data.attrs {
            if attr.data.name.data == "id" {
                if let ast::Expr::Var(var) = &attr.data.value.data {
                    if id.is_none() {
                        id = Some(Node {
                            span: attr.data.value.span.clone(),
                            pos: attr.data.value.pos.clone(),
                            data: var.into(),
                        });
                    } else {
                        self.error(attr, "'id' set more than once");
                    }
                } else {
                    self.error(attr, "'id' attribute must be an identifier");
                }
            } else {
                let name = attr.data.name.data.clone();
                let value = self.create_thunk(Thunk::Unevaluated {
                    scope,
                    expr: attr.data.value.clone(),
                });
                if let Some(old) = attrs.insert(name, value) {
                    // restore old value
                    *attrs.get_mut(&attr.data.name.data).unwrap() = old;
                    self.error(
                        attr,
                        format!("'{}' set more than once", attr.data.name.data),
                    );
                }
            }
        }

        // Type-specific handling of objects
        match node.data.name.data.as_str() {
            "Sequence" => {
                let has_start = attrs.contains_key("start");
                let has_end = attrs.contains_key("end");
                if has_start && has_end {
                    self.error(
                        node,
                        "Sequence cannot have both `start` and `end` attributes",
                    );
                } else if has_end {
                    todo!("Dynamic start attribute")
                } else {
                    if !has_start {
                        self.error(node, "Sequence has neither `start` nor `end` attribute");
                        // Default to beginning of song
                        let start =
                            self.create_thunk(Thunk::Evaluated(Value::Ratio(Rational::ZERO)));
                        attrs.insert("start".into(), start);
                    }
                    let end = self.create_thunk(Thunk::ObjectPrimitive {
                        obj: Node {
                            span: node.span.clone(),
                            pos: node.pos.clone(),
                            data: obj,
                        },
                        prim: Primitive(builtins::sequence_end),
                    });
                    attrs.insert("end".into(), end);
                }
            }
            // Regular object without special behavior
            _ => {}
        }

        if let Some(id) = &id {
            let object_value = self.create_thunk(Thunk::Evaluated(Value::Object(obj)));
            self.bind(scope, id, object_value);
        }

        let children: Vec<ObjectId> = node
            .data
            .children
            .iter()
            .map(|child| self.build_hierarchy(child, scope))
            .collect();

        let mut_object = self.object_mut(obj);
        mut_object.id = id.map(|node| node.data);
        mut_object.attrs = attrs;
        mut_object.children = children;

        obj
    }

    /// Evaluate all thunks.
    fn force_all(&mut self) {
        let mut index = 0;
        // Evaluating thunks may create new thunks,
        // hence we cannot use for ... in 0..self.thunks.len()
        while index < self.thunks.len() {
            self.force_thunk(ThunkId(index));
            index += 1;
        }
    }

    fn force_thunk(&mut self, thunk_id: ThunkId) -> Option<Value> {
        match &self.thunks[thunk_id.0] {
            Thunk::Unevaluated { scope, expr } => {
                let expr = expr.clone();
                let entry_point = expr.replace(());
                let scope = *scope;
                self.thunks[thunk_id.0] = Thunk::Evaluating { entry_point };
                if let Some(result) = self.eval_expr(&expr, scope) {
                    self.thunks[thunk_id.0] = Thunk::Evaluated(result.clone());
                    Some(result)
                } else {
                    self.error(&expr, format!("Failed to evaluate thunk {}", thunk_id.0));
                    self.thunks[thunk_id.0] = Thunk::Failed;
                    None
                }
            }
            Thunk::ObjectPrimitive { obj, prim } => {
                let obj = obj.clone();
                let prim = *prim;
                let entry_point = obj.replace(());
                self.thunks[thunk_id.0] = Thunk::Evaluating { entry_point };
                if let Some(result) = prim.0(self, obj.clone()) {
                    self.thunks[thunk_id.0] = Thunk::Evaluated(result.clone());
                    Some(result)
                } else {
                    self.error(
                        &obj,
                        format!("Failed to evaluate thunk {}.{:?}", thunk_id.0, prim),
                    );
                    self.thunks[thunk_id.0] = Thunk::Failed;
                    None
                }
            }
            Thunk::Evaluating { entry_point } => {
                let entry_point = entry_point.clone(); // appease the borrow checker
                self.error(
                    &entry_point,
                    format!("Evaluating thunk {} caused endless recursion", thunk_id.0),
                );
                None
            }
            Thunk::Evaluated(value) => Some(value.clone()),
            Thunk::Failed => None,
        }
    }

    fn eval_expr(&mut self, expr: &Node<ast::Expr>, scope: ScopeId) -> Option<Value> {
        // TODO: fill remaining cases
        match &expr.data {
            ast::Expr::String(x) => Some(Value::String(x.clone())),
            ast::Expr::Int(x) => Some(Value::Int(x.clone())),
            ast::Expr::Ratio(x) => Some(Value::Ratio(x.clone())),
            ast::Expr::Float(x) => Some(Value::Float(x.into_inner())),
            ast::Expr::Bool(x) => Some(Value::Bool(x.clone())),
            ast::Expr::Unary { operator, operand } => match operator.data {
                ast::UnaryOp::Plus => todo!("ast::UnaryOp::Plus"),
                ast::UnaryOp::Minus => todo!("ast::UnaryOp::Minus"),
                ast::UnaryOp::Not => {
                    let operand_value = self.eval_expr(operand, scope)?;
                    let b = self.expect_bool(operand, operand_value)?;
                    Some(Value::Bool(!b))
                }
            },
            ast::Expr::Binary {
                left,
                operator,
                right,
            } => match operator.data {
                ast::BinaryOp::Add => todo!("ast::BinaryOp::Add"),
                ast::BinaryOp::Sub => todo!("ast::BinaryOp::Sub"),
                ast::BinaryOp::Mult => todo!("ast::BinaryOp::Mult"),
                ast::BinaryOp::Div => todo!("ast::BinaryOp::Div"),
                ast::BinaryOp::And => {
                    let left_value = self.eval_expr(left, scope)?;
                    let l = self.expect_bool(left, left_value)?;
                    if l {
                        let right_value = self.eval_expr(right, scope)?;
                        let r = self.expect_bool(right, right_value)?;
                        Some(Value::Bool(r))
                    } else {
                        Some(Value::Bool(false))
                    }
                }
                ast::BinaryOp::Or => {
                    let left_value = self.eval_expr(left, scope)?;
                    let l = self.expect_bool(left, left_value)?;
                    if l {
                        Some(Value::Bool(true))
                    } else {
                        let right_value = self.eval_expr(right, scope)?;
                        let r = self.expect_bool(right, right_value)?;
                        Some(Value::Bool(r))
                    }
                }
            },
            ast::Expr::Paren { expr, .. } => self.eval_expr(expr, scope),
            ast::Expr::Object(node) => {
                // evaluate anonymous objects in their own scope
                let nested = self.create_scope(Some(scope));
                let object = self.build_hierarchy(&node, nested);
                Some(Value::Object(object))
            }
            ast::Expr::Var(var) => {
                let thunk = self.lookup(
                    scope,
                    &Node {
                        span: expr.span.clone(),
                        pos: expr.pos.clone(),
                        data: var.as_str(),
                    },
                )?;
                self.force_thunk(thunk)
            }
            ast::Expr::Accessor {
                expr, attribute, ..
            } => {
                let accessee = self.eval_expr(expr, scope)?;
                if let Some(obj) = accessee.as_object() {
                    if let Some(thunk) = self.objects[obj.0].attrs.get(&attribute.data).cloned() {
                        self.force_thunk(thunk)
                    } else {
                        self.error(attribute, "attribute missing");
                        None
                    }
                } else {
                    self.error(expr, "cannot access attribute of non-object");
                    None
                }
            }
            ast::Expr::Call {
                callee, arguments, ..
            } => todo!("ast::Expr::Call"),
            ast::Expr::Sequence(seq_expr) => {
                let mut sequence = Sequence::new();
                for sym in &seq_expr.data.symbols {
                    let item = match sym.data {
                        ast::SeqSym::Note { note, duration } => {
                            SeqItem::Note { note, duration }
                        }
                        ast::SeqSym::Rest { duration } => {
                            SeqItem::Rest { duration }
                        }
                        ast::SeqSym::Group(_) => todo!("ast::SeqSym::Group"),
                    };
                    sequence.add(item);
                }
                Some(Value::Sequence(sequence))
            },
        }
    }

    fn expect_bool<T>(&mut self, source: &Node<T>, value: Value) -> Option<bool> {
        let result = value.as_bool();
        if result.is_none() {
            self.error(source, "expected bool");
        }
        result
    }
}

#[derive(Debug)]
pub struct Scope {
    parent: Option<ScopeId>,
    bindings: HashMap<String, ThunkId>,
}

impl Scope {
    pub fn new(parent: Option<ScopeId>) -> Self {
        Self {
            parent,
            bindings: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScopeId(usize);

#[derive(Debug)]
pub struct EvalError {
    pub span: lexer::Span,
    pub pos: Range<Pos>,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Ratio(Rational),
    String(String),
    Bool(bool),
    Sequence(Sequence),
    Object(ObjectId),
}

impl Value {
    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_object(&self) -> Option<ObjectId> {
        if let Self::Object(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn try_into_ratio(self) -> Result<Rational, Self> {
        match self {
            Value::Int(i) => Ok(Rational::int(i)),
            Value::Ratio(r) => Ok(r),
            _ => Err(self),
        }
    }
}

#[derive(Clone, Copy)]
pub struct Primitive(fn(cxt: &mut Context, obj: Node<ObjectId>) -> Option<Value>);

impl Debug for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<primitive@{:p}>",
            self.0 as *const fn(&mut Context, ObjectId) -> Option<Value>
        )
    }
}

mod builtins {
    use syntxt_core::rational::Rational;

    use crate::ast::Node;

    use super::{Context, ObjectId, Value};

    pub fn sequence_end(cxt: &mut Context, obj: Node<ObjectId>) -> Option<Value> {
        let start = if let Some(thunk) = cxt.objects[obj.data.0].attrs.get("start").cloned() {
            if let Ok(r) = cxt.force_thunk(thunk)?.try_into_ratio() {
                r
            } else {
                cxt.error(
                    &obj,
                    format!(
                        "`start` attribute of object {} is not a rational",
                        obj.data.0
                    ),
                );
                return None;
            }
        } else {
            cxt.error(
                &obj,
                format!("Object {} does not have a `start` attribute", obj.data.0),
            );
            return None;
        };

        // TODO: take actual length
        let len = Rational::int(1);

        Some(Value::Ratio(start + len))
    }
}
