// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! The compiler turns the AST into values.

use super::ast;
use super::heap;
use super::lexer;
use super::parser;
use super::{
    debug,
    span::{LineMap, Span},
    Value,
};

use std::{fmt, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileError {
    filename: Arc<str>,
    lexer_errors: Vec<lexer::LexerError>,
    parse_errors: Vec<parser::ParseError>,
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "failed to compile {}:", self.filename)?;
        for l in self.lexer_errors.iter() {
            writeln!(f, "{}", l.kind())?;
        }
        for p in self.parse_errors.iter() {
            writeln!(f, "{}", p.info())?;
        }
        Ok(())
    }
}

impl CompileError {
    pub fn new(
        filename: &str,
        lexer_errors: Vec<lexer::LexerError>,
        parse_errors: Vec<parser::ParseError>,
    ) -> Self {
        Self {
            filename: filename.into(),
            lexer_errors,
            parse_errors,
        }
    }

    pub fn location(&self, span: Span) -> debug::SourceLocation {
        debug::SourceLocation {
            file: self.filename.clone(),
            span,
        }
    }

    pub fn lexer_errors(&self) -> &[lexer::LexerError] {
        &self.lexer_errors
    }

    pub fn parse_errors(&self) -> &[parser::ParseError] {
        &self.parse_errors
    }
}

/// Surrounding context needed for compiling.
pub struct Context<'a> {
    pub heap: &'a mut heap::Heap,
    pub debug_table: &'a mut debug::DebugTable,
    pub filename: Arc<str>,
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
            file: Arc::clone(&self.filename),
            span,
        }
    }
}

pub fn compile_str<'a>(
    heap: &mut heap::Heap,
    debug_table: &mut debug::DebugTable,
    filename: &str,
    source: &str,
) -> Result<Vec<heap::GcPin<Value>>, CompileError> {
    let mut lex = lexer::Lexer::new(source);
    let mut tokens = Vec::new();
    let lines = LineMap::new(source);

    let mut lex_errs = Vec::new();

    // Prevent log spam when compiling internal modules. They are expected to work.
    let is_internal = filename.starts_with('<') && filename.ends_with('>');
    let status_level = if is_internal {
        log::Level::Debug
    } else {
        log::Level::Info
    };

    log::log!(status_level, "lexing {}", filename);
    while let Some(token_or_error) = lex.next_token() {
        match token_or_error {
            Ok(tok) => tokens.push(tok),
            Err(err) => {
                log_error(&lines, filename, err.location(), err.kind());
                lex_errs.push(err)
            }
        }
    }

    if !lex_errs.is_empty() {
        return Err(CompileError::new(filename, lex_errs, vec![]));
    }

    log::log!(status_level, "parsing {}", filename);
    let mut parser = parser::Parser::new(source, &tokens);

    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(err) => {
            log_error(&lines, filename, err.location(), err.info());
            return Err(CompileError::new(filename, lex_errs, vec![err]));
        }
    };

    log::log!(status_level, "compiling {}", filename);
    // TODO: make debug info configurable
    debug_table.insert_source(filename.into(), source.into());
    let mut context = Context {
        heap,
        debug_table,
        filename: filename.into(),
    };
    Ok(ast.iter().map(|exp| context.compile(exp).pin()).collect())
}

fn log_error<E: std::fmt::Display>(lines: &LineMap, input_name: &str, location: Span, message: E) {
    let start = lines.offset_to_pos(location.begin);
    let end = lines.offset_to_pos(location.end);
    log::error!(
        "error: {} ({}:{}-{})\n{}",
        message,
        input_name,
        start,
        end,
        lines.highlight(start, end, true)
    );
}
