// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

use std::{cell::RefCell, rc::Rc};
use syn_txt::lang::interpreter::*;
use syn_txt::lang::lexer::Lexer;
use syn_txt::lang::parser::Parser;
use syn_txt::lang::span::{LineMap, Span};

fn main() {
    let input = r#"
        (define r 5)
        (define area
            (begin
                (define pi 3.14)
                (define result (* pi (* r r)))
                (set! r (+ r 1))
                result
            ))
        (print area)
        (define f1 (foo/new))
        (define f2 (foo/new))
        (print f1 f2)
        (f1)

        (define plus-one
            (lambda (x) (+ x 1)))
        (plus-one 2)

        (define global-state 0)
        (define get-global
            (lambda ()
                (begin
                    (define ret global-state)
                    (set! global-state (+ ret 1))
                    ret
                )
            )
        )
        (get-global)
        (get-global)
        (get-global)
        global-state

        (define foo (list 1 2 3 4))
        (define (cat-rev l) (concat l (reverse l)))
        (print (cat-rev foo))
        (for-each print (map (lambda (x) (+ 1 x)) (cat-rev foo)))
    "#;

    println!("{}", std::mem::size_of::<Value>());

    run_test(input)
}

fn run_test(input: &str) {
    let mut lex = Lexer::new(input);
    let mut tokens = Vec::new();
    let lines = LineMap::new(input);

    println!("Lexing...");
    while let Some(token_or_error) = lex.next_token() {
        match token_or_error {
            Ok(tok) => tokens.push(tok),
            Err(err) => {
                print_error(&lines, err.location(), err.kind());
            }
        }
    }

    println!("Parsing...");
    let mut parser = Parser::new(input, &tokens);
    // println!("{:?}", parser.parse());
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(err) => {
            print_error(&lines, err.location(), err.info());
            return;
        }
    };

    println!("Evaluating...");
    let mut int = Interpreter::new();
    let extension_state = Rc::new(RefCell::new(0));
    int.register_primop_ext("foo/new", move |intp, args| {
        foo_ext_foo_new(&mut *extension_state.borrow_mut(), intp, args)
    })
    .unwrap();

    for s in ast {
        println!("{}", &input[s.src.begin..s.src.end]);
        match int.eval(&s) {
            Ok(val) => println!("  {:?}", val),
            Err(err) => {
                print_error(&lines, err.location(), err.info());
                return;
            }
        }
    }
}

fn print_error<E: std::fmt::Display>(lines: &LineMap, location: Span, message: E) {
    let start = lines.offset_to_pos(location.begin);
    let end = lines.offset_to_pos(location.end);
    println!("error: {} (<input>:{}-{})", message, start, end);
    println!("{}", lines.highlight(start, end, true));
}

fn foo_ext_foo_new(
    state: &mut usize,
    _intp: &mut Interpreter,
    args: ArgParser,
) -> InterpreterResult<Value> {
    args.done()?;
    let foo = FooVal(*state);
    *state += 1;
    Ok(Value::ext(foo))
}

#[derive(Debug, PartialEq, Clone)]
struct FooVal(usize);

impl ExtensionValue for FooVal {
    fn partial_eq(&self, other: &dyn ExtensionValue) -> bool {
        if let Some(foo) = other.as_any().downcast_ref::<Self>() {
            self == foo
        } else {
            false
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn call(&self, _intp: &mut Interpreter, args: ArgParser) -> InterpreterResult<Value> {
        args.done()?;
        Ok(Value::Int(self.0 as i64))
    }
}
