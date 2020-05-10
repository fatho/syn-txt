// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

use syn_txt::lang::compiler;
use syn_txt::lang::debug;
use syn_txt::lang::heap;
use syn_txt::lang::interpreter::*;
use syn_txt::lang::pretty;
use syn_txt::lang::span::LineMap;
use syn_txt::lang::value::*;

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
    println!("{}", std::mem::size_of::<syn_txt::lang::Value>());
    println!(
        "{}",
        std::mem::size_of::<syn_txt::lang::Gc<syn_txt::lang::Value>>()
    );
    println!(
        "{}",
        std::mem::size_of::<Option<syn_txt::lang::Gc<syn_txt::lang::Value>>>()
    );

    run_test(input)
}

fn run_test(input: &str) {
    let mut heap = heap::Heap::new();
    let mut debug = debug::DebugTable::new();
    let values = compiler::compile_str(&mut heap, &mut debug, "<input>", input).unwrap();

    log::info!("evaluating <input>");
    let mut int = Interpreter::new(&mut heap, &mut debug);

    for v in values {
        println!("In:\n{}", pretty::pretty(&v));
        println!();

        match int.eval(v) {
            Ok(val) => {
                println!("Out:\n{}", pretty::pretty(&val.pin()));
            }
            Err(err) => {
                let source = err
                    .location()
                    .and_then(|loc| int.debug_info().get_source(&loc.file));
                let lines = source.map(LineMap::new);
                print_error(lines.as_ref(), err.location(), err.info());
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
    println!("GC values remaining: {}", heap.len());
}

fn print_error<E: std::fmt::Display>(
    lines: Option<&LineMap>,
    location: Option<&debug::SourceLocation>,
    message: E,
) {
    print!("error: {}", message);
    if let Some(location) = location {
        if let Some(lines) = lines {
            let start = lines.offset_to_pos(location.span.begin);
            let end = lines.offset_to_pos(location.span.end);
            println!("({} {}-{})", location.file, start, end);
            println!("{}", lines.highlight(start, end, true));
        } else {
            println!(
                "({} {}-{})",
                location.file, location.span.begin, location.span.end
            );
        }
    }
}
