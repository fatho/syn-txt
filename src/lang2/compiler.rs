//! The compiler turns the AST into values.

use super::ast;
use super::heap;
use super::{debug, span::Span, Value};

use std::rc::Rc;

/// Surrounding context needed for compiling.
pub struct Context<'a> {
    pub heap: &'a mut heap::Heap,
    pub debug_table: &'a mut debug::DebugTable,
    pub filename: Rc<str>,
}

impl<'a> Context<'a> {
    pub fn compile(&mut self, expr: &ast::SymExpSrc) -> heap::Gc<Value> {
        let value = self.compile_exp(&expr.exp);
        let gced = self.heap.alloc(value);
        self.debug_table.insert(
            gced.id(),
            debug::DebugInfo {
                location: Some(self.make_location(expr.src)),
            },
        );
        gced
    }

    pub fn compile_exp(&mut self, expr: &ast::SymExp) -> Value {
        match expr {
            // TODO: intern symbols
            ast::SymExp::Keyword(sym) => Value::Keyword(sym.0.as_ref().into()),
            ast::SymExp::Variable(sym) => Value::Symbol(sym.0.as_ref().into()),
            ast::SymExp::Str(s) => Value::Str(s.as_ref().into()),
            ast::SymExp::Float(x) => Value::Float(*x),
            ast::SymExp::Ratio(x) => Value::Ratio(*x),
            ast::SymExp::Int(x) => Value::Int(*x),
            // TODO: find a way for a more efficient representation of list again
            ast::SymExp::List(xs) => {
                let mut list = Value::Nil;
                for x in xs.iter().rev() {
                    let head = self.compile(x);
                    let tail = self.heap.alloc(list);
                    list = Value::Cons(head, tail);
                }
                list
            }
        }
    }

    pub fn make_location(&self, span: Span) -> debug::SourceLocation {
        debug::SourceLocation {
            file: Rc::clone(&self.filename),
            span,
        }
    }
}
