//! Implements the lexer for our s-expression based language.

use super::span::Span;

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

    // Tokens representing invalid input
    ErrUnrecognized,
    ErrUnterminatedString,
}

impl Token {
    pub fn is_error(self) -> bool {
        match self {
            Token::ErrUnrecognized => true,
            Token::ErrUnterminatedString => true,
            _ => false,
        }
    }
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

    pub fn input(&self) -> &'a str {
        self.input
    }

    pub fn eof(&self) -> bool {
        self.peek_char().is_none()
    }

    /// Return the byte-offset of the next character that would be read.
    pub fn current_offset(&self) -> usize {
        self.peek_char().map_or(self.input.len(), |(pos, _)| pos)
    }

    fn peek_char(&self) -> Option<(usize, char)> {
        self.stream.clone().next()
    }

    fn peek_char_skip(&self, skip: usize) -> Option<(usize, char)> {
        self.stream.clone().skip(skip).next()
    }

    fn next_char(&mut self) -> Option<(usize, char)> {
        self.stream.next()
    }

    fn skip_to_next_line(&mut self) {
        while let Some((_, ch)) = self.next_char() {
            if ch == '\n' {
                break;
            }
        }
    }

    fn skip_while<P: Fn(char) -> bool>(&mut self, predicate: P) {
        while let Some((_, ch)) = self.peek_char() {
            if predicate(ch) {
                self.next_char();
            } else {
                break;
            }
        }
    }

    /// Indentify the next token
    pub fn next_token(&mut self) -> Option<(Span, Token)> {
        while let Some((pos, ch)) = self.next_char() {
            self.token_start = pos;
            match ch {
                // Single-character tokens
                '(' => return Some(self.pack_token(Token::ParenOpen)),
                ')' => return Some(self.pack_token(Token::ParenClose)),

                // Multi-character tokens

                // Identifiers
                _ if charsets::is_ident_start(ch) => return Some(self.lex_ident()),

                // Strings
                '"' => return Some(self.lex_string()),

                // Numbers
                _ if ch.is_ascii_digit() || ch == '+' || ch == '-' => {
                    return Some(self.lex_number())
                }

                // Line comments
                ';' => self.skip_to_next_line(),

                // Ignore whitespace between tokens
                _ if ch.is_whitespace() => continue,

                // Produce error token for unrecognized characters
                _ => return Some(self.pack_token(Token::ErrUnrecognized)),
            }
        }
        // If we exhausted the stream without producing a token, we're done
        None
    }

    fn lex_string(&mut self) -> (Span, Token) {
        let mut escaped = false;
        let mut terminated = false;
        while let Some((_, ch)) = self.peek_char() {
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

        let tok = if terminated {
            Token::String
        } else {
            Token::ErrUnterminatedString
        };

        self.pack_token(tok)
    }

    fn lex_ident(&mut self) -> (Span, Token) {
        self.skip_while(charsets::is_ident_cont);
        self.pack_token(Token::Ident)
    }

    fn lex_number(&mut self) -> (Span, Token) {
        while let Some((_, ch)) = self.peek_char() {
            if ch.is_ascii_digit() {
                self.next_char();
            } else if ch == '/' {
                // Only consume the '/' if followed by a digit again
                if let Some((_, after)) = self.peek_char_skip(1) {
                    if after.is_ascii_digit() {
                        self.next_char();
                        return self.lex_rational_denominator();
                    }
                }
                break;
            } else if ch == '.' {
                self.next_char();
                return self.lex_float_fractional();
            } else {
                break;
            }
        }
        // If we made it out, it's an int
        self.pack_token(Token::Int)
    }

    fn lex_rational_denominator(&mut self) -> (Span, Token) {
        self.skip_while(|c| c.is_ascii_digit());
        self.pack_token(Token::Rational)
    }

    /// Lex the part of a float after the dot.
    /// `start` refers to the start of the number itself,
    /// not to the start of the fractional part.
    /// TODO: add support for scientific notation.
    fn lex_float_fractional(&mut self) -> (Span, Token) {
        self.skip_while(|c| c.is_ascii_digit());
        self.pack_token(Token::Float)
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
}

/// Defines the charsets of various things that can be lexed
mod charsets {

    pub fn is_ident_start(ch: char) -> bool {
        let extra = "!$%&*/:<=>?~_^";
        ch.is_alphabetic() || extra.chars().any(|c| c == ch)
    }

    pub fn is_ident_cont(ch: char) -> bool {
        let extra = ".+-";
        is_ident_start(ch) || ch.is_ascii_digit() || extra.chars().any(|c| c == ch)
    }
}
