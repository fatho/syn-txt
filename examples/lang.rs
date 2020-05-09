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
use syn_txt::lang2::value::*;
use syn_txt::lang2::interpreter::*;
use syn_txt::lang2::compiler;
use syn_txt::lang2::heap;
use syn_txt::lang2::debug;
use syn_txt::lang2::pretty;
use syn_txt::lang2::lexer::Lexer;
use syn_txt::lang2::parser::Parser;
use syn_txt::lang2::span::{LineMap, Span};

fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let input = r#"
        (print 5 1/3 "foo")
        (define r 5)
        r
        (define area
            (begin
                (define pi 3.14)
                (define result (* pi (* r r)))
                (set! r (+ r 1))
                result
            ))
        (print area)
        ; TODO: external callables
        ; (define f1 (foo/new))
        ; (define f2 (foo/new))
        ; (print f1 f2)
        ; (f1)

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

        (define (map f l)
            (if (cons? l)
                (cons
                    (f (head l))
                    (map f (tail l))
                )
                nil
            )
        )

        (define reverse
            (begin
                (define (reverse-impl l acc)
                    (if (cons? l)
                        (reverse-impl
                            (tail l)
                            (cons (head l) acc)
                        )
                        acc
                    )
                )
                (lambda (l) (reverse-impl l nil))
            )
        )

        (define foo (list 1 2 3 4))
        (map (lambda (x) (+ 1 x)) (reverse foo))
        ; (define (cat-rev l) (concat l (reverse l)))
        ; (print (cat-rev foo))
        ; (for-each print (map (lambda (x) (+ 1 x)) (cat-rev foo)))
    "#;

    println!("{}", std::mem::size_of::<Value>());
    println!("{}", std::mem::size_of::<syn_txt::lang2::Value>());
    println!("{}", std::mem::size_of::<syn_txt::lang2::Gc<syn_txt::lang2::Value>>());
    println!("{}", std::mem::size_of::<Option<syn_txt::lang2::Gc<syn_txt::lang2::Value>>>());

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
                print_error(&lines, mk_loc(err.location()).as_ref(), err.kind());
            }
        }
    }

    println!("Parsing...");
    let mut parser = Parser::new(input, &tokens);
    // println!("{:?}", parser.parse());
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(err) => {
            print_error(&lines, mk_loc(err.location()).as_ref(), err.info());
            return;
        }
    };

    println!("Compiling...");
    let mut heap = heap::Heap::new();
    let mut debug = debug::DebugTable::new();
    let mut context = compiler::Context {
        debug_table: &mut debug,
        heap: &mut heap,
        filename: "<input>".into(),
    };
    let values: Vec<heap::Gc<Value>> = ast.iter().map(|e| context.compile(e)).collect();

    println!("Evaluating...");
    let mut int = Interpreter::new(&mut heap, &mut debug);
    // let extension_state = Rc::new(RefCell::new(0));
    // int.register_primop_ext("foo/new", move |intp, args| {
    //     foo_ext_foo_new(&mut *extension_state.borrow_mut(), intp, args)
    // })
    // .unwrap();

    for v in values {
        print!("at ");
        let dbg_loc = int.debug_info().get_location(v.id());
        if let Some(loc) = dbg_loc {
            let start = lines.offset_to_pos(loc.span.begin);
            let end = lines.offset_to_pos(loc.span.end);
            println!("{} {}-{}", loc.file, start, end);
        } else {
            println!("unknown location");
        }
        println!("{}", pretty::pretty(&v.pin()));
        println!();

        match int.eval(v) {
            Ok(val) => {
                println!("{}", pretty::pretty(&val.pin()));
            },
            Err(err) => {
                print_error(&lines, err.location(), err.info());
                break;
            }
        }
        println!("----------------------------");
    }
    drop(int);

    println!("GC values: {}", heap.len());
    heap.gc_non_cycles();
    println!("GC values in cycles: {}", heap.len());
    heap.gc_cycles();
    println!("Heap: {:?}", heap);
}

fn mk_loc(span: Span) -> Option<debug::SourceLocation> {
    Some(debug::SourceLocation{
        file: "<input>".into(),
        span,
    })
}

fn print_error<E: std::fmt::Display>(lines: &LineMap, location: Option<&debug::SourceLocation>, message: E) {
    if let Some(location) = location {
        let start = lines.offset_to_pos(location.span.begin);
        let end = lines.offset_to_pos(location.span.end);
        println!("error: {} ({} {}-{})", message, location.file, start, end);
        println!("{}", lines.highlight(start, end, true));
    } else {
        println!("error: {}", message)
    }
}

// fn foo_ext_foo_new(
//     state: &mut usize,
//     _intp: &mut Interpreter,
//     args: ArgParser,
// ) -> InterpreterResult<Value> {
//     args.done()?;
//     let foo = FooVal(*state);
//     *state += 1;
//     Ok(Value::ext(foo))
// }

// #[derive(Debug, PartialEq, Clone)]
// struct FooVal(usize);

// impl ExtensionValue for FooVal {
//     fn partial_eq(&self, other: &dyn ExtensionValue) -> bool {
//         if let Some(foo) = other.as_any().downcast_ref::<Self>() {
//             self == foo
//         } else {
//             false
//         }
//     }

//     fn as_any(&self) -> &dyn std::any::Any {
//         self
//     }

//     fn call(&self, _intp: &mut Interpreter, args: ArgParser) -> InterpreterResult<Value> {
//         args.done()?;
//         Ok(Value::Int(self.0 as i64))
//     }
// }
