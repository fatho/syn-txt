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

use std::{
    collections::HashMap,
    ops::Range,
};

use syntxt_core::rational::Rational;

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
    Unevaluated(NodePtr<ast::Expr>),
    Evaluating,
    Evaluated(Value),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ThunkId(usize);

#[derive(Debug)]
pub struct Context {
    thunks: Vec<Thunk>,
    objects: Vec<Object>,
    errors: Vec<EvalError>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            thunks: Vec::new(),
            objects: Vec::new(),
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

    pub fn create_thunk(&mut self, expr: NodePtr<ast::Expr>) -> ThunkId {
        let id = ThunkId(self.thunks.len());
        self.thunks.push(Thunk::Unevaluated(expr));
        id
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
        node.data
        .objects
        .iter()
        .map(|child| self.build_hierarchy(child))
        .collect()
    }

    fn build_hierarchy(&mut self, node: &Node<ast::Object>) -> ObjectId {
        let obj = self.create_object(node.data.name.data.clone());

        let mut id: Option<String> = None;
        let mut attrs: HashMap<String, ThunkId> = HashMap::new();

        for attr in &node.data.attrs {
            if attr.data.name.data == "id" {
                if let ast::Expr::Var(var) = &attr.data.value.data {
                    if id.is_none() {
                        id = Some(var.into())
                    } else {
                        self.error(attr, "'id' set more than once");
                    }
                } else {
                    self.error(attr, "'id' attribute must be an identifier");
                }
            } else {
                let name = attr.data.name.data.clone();
                if attrs.contains_key(&name) {
                    self.error(attr, format!("'{}' set more than once", name));
                } else {
                    let value = self.create_thunk(attr.data.value.clone());
                    attrs.insert(name, value);
                }
            }
        }

        let children: Vec<ObjectId> = node
            .data
            .children
            .iter()
            .map(|child| self.build_hierarchy(child))
            .collect();

        let mut_object = self.object_mut(obj);
        mut_object.id = id;
        mut_object.attrs = attrs;
        mut_object.children = children;

        obj
    }
}

#[derive(Debug)]
pub struct EvalError {
    pub span: lexer::Span,
    pub pos: Range<Pos>,
    pub message: String,
}

#[derive(Debug)]
pub enum Value {
    Int(i64),
    Float(f64),
    Ratio(Rational),
    String(String),
    Object(ObjectId),
}


#[allow(unused)]
mod v1 {

    use std::{
        any::{Any, TypeId},
        collections::HashMap,
        marker::PhantomData,
        rc::Rc,
    };

    use syntxt_core::rational::Rational;

    use crate::ast::{self, Node, NodePtr};

    /// Holds the data needed during evaluation.
    pub struct Context {
        thunks: Vec<Thunk>,
        objects: Vec<Rc<dyn Object>>,
        objects_by_name: HashMap<Rc<str>, ObjectRef>,
    }

    pub struct ObjectRef(usize);

    pub enum Thunk {
        Unevaluated(NodePtr<ast::Expr>),
        Evaluating,
        Evaluated(Value),
    }

    pub enum Value {
        Int(i64),
        Float(f64),
        Ratio(Rational),
        Sequence(ast::Sequence),
        Object(ObjectRef),
    }

    pub struct ValueRef<T> {
        id: usize,
        _type: PhantomData<T>,
    }

    // pub struct Object {
    //     id: String,
    // }

    pub struct Song {}

    pub struct Meta {}

    pub fn eval(source: &Node<ast::Root>, cxt: &mut Context) {}

    fn constructor<T, F>(concrete: F) -> impl Fn(&mut Context) -> Box<dyn Object>
    where
        T: Object,
        F: Fn(&mut Context) -> T + 'static,
    {
        move |cxt| Box::new(concrete(cxt))
    }

    pub trait Object: Any {
        fn type_name(&self) -> &str;
    }

    impl dyn Object {
        pub fn is<T: 'static>(&self) -> bool {
            TypeId::of::<T>() == self.type_id()
        }

        pub fn downcast_rc<T: 'static>(self: Rc<Self>) -> Result<Rc<T>, Rc<Self>> {
            if self.is::<T>() {
                unsafe { Ok(Rc::from_raw(Rc::into_raw(self) as _)) }
            } else {
                Err(self)
            }
        }

        pub fn downcast_box<T: 'static>(self: Box<Self>) -> Result<Box<T>, Box<Self>> {
            if self.is::<T>() {
                unsafe { Ok(Box::from_raw(Box::into_raw(self) as _)) }
            } else {
                Err(self)
            }
        }
    }
}
