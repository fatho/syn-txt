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

use std::{fmt::Display, iter::Peekable, ops::Range, str::FromStr, sync::Arc};

use crate::ast::{self, Node};
use logos::Logos;
use syntxt_core::{
    note::{Accidental, Note, NoteName},
    rational::Rational,
};

use crate::{
    lexer::{Span, Token},
    line_map::{LineMap, Pos},
};

#[cfg(test)]
mod expect_tests;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    pub span: Span,
    pub pos: Range<Pos>,
    pub message: String,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Prec(u16);

impl Prec {
    const LOWEST: Prec = Prec(0);
    const DISJUNCTIVE: Prec = Prec(1);
    const CONJUNCTIVE: Prec = Prec(2);
    const ADDITIVE: Prec = Prec(3);
    const MULTIPLICATIVE: Prec = Prec(4);
    const UNARY: Prec = Prec(5);
    const CALL: Prec = Prec(6);
    const DOT: Prec = Prec(7);
    const HIGHEST: Prec = Prec(8);

    pub fn succ(self) -> Prec {
        // this would be a parser bug:
        assert!(self < Self::HIGHEST);
        Prec(self.0 + 1)
    }
}

pub type Parse<T> = Result<Node<T>, ParseError>;

pub struct Parser<'a> {
    source: &'a str,
    stream: Peekable<logos::SpannedIter<'a, Token>>,
    consumed: usize,
    line_map: LineMap<'a>,
    errors: Vec<ParseError>,
}

impl<'a> Parser<'a> {
    // Public interface
    pub fn parse(source: &'a str) -> Result<Node<ast::Root>, (Node<ast::Root>, Vec<ParseError>)> {
        let mut parser = Parser::new(source);
        let root = parser.parse_root();
        if parser.errors.is_empty() {
            Ok(root)
        } else {
            Err((root, parser.errors))
        }
    }

    // Private helpers

    fn new(source: &'a str) -> Self {
        Parser {
            source,
            stream: Token::lexer(source).spanned().peekable(),
            consumed: 0,
            line_map: LineMap::new(source),
            errors: Vec::new(),
        }
    }

    fn peek(&mut self) -> (Option<Token>, Span) {
        if let Some((tok, span)) = self.stream.peek().cloned() {
            (Some(tok), span)
        } else {
            (None, self.eof())
        }
    }

    fn consume(&mut self) -> Option<(Token, Span)> {
        let result = self.stream.next();
        if let Some((_, span)) = &result {
            self.consumed = span.end;
        }
        result
    }

    fn parse_expect_token(&mut self, expected: Token) -> Parse<()> {
        if let Some((token, span)) = self.consume() {
            if token == expected {
                return Ok(self.make_node(span, ()));
            } else {
                return Err(self.expected_but_got(span, &[expected], token));
            }
        } else {
            return Err(self.unexpected_eof(&[expected]));
        }
    }

    fn eof(&self) -> Span {
        self.source.len()..self.source.len()
    }

    fn make_node<T>(&self, span: Span, data: T) -> Node<T> {
        Node {
            pos: self.line_map.offset_to_pos(span.start)..self.line_map.offset_to_pos(span.end),
            span,
            data,
        }
    }

    fn make_error(&self, span: Span, message: String) -> ParseError {
        ParseError {
            pos: self.line_map.offset_to_pos(span.start)..self.line_map.offset_to_pos(span.end),
            message,
            span,
        }
    }

    pub fn expected_but_got(&self, span: Span, expected: &[Token], got: Token) -> ParseError {
        self.make_error(
            span,
            format!("Expected one of {:?}, but got {:?}", expected, got),
        )
    }

    pub fn expected_str_but_got(&self, span: Span, expected: &str, got: Token) -> ParseError {
        self.make_error(span, format!("Expected {}, but got {:?}", expected, got))
    }

    pub fn unexpected_eof(&self, expected: &[Token]) -> ParseError {
        self.make_error(
            self.eof(),
            format!("Expected one of {:?}, but reached end of file", expected),
        )
    }

    pub fn unexpected_str_eof(&self, span: Span, expected: &str) -> ParseError {
        self.make_error(
            span,
            format!("Expected one of {}, but reached end of file", expected),
        )
    }

    /// Recover from a parsing error by skipping ahead to the next line.
    fn recover_next_line<T>(&mut self, result: Parse<T>) -> Option<Node<T>> {
        self.recover_skip(result, true, &[])
    }

    /// Recover from a parse error by substituting a token
    fn recover_replace<T>(&mut self, result: Parse<T>, replacement: T) -> Node<T> {
        match result {
            Ok(node) => node,
            Err(error) => {
                let span = error.span.clone();
                self.errors.push(error);
                self.make_node(span, replacement)
            }
        }
    }

    /// Recover from a parse error by skipping until one of the given sync tokens or until
    /// the next line, if `sync_next_line` is set.
    fn recover_skip<T>(
        &mut self,
        result: Parse<T>,
        sync_next_line: bool,
        sync_tokens: &[Token],
    ) -> Option<Node<T>> {
        match result {
            Ok(node) => Some(node),
            Err(error) => {
                self.errors.push(error);
                self.skip_until_next_line_or_token(sync_next_line, sync_tokens);
                None
            }
        }
    }

    /// Eat tokens until we reach the next line. The parser itself isn't whitespace sensitive
    /// under normal conditions, but when recovering from an error, whitespace can be a good
    /// heuristic.
    fn skip_until_next_line(&mut self) {
        self.skip_until_next_line_or_token(true, &[])
    }

    /// Eat tokens until we reach the next line or one of the specified synchronisation tokens. The
    /// parser itself isn't whitespace sensitive under normal conditions, but when recovering from
    /// an error, whitespace can be a good heuristic.
    fn skip_until_next_line_or_token(&mut self, sync_line: bool, sync_tokens: &[Token]) {
        let start_line = self.line_map.offset_to_pos(self.consumed).line;
        while let (Some(token), span) = self.peek() {
            let current_line = self.line_map.offset_to_pos(span.start).line;
            if (sync_line && (current_line > start_line)) || sync_tokens.contains(&token) {
                break;
            } else {
                let _ = self.consume();
            }
        }
    }

    // Parse rules

    fn parse_root(&mut self) -> Node<ast::Root> {
        let mut objects = Vec::new();

        while self.peek().0.is_some() {
            let object_parse = self.parse_object();
            if let Some(object) = self.recover_next_line(object_parse) {
                objects.push(object)
            }
        }

        let span = objects.first().map_or(0, |obj| obj.span.start)
            ..objects.last().map_or(self.consumed, |obj| obj.span.end);
        self.make_node(span, ast::Root { objects })
    }

    fn parse_object(&mut self) -> Parse<ast::Object> {
        let name_parse = self.parse_ident();
        let name = self.recover_replace(name_parse, "<<unknown>>".into());
        self.parse_object_body(name)
    }

    fn parse_object_body(&mut self, name: Node<String>) -> Parse<ast::Object> {
        let lbrace = self.parse_expect_token(Token::LBrace)?;

        let mut attrs = Vec::new();
        let mut children = Vec::new();
        loop {
            match self.peek().0 {
                Some(Token::Ident) => {
                    let inner_name = self.parse_ident()?;
                    // Colon or brace
                    static EXPECTATION: &[Token] = &[Token::Colon, Token::LBrace];
                    let (token, span) = self.peek();
                    match token {
                        Some(Token::Colon) => {
                            let colon = self.parse_expect_token(Token::Colon)?;

                            let value_parse = self.parse_expr();
                            if let Some(value) = self.recover_next_line(value_parse) {
                                attrs.push(self.make_node(
                                    inner_name.span.start..value.span.end,
                                    ast::Attribute {
                                        name: inner_name,
                                        colon,
                                        value: Arc::new(value),
                                    },
                                ))
                            }
                        }
                        Some(Token::LBrace) => {
                            let child_object = self.parse_object_body(inner_name)?;
                            children.push(child_object);
                        }
                        Some(other) => {
                            self.errors
                                .push(self.expected_but_got(span, EXPECTATION, other));
                            self.skip_until_next_line();
                        }
                        None => {
                            self.errors.push(self.unexpected_eof(EXPECTATION));
                            break;
                        }
                    }
                }
                Some(Token::RBrace) => break,
                Some(other) => {
                    let (_, span) = self.consume().unwrap();
                    self.errors.push(self.expected_but_got(
                        span,
                        &[Token::Ident, Token::RBrace],
                        other,
                    ));
                    self.skip_until_next_line();
                }
                None => {
                    self.errors
                        .push(self.unexpected_eof(&[Token::Ident, Token::RBrace]));
                    break;
                }
            }
        }

        let rbrace = self.parse_expect_token(Token::RBrace)?;
        Ok(self.make_node(
            name.span.start..rbrace.span.end,
            ast::Object {
                name,
                lbrace,
                attrs,
                children,
                rbrace,
            },
        ))
    }

    fn parse_ident(&mut self) -> Parse<String> {
        let node = self.parse_expect_token(Token::Ident)?;
        Ok(self.make_node(node.span.clone(), self.source[node.span].into()))
    }

    fn parse_expr(&mut self) -> Parse<ast::Expr> {
        self.parse_prec_expr(Prec::LOWEST)
    }

    fn parse_prec_expr(&mut self, min_prec: Prec) -> Parse<ast::Expr> {
        // Prefix rules
        let mut left = self.parse_prefix_expr()?;

        // Infix/postfix rules
        loop {
            // Our expression could already be complete here, so EOF is a valid possibility
            let (token, span) = match self.peek() {
                (None, _) => break,
                (Some(token), span) => (token, span),
            };

            left = match token {
                // Infix operations
                Token::Or if min_prec <= Prec::DISJUNCTIVE => self.parse_binary_operand(
                    left,
                    self.make_node(span, ast::BinaryOp::Or),
                    Prec::DISJUNCTIVE.succ(),
                )?,
                Token::And if min_prec <= Prec::CONJUNCTIVE => self.parse_binary_operand(
                    left,
                    self.make_node(span, ast::BinaryOp::And),
                    Prec::CONJUNCTIVE.succ(),
                )?,
                Token::Plus if min_prec <= Prec::ADDITIVE => self.parse_binary_operand(
                    left,
                    self.make_node(span, ast::BinaryOp::Add),
                    Prec::ADDITIVE.succ(),
                )?,
                Token::Minus if min_prec <= Prec::ADDITIVE => self.parse_binary_operand(
                    left,
                    self.make_node(span, ast::BinaryOp::Sub),
                    Prec::ADDITIVE.succ(),
                )?,
                Token::Star if min_prec <= Prec::MULTIPLICATIVE => self.parse_binary_operand(
                    left,
                    self.make_node(span, ast::BinaryOp::Mult),
                    Prec::MULTIPLICATIVE.succ(),
                )?,
                Token::Slash if min_prec <= Prec::MULTIPLICATIVE => self.parse_binary_operand(
                    left,
                    self.make_node(span, ast::BinaryOp::Div),
                    Prec::MULTIPLICATIVE.succ(),
                )?,
                // Postfix operations
                Token::Dot if min_prec <= Prec::DOT => {
                    let dot = self.parse_expect_token(Token::Dot)?;
                    let attribute = self.parse_ident()?;
                    self.make_node(
                        left.span.start..attribute.span.end,
                        ast::Expr::Accessor {
                            expr: Arc::new(left),
                            dot,
                            attribute,
                        },
                    )
                }
                Token::LParen if min_prec <= Prec::CALL => {
                    let (lparen, arguments, rparen) =
                        self.parse_expr_list(Token::LParen, Token::RParen)?;
                    self.make_node(
                        left.span.start..rparen.span.end,
                        ast::Expr::Call {
                            callee: Arc::new(left),
                            lparen,
                            arguments,
                            rparen,
                        },
                    )
                }
                // any unexpected token is not consumed, this is a problem for the caller
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_prefix_expr(&mut self) -> Parse<ast::Expr> {
        let (token, span) = match self.peek() {
            (None, span) => return Err(self.unexpected_str_eof(span, "expression")),
            (Some(token), span) => (token, span),
        };
        match token {
            Token::Plus => self.parse_unary_operand(ast::UnaryOp::Plus, span),
            Token::Minus => self.parse_unary_operand(ast::UnaryOp::Minus, span),
            Token::Not => self.parse_unary_operand(ast::UnaryOp::Not, span),
            Token::LParen => self.parse_paren_expr(),
            Token::LLBracket => self.parse_sequence_expr(),
            Token::LitInt => self.parse_int_expr(),
            Token::LitFloat => self.parse_float_expr(),
            Token::LitRatio => self.parse_ratio_expr(),
            Token::LitString => self.parse_string_expr(),
            Token::LitBool => self.parse_bool_expr(),
            Token::Ident => {
                let name = self.parse_ident()?;

                // Can either be an object or a normal identifier, depending on what follows
                if let Some(Token::LBrace) = self.peek().0 {
                    let object = self.parse_object_body(name)?;
                    Ok(object.nest(ast::Expr::Object))
                } else {
                    Ok(name.map(ast::Expr::Var))
                }
            }
            _ => Err(self.expected_str_but_got(span, "expression", token)),
        }
    }

    fn parse_unary_operand(&mut self, op: ast::UnaryOp, op_span: Span) -> Parse<ast::Expr> {
        // assumes that the caller did not consume the operand yet
        self.consume();
        let expr = self.parse_prec_expr(Prec::UNARY)?;
        Ok(self.make_node(
            op_span.start..expr.span.end,
            ast::Expr::Unary {
                operator: self.make_node(op_span, op),
                operand: Arc::new(expr),
            },
        ))
    }

    fn parse_binary_operand(
        &mut self,
        left: Node<ast::Expr>,
        operator: Node<ast::BinaryOp>,
        right_prec: Prec,
    ) -> Parse<ast::Expr> {
        // assumes that the caller did not consume the operand yet
        self.consume();
        let expr = self.parse_prec_expr(right_prec)?;
        Ok(self.make_node(
            left.span.start..expr.span.end,
            ast::Expr::Binary {
                left: Arc::new(left),
                operator,
                right: Arc::new(expr),
            },
        ))
    }

    fn parse_paren_expr(&mut self) -> Parse<ast::Expr> {
        let lparen = self.parse_expect_token(Token::LParen)?;
        let expr = Arc::new(self.parse_expr()?);
        let rparen = self.parse_expect_token(Token::RParen)?;
        Ok(self.make_node(
            lparen.span.start..rparen.span.end,
            ast::Expr::Paren {
                lparen,
                expr,
                rparen,
            },
        ))
    }

    fn parse_expr_list(
        &mut self,
        start: Token,
        end: Token,
    ) -> Result<(Node<()>, Vec<Node<ast::Expr>>, Node<()>), ParseError> {
        let lparen = self.parse_expect_token(start)?;
        let mut arguments = Vec::new();
        while self.peek().0 != Some(end) {
            let expr_parse = self.parse_expr();
            if let Some(expr) = self.recover_skip(expr_parse, false, &[end, Token::Comma]) {
                arguments.push(expr);
            }

            let (token, span) = self.peek();
            match token {
                Some(Token::Comma) => {
                    self.parse_expect_token(Token::Comma)?;
                }
                Some(other) if other == end => break,
                Some(got) => {
                    let _ = self.consume();
                    self.errors
                        .push(self.expected_but_got(span, &[end, Token::Comma], got));
                    self.skip_until_next_line_or_token(false, &[end, Token::Comma]);
                    // The previous recovery doesn't consume the sync token, but if it is
                    // a comma, we actually need to consume it here so that the next loop
                    // iteration can resume parsing the argument expression.
                    if self.peek().0 == Some(Token::Comma) {
                        let _ = self.consume();
                    }
                }
                None => {
                    self.errors.push(self.unexpected_eof(&[end, Token::Comma]));
                    break;
                }
            }
        }

        let rparen_parse = self.parse_expect_token(end);
        let rparen = self.recover_replace(rparen_parse, ());
        Ok((lparen, arguments, rparen))
    }

    fn parse_native<T: FromStr>(&mut self, token: Token, ignore_underscores: bool) -> Parse<T>
    where
        T::Err: Display,
    {
        let node = self.parse_expect_token(token)?;
        let input = &self.source[node.span.clone()];
        let result = if ignore_underscores {
            input
                .chars()
                .filter(|ch| *ch != '_')
                .collect::<String>()
                .parse::<T>()
        } else {
            input.parse::<T>()
        };
        match result {
            Ok(int) => Ok(self.make_node(node.span, int)),
            Err(err) => Err(self.make_error(node.span, format!("{}", err))),
        }
    }

    fn parse_int_expr(&mut self) -> Parse<ast::Expr> {
        let int = self.parse_native(Token::LitInt, true)?;
        Ok(self.make_node(int.span, ast::Expr::Int(int.data)))
    }

    fn parse_ratio_expr(&mut self) -> Parse<ast::Expr> {
        let ratio = self.parse_native(Token::LitRatio, true)?;
        Ok(self.make_node(ratio.span, ast::Expr::Ratio(ratio.data)))
    }

    fn parse_float_expr(&mut self) -> Parse<ast::Expr> {
        let float = self.parse_native(Token::LitFloat, true)?;
        Ok(self.make_node(float.span, ast::Expr::Float(float.data)))
    }

    fn parse_bool_expr(&mut self) -> Parse<ast::Expr> {
        let bool = self.parse_native(Token::LitBool, false)?;
        Ok(self.make_node(bool.span, ast::Expr::Bool(bool.data)))
    }

    fn parse_string(&mut self) -> Parse<String> {
        // get literal text
        let node = self.parse_expect_token(Token::LitString)?;
        let lit_full = &self.source[node.span.clone()];

        // skip quotation marks
        assert!(
            lit_full.len() >= 2,
            "lexer bug: string literals must be at least two bytes"
        );
        let mut lit_remaining = lit_full[1..lit_full.len() - 1].char_indices();

        // parse string data
        let mut data = String::new();
        while let Some((index, ch)) = lit_remaining.next() {
            let start_index = node.span.start + 1 + index;
            if ch == '\\' {
                match lit_remaining.next() {
                    None => {
                        return Err(self.make_error(
                            start_index..start_index + 1,
                            "unterminated escape sequence".into(),
                        ))
                    }
                    Some((index, ch)) => {
                        let end_index = node.span.start + 1 + index + ch.len_utf8();
                        match ch {
                            'n' => data.push('\n'),
                            't' => data.push('\t'),
                            'r' => data.push('\r'),
                            '\\' => data.push('\\'),
                            _ => {
                                return Err(self.make_error(
                                    start_index..end_index,
                                    "unknown escape sequence".into(),
                                ))
                            }
                        }
                    }
                }
            } else {
                data.push(ch);
            }
        }
        Ok(self.make_node(node.span, data))
    }

    fn parse_string_expr(&mut self) -> Parse<ast::Expr> {
        let string = self.parse_string()?;
        Ok(self.make_node(string.span, ast::Expr::String(string.data)))
    }

    fn parse_sequence_expr(&mut self) -> Parse<ast::Expr> {
        let sequence = self.parse_sequence_group()?;
        Ok(sequence.nest(ast::Expr::Sequence))
    }

    fn parse_sequence_group(&mut self) -> Parse<ast::Sequence> {
        let llbracket = self.parse_expect_token(Token::LLBracket)?;
        let mut symbols = Vec::new();
        loop {
            let (token, span) = self.peek();
            match token {
                Some(Token::Note) => {
                    self.consume();
                    let note_str = &self.source[span.clone()];
                    if let Some(note) = seq_sym_from_str(note_str) {
                        symbols.push(self.make_node(span, note));
                    } else {
                        self.errors
                            .push(self.make_error(span, format!("Invalid note: {}", note_str)));
                    }
                }
                Some(Token::RRBracket) => {
                    break;
                }
                Some(_) => match self.parse_expr() {
                    Ok(expr) => {
                        symbols.push(expr.nest(ast::SeqSym::Expr));
                    }
                    Err(err) => self.errors.push(err),
                },
                None => {
                    break;
                }
            }
        }

        let rrbracket_parse = self.parse_expect_token(Token::RRBracket);
        let rrbracket = self.recover_replace(rrbracket_parse, ());

        Ok(self.make_node(
            llbracket.span.start..rrbracket.span.end,
            ast::Sequence {
                llbracket,
                symbols,
                rrbracket,
            },
        ))
    }
}

/// Parse a symbol in sequence notation.
fn seq_sym_from_str(input: &str) -> Option<ast::SeqSym> {
    let mut chars = input.chars().peekable();

    if matches!(chars.peek(), Some('r') | Some('R')) {
        chars.next();
        let duration = parse_duration(&mut chars)?;
        Some(ast::SeqSym::Rest { duration })
    } else {
        // If it's not a rest, it's a note.
        let note = parse_note(&mut chars)?;
        let duration = parse_duration(&mut chars)?;

        Some(ast::SeqSym::Note { note, duration })
    }
}

fn parse_note<I: Iterator<Item = char>>(chars: &mut Peekable<I>) -> Option<Note> {
    // First comes the name
    let name = match chars.next()? {
        'a' | 'A' => NoteName::A,
        'b' | 'B' => NoteName::B,
        'c' | 'C' => NoteName::C,
        'd' | 'D' => NoteName::D,
        'e' | 'E' => NoteName::E,
        'f' | 'F' => NoteName::F,
        'g' | 'G' => NoteName::G,
        _ => return None,
    };
    // Then any accidental
    let accidental = match chars.peek().copied() {
        Some(ch) => {
            if ch == '♯' || ch == '#' {
                chars.next();
                Accidental::Sharp
            } else if ch == '♭' || ch == 'b' {
                chars.next();
                Accidental::Flat
            } else {
                Accidental::Base
            }
        }
        _ => Accidental::Base,
    };
    // Then the octave
    let octave = match chars.peek().copied() {
        Some(ch) if ch.is_ascii_digit() => {
            chars.next();
            ch.to_digit(10).unwrap() as i32
        }
        _ => return None,
    };

    Note::try_named(name, accidental, octave)
}

/// Parse the duration part of a note symbol.
/// Individual note lengths start as quarters, and can be doubled or halved with `+` or `-`
/// respectively, optionally followed by one or more dots `.` for dotted lengths.
/// Tied repetitions of the same note or rest are separated by `_`.
/// For example, the duration `+_` is `1/2 + 1/4 = 3/4`, and `-_-.` is `1/8 + 1/8 + 1/16 = 5/16`.
fn parse_duration<I: Iterator<Item = char>>(chars: &mut Peekable<I>) -> Option<Rational> {
    let mut full_duration = Rational::zero();
    loop {
        // Then comes the duration,
        // first in powers of two
        let mut power: i64 = -2; // quarters, 2^(-2) == 1 / 2^2 == 1 / 4
        loop {
            match chars.peek() {
                Some('+') => {
                    chars.next();
                    power += 1;
                }
                Some('-') => {
                    chars.next();
                    power -= 1;
                }
                _ => break,
            }
        }
        // then the dots
        let mut dots = 0;
        while let Some('.') = chars.peek() {
            chars.next();
            dots += 1;
        }
        // Then put everything together
        let mut duration = Rational::int(2).checked_powi(power)?;
        for i in 0..dots {
            // each dot is worth half of the previous note duration
            duration = duration.checked_add(Rational::int(2).checked_powi(power - i - 1)?)?;
        }

        full_duration = full_duration.checked_add(duration)?;

        // Check for tie to next note of the same pitch
        if let Some('_') = chars.peek() {
            chars.next();
        } else {
            break;
        }
    }
    Some(full_duration)
}
