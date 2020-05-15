// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

use super::heap::Gc;
use super::value::*;
use std::fmt::Write;

struct PrettyPrinter {
    output: String,
    indent: usize,
}

impl PrettyPrinter {
    fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
        }
    }

    fn print(&mut self, value: &Value) {
        match value {
            Value::Symbol(x) => self.output.push_str(x.as_str()),
            Value::Keyword(x) => self.output.push_str(x.as_str()),
            Value::Str(x) => write!(&mut self.output, "{:?}", x).unwrap(),
            Value::Float(x) => write!(&mut self.output, "{}", x).unwrap(),
            Value::Ratio(x) => write!(&mut self.output, "{}", x).unwrap(),
            Value::Int(x) => write!(&mut self.output, "{}", x).unwrap(),
            Value::Bool(x) => write!(&mut self.output, "{}", x).unwrap(),
            Value::Void => self.output.push_str("<<<void>>>"),
            Value::Nil => self.output.push_str("'()"),
            Value::Cons(head, tail) => self.print_list(head, tail),
            Value::Closure(_) => self.output.push_str("<<<closure>>>"),
            Value::PrimOp(_) => self.output.push_str("<<<prim-op>>>"),
            Value::Dict(d) => self.print_dict(d),
        }
    }

    fn print_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push(' ');
        }
    }

    fn print_list(&mut self, head: &Gc<Value>, tail: &Gc<Value>) {
        self.output.push('(');
        self.print(&head.pin());
        self.indent += 2;

        let mut current = tail.pin();
        while let Value::Cons(value, tail) = &*current {
            self.output.push('\n');
            self.print_indent();
            self.print(&value.pin());
            current = tail.pin();
        }
        // Dotted list, but there's no input syntax for this yet
        if !current.is_nil() {
            self.output.push_str(" . ");
            self.print(&current);
        }
        self.output.push(')');
        self.indent -= 2;
    }

    fn print_dict(&mut self, d: &std::collections::HashMap<Symbol, Gc<Value>>) {
        self.output.push_str("(dict");
        self.indent += 2;

        let mut sorted_entries: Vec<_> = d.iter().collect();
        sorted_entries.sort_by_key(|(k, _)| k.as_str());

        for (key, value) in sorted_entries {
            self.output.push('\n');
            self.print_indent();
            self.output.push_str(key.as_str());
            self.output.push(' ');
            self.print(&value.pin());
        }
        self.output.push(')');
        self.indent -= 2;
    }
}

pub fn pretty(value: &Value) -> String {
    let mut printer = PrettyPrinter::new();
    printer.print(value);
    printer.output
}
