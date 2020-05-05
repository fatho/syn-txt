//! The module responsible for evaluating syn.txt files to an abstract description of a song.

use log::{error, info};
use std::io;

use crate::lang::ast::SymExpSrc;
use crate::lang::interpreter::{FromValue, Interpreter, Value};
use crate::lang::lexer::{Lexer, Token};
use crate::lang::parser::Parser;
use crate::lang::span::{LineMap, Span};

use super::langext;

/// Evaluate syn.txt source code into a song description.
///
/// TODO: allow including other files.
pub fn eval(input_name: &str, input: &str) -> io::Result<langext::SongValue> {
    let lines = LineMap::new(input);

    info!("lexing {}", input_name);
    let tokens = lex(input_name, &lines)?;

    info!("parsing {}", input_name);
    let ast = parse(input_name, &lines, &tokens)?;

    info!("evaluating {}", input_name);

    let mut int = create_interpreter()?;

    let mut final_value = Value::Unit;
    for s in ast {
        match int.eval(&s) {
            Ok(val) => final_value = val,
            Err(err) => {
                log_error(&lines, input_name, err.location(), err.info());
                return Err(io::ErrorKind::InvalidData.into());
            }
        }
    }

    langext::SongValue::from_value(final_value)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "not a song"))
}

pub fn lex(input_name: &str, lines: &LineMap) -> io::Result<Vec<(Span, Token)>> {
    let mut lex = Lexer::new(lines.source());
    let mut tokens = Vec::new();
    let mut has_errors = false;
    while let Some(token_or_error) = lex.next_token() {
        match token_or_error {
            Ok(tok) => tokens.push(tok),
            Err(err) => {
                log_error(&lines, input_name, err.location(), err.kind());
                has_errors = true;
            }
        }
    }
    if has_errors {
        return Err(io::ErrorKind::InvalidData.into());
    }
    Ok(tokens)
}

pub fn parse(
    input_name: &str,
    lines: &LineMap,
    tokens: &[(Span, Token)],
) -> io::Result<Vec<SymExpSrc>> {
    let mut parser = Parser::new(lines.source(), tokens);
    parser.parse().map_err(|parse_error| {
        log_error(
            &lines,
            input_name,
            parse_error.location(),
            parse_error.info(),
        );
        io::ErrorKind::InvalidData.into()
    })
}

/// Initialize the interpreter with the music language extensions.
fn create_interpreter() -> io::Result<Interpreter> {
    let mut int = Interpreter::new();
    for (name, op) in langext::PRIMOPS {
        int.register_primop(name, op.0)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{}", e.info())))?;
    }
    Ok(int)
}

fn log_error<E: std::fmt::Display>(lines: &LineMap, input_name: &str, location: Span, message: E) {
    let start = lines.offset_to_pos(location.begin);
    let end = lines.offset_to_pos(location.end);
    error!(
        "error: {} ({}:{}-{})\n{}",
        message,
        input_name,
        start,
        end,
        lines.highlight(start, end, true)
    );
}
