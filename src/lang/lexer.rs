// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! Implements the lexer for our s-expression based language.

use std::fmt;

use super::span::Span;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LexerError {
    location: Span,
    kind: LexerErrorKind,
}

impl LexerError {
    pub fn location(&self) -> Span {
        self.location
    }

    pub fn kind(&self) -> LexerErrorKind {
        self.kind
    }
}

/// The types of lexer errors.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LexerErrorKind {
    UnrecognizedChar,
    UnterminatedString,
    MalformedNumber,
    MalformedIdentifier,
    IllegalOperator,
}

impl fmt::Display for LexerErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexerErrorKind::UnrecognizedChar => write!(f, "unrecognized character"),
            LexerErrorKind::UnterminatedString => write!(f, "unterminated string literal"),
            LexerErrorKind::IllegalOperator => write!(f, "illegal multi-character operator"),
            LexerErrorKind::MalformedNumber => write!(f, "malformed number"),
            LexerErrorKind::MalformedIdentifier => write!(f, "malformed identifier"),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Token {
    // Atoms
    String,
    Int,
    Float,
    Rational,
    /// Identifiers can denote variables or keywords
    Ident,

    // Symbols
    ParenOpen,
    ParenClose,
}

pub struct Lexer<'a> {
    input: &'a str,
    stream: std::str::CharIndices<'a>,
    /// Byte-offset where the current token started
    token_start: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let stream = input.char_indices();
        Self {
            input,
            stream,
            token_start: 0,
        }
    }

    /// Indentify the next token
    pub fn next_token(&mut self) -> Option<Result<(Span, Token), LexerError>> {
        while let Some((pos, ch)) = self.next_char_offset() {
            self.token_start = pos;
            let token_or_error = match ch {
                // Single-character tokens
                '(' => Ok(self.pack_token(Token::ParenOpen)),
                ')' => Ok(self.pack_token(Token::ParenClose)),

                // Multi-character tokens

                // Identifiers
                _ if charsets::is_ident_start(ch) => self.lex_ident(),

                // Strings
                '"' => self.lex_string(),

                // Numbers
                _ if ch.is_ascii_digit() => self.lex_number(),
                // HACK: If a sign is followed by a digit, lex as number
                '+' | '-'
                    if self
                        .peek_char()
                        .map(|ch| ch.is_ascii_digit())
                        .unwrap_or(false) =>
                {
                    self.lex_number()
                }
                // Otherwise, lex as single-letter identifier
                '+' | '-' => self
                    .lookahead_atom_separator(LexerErrorKind::IllegalOperator)
                    .and_then(|_| Ok(self.pack_token(Token::Ident))),

                // Line comments
                ';' => {
                    self.skip_to_next_line();
                    continue;
                }

                // Ignore whitespace between tokens
                _ if ch.is_whitespace() => continue,

                // Produce error token for unrecognized characters
                _ => Err(self.pack_error(LexerErrorKind::UnrecognizedChar)),
            };
            return Some(token_or_error);
        }
        // If we exhausted the stream without producing a token, we're done
        None
    }

    /// Return the next character that would be read and its byte offset.
    fn peek_char_offset(&self) -> Option<(usize, char)> {
        self.stream.clone().next()
    }

    /// Return the byte-offset of the next character that would be read,
    /// or the byte-offset one past the last character if the end of input was reached.
    fn current_offset(&self) -> usize {
        self.peek_char_offset()
            .map_or(self.input.len(), |(pos, _)| pos)
    }

    /// Return the next character that would be read.
    fn peek_char(&self) -> Option<char> {
        self.peek_char_offset().map(|(_, ch)| ch)
    }

    /// Read the next character and also return its byte-offset. Advances the read cursor.
    fn next_char_offset(&mut self) -> Option<(usize, char)> {
        self.stream.next()
    }

    /// Read the next character, advances the read cursor.
    fn next_char(&mut self) -> Option<char> {
        self.next_char_offset().map(|(_, ch)| ch)
    }

    /// Skip all characters up to and including the next `\n`, or to the end of the input.
    fn skip_to_next_line(&mut self) {
        while let Some(ch) = self.next_char() {
            if ch == '\n' {
                break;
            }
        }
    }

    fn skip_while<P: Fn(char) -> bool>(&mut self, predicate: P) {
        while let Some(ch) = self.peek_char() {
            if predicate(ch) {
                self.next_char_offset();
            } else {
                break;
            }
        }
    }

    /// Read the next char, and fail with the given error if the end of the input was reached,
    /// or the char does not fulfil the predicate.
    fn expect_char<P: Fn(char) -> bool>(
        &mut self,
        error: LexerErrorKind,
        predicate: P,
    ) -> Result<(), LexerError> {
        if self.next_char().map_or(true, |ch| !predicate(ch)) {
            Err(self.pack_error(error))
        } else {
            Ok(())
        }
    }

    /// If there is a next char, it must be one that can separate atoms.
    /// If not, it consumes all non-separating characters
    fn lookahead_atom_separator(&mut self, error: LexerErrorKind) -> Result<(), LexerError> {
        if self
            .peek_char()
            .map_or(false, |ch| !charsets::is_atom_separator(ch))
        {
            self.skip_while(|ch| !charsets::is_atom_separator(ch));
            Err(self.pack_error(error))
        } else {
            Ok(())
        }
    }

    fn lex_string(&mut self) -> Result<(Span, Token), LexerError> {
        let mut escaped = false;
        let mut terminated = false;
        while let Some(ch) = self.peek_char() {
            if ch == '\n' {
                // Multi-line strings are disallowed
                break;
            }
            // Consume the character otherwise
            self.next_char();

            if escaped {
                escaped = false;
            } else if ch == '"' {
                terminated = true;
                break;
            } else if ch == '\\' {
                escaped = true;
            }
        }

        if terminated {
            Ok(self.pack_token(Token::String))
        } else {
            Err(self.pack_error(LexerErrorKind::UnterminatedString))
        }
    }

    fn lex_ident(&mut self) -> Result<(Span, Token), LexerError> {
        self.skip_while(charsets::is_ident_cont);
        self.lookahead_atom_separator(LexerErrorKind::MalformedIdentifier)?;
        Ok(self.pack_token(Token::Ident))
    }

    fn lex_number(&mut self) -> Result<(Span, Token), LexerError> {
        self.skip_while(|c| c.is_ascii_digit());
        match self.peek_char() {
            Some('/') => {
                self.next_char();
                self.lex_rational_denominator()
            }
            Some('.') => {
                self.next_char();
                self.lex_float_fractional()
            }
            _ => {
                self.lookahead_atom_separator(LexerErrorKind::MalformedNumber)?;
                Ok(self.pack_token(Token::Int))
            }
        }
    }

    /// Lex the denominator of a rational, must consist of at least one ascii digit.
    fn lex_rational_denominator(&mut self) -> Result<(Span, Token), LexerError> {
        self.expect_char(LexerErrorKind::MalformedNumber, |ch| ch.is_ascii_digit())?;
        self.skip_while(|c| c.is_ascii_digit());
        self.lookahead_atom_separator(LexerErrorKind::MalformedNumber)?;
        Ok(self.pack_token(Token::Rational))
    }

    /// Lex the part of a float after the dot, which may be empty, i.e. `1.` is a valid float literal.
    /// TODO: add support for scientific notation.
    fn lex_float_fractional(&mut self) -> Result<(Span, Token), LexerError> {
        self.skip_while(|c| c.is_ascii_digit());
        self.lookahead_atom_separator(LexerErrorKind::MalformedNumber)?;
        Ok(self.pack_token(Token::Float))
    }

    fn pack_token(&self, token: Token) -> (Span, Token) {
        (
            Span {
                begin: self.token_start,
                end: self.current_offset(),
            },
            token,
        )
    }

    fn pack_error(&self, kind: LexerErrorKind) -> LexerError {
        LexerError {
            location: Span {
                begin: self.token_start,
                end: self.current_offset(),
            },
            kind,
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<(Span, Token), LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

/// Defines the charsets of various things that can be lexed
mod charsets {
    /// Characters that are allowed as the first character of an identifier.
    pub fn is_ident_start(ch: char) -> bool {
        let extra = "!$%&*/:<=>?~_^";
        ch.is_alphabetic() || extra.chars().any(|c| c == ch)
    }

    /// Characters that are allowed as the second or later character of an identifier.
    pub fn is_ident_cont(ch: char) -> bool {
        let extra = ".+-";
        is_ident_start(ch) || ch.is_ascii_digit() || extra.chars().any(|c| c == ch)
    }

    /// Two atoms must be separated by one of these characters to be counted as separate.
    pub fn is_atom_separator(ch: char) -> bool {
        ch == '(' || ch == ')' || ch.is_whitespace()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn expect_single_token(input: &str, span: Span, token: Token) {
        let mut l = Lexer::new(input);
        if let Some(token_or_error) = l.next_token() {
            assert_eq!(token_or_error, Ok((span, token)));
            assert_eq!(l.next_token(), None);
        } else {
            panic!("input did not result in a token");
        }
    }

    fn expect_single_failure(input: &str, location: Span, kind: LexerErrorKind) {
        let mut l = Lexer::new(input);
        if let Some(token_or_error) = l.next_token() {
            assert_eq!(token_or_error, Err(LexerError { location, kind }));
            assert_eq!(l.next_token(), None);
        } else {
            panic!("input did not result in a token");
        }
    }

    fn expect_token_sequence(input: &str, tokens: &[Result<Token, LexerErrorKind>]) {
        let mut l = Lexer::new(input);
        let mut actual = Vec::new();
        while let Some(token_or_error) = l.next_token() {
            actual.push(token_or_error.map(|x| x.1).map_err(|e| e.kind))
        }
        assert_eq!(&actual[..], tokens)
    }

    #[test]
    fn test_string() {
        expect_single_token(r#""hello""#, Span { begin: 0, end: 7 }, Token::String);
        expect_single_token(
            r#""hello\"world""#,
            Span { begin: 0, end: 14 },
            Token::String,
        );
        expect_single_token(
            r#"  "hello\"world"  "#,
            Span { begin: 2, end: 16 },
            Token::String,
        );

        expect_single_failure(
            r#""hello\"world  "#,
            Span { begin: 0, end: 15 },
            LexerErrorKind::UnterminatedString,
        );
    }

    #[test]
    fn test_int() {
        expect_single_token("123", Span { begin: 0, end: 3 }, Token::Int);
        expect_single_token("+123", Span { begin: 0, end: 4 }, Token::Int);
        expect_single_token("-123", Span { begin: 0, end: 4 }, Token::Int);

        expect_single_failure(
            "123x ",
            Span { begin: 0, end: 4 },
            LexerErrorKind::MalformedNumber,
        );

        expect_token_sequence(
            "123)(",
            &[Ok(Token::Int), Ok(Token::ParenClose), Ok(Token::ParenOpen)],
        )
    }

    #[test]
    fn test_float() {
        expect_single_token("123.", Span { begin: 0, end: 4 }, Token::Float);
        expect_single_token("+123.412", Span { begin: 0, end: 8 }, Token::Float);
        expect_single_token("-123.4 ", Span { begin: 0, end: 6 }, Token::Float);

        expect_single_failure(
            "123.1xy",
            Span { begin: 0, end: 7 },
            LexerErrorKind::MalformedNumber,
        );
    }

    #[test]
    fn test_rational() {
        expect_single_token("123/31", Span { begin: 0, end: 6 }, Token::Rational);
        expect_single_token("+1/3 ", Span { begin: 0, end: 4 }, Token::Rational);
        expect_single_token("-5/2", Span { begin: 0, end: 4 }, Token::Rational);

        expect_single_failure(
            "-11/",
            Span { begin: 0, end: 4 },
            LexerErrorKind::MalformedNumber,
        );
        expect_single_failure(
            "-11/24-42/4",
            Span { begin: 0, end: 11 },
            LexerErrorKind::MalformedNumber,
        );
    }

    #[test]
    fn test_ident() {
        expect_single_token("-", Span { begin: 0, end: 1 }, Token::Ident);
        expect_single_token("+", Span { begin: 0, end: 1 }, Token::Ident);
        expect_single_token(
            " prim/float-int! ",
            Span { begin: 1, end: 16 },
            Token::Ident,
        );
        expect_single_token("foo-αβγ!", Span { begin: 0, end: 11 }, Token::Ident);
        expect_single_token("bar", Span { begin: 0, end: 3 }, Token::Ident);

        expect_single_failure(
            "-/",
            Span { begin: 0, end: 2 },
            LexerErrorKind::IllegalOperator,
        );

        expect_token_sequence(
            "(foo-αβγ!)",
            &[
                Ok(Token::ParenOpen),
                Ok(Token::Ident),
                Ok(Token::ParenClose),
            ],
        )
    }
}
