use super::ast::*;
use super::lexer::Token;
use super::span::Span;
use crate::rational::{ParseRationalError, Rational};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    InvalidInt {
        error: std::num::ParseIntError,
        span: Span,
    },
    InvalidFloat {
        error: std::num::ParseFloatError,
        span: Span,
    },
    InvalidRational {
        error: ParseRationalError,
        span: Span,
    },
    InvalidString {
        span: Span,
        offset: usize,
    },
    Unexpected {
        expected: Vec<Token>,
        actual: Token,
        span: Span,
    },
    /// The end of the input was reached, but the parser was expecting more.
    EOF,
}

type ParseResult<T> = Result<T, ParseError>;

pub struct Parser<'a> {
    tokens: &'a [(Span, Token)],
    source: &'a str,
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, tokens: &'a [(Span, Token)]) -> Self {
        Self {
            tokens,
            source,
            current: 0,
        }
    }

    /// Parse a document consisting of zero or more symbolic expressions.
    pub fn parse(&mut self) -> ParseResult<Vec<SymExpSrc>> {
        let mut out = Vec::new();
        while let Some(sym) = self.next_symexp()? {
            out.push(sym)
        }
        Ok(out)
    }

    pub fn next_symexp(&mut self) -> ParseResult<Option<SymExpSrc>> {
        match self.parse_exp() {
            Ok(symexpr) => Ok(Some(symexpr)),
            Err(err) => {
                if let ParseError::EOF = err {
                    Ok(None)
                } else {
                    Err(err)
                }
            }
        }
    }

    // Parser for expressions

    fn parse_exp(&mut self) -> ParseResult<SymExpSrc> {
        let (span, token) = self.parse_token()?;
        match token {
            Token::ParenOpen => {
                let mut list = Vec::new();
                while !self.is_terminated(Token::ParenClose) {
                    list.push(self.parse_exp()?);
                }
                let end = self.expect_token(Token::ParenClose)?;
                let list_span = Span {
                    begin: span.begin,
                    end: end.end,
                };
                Ok(SymExpSrc {
                    src: list_span,
                    exp: SymExp::List(list),
                })
            }
            Token::Int => {
                let i = self.parse_int(span)?;
                Ok(SymExpSrc {
                    src: span,
                    exp: SymExp::Int(i),
                })
            }
            Token::Rational => {
                let r = self.parse_rational(span)?;
                Ok(SymExpSrc {
                    src: span,
                    exp: SymExp::Ratio(r),
                })
            }
            Token::Float => {
                let f = self.parse_float(span)?;
                Ok(SymExpSrc {
                    src: span,
                    exp: SymExp::Float(f),
                })
            }
            Token::String => {
                let s = self.parse_string(span)?;
                Ok(SymExpSrc {
                    src: span,
                    exp: SymExp::Str(s),
                })
            }
            Token::Ident => {
                let ident = self.get_span(span);
                if ident.starts_with(':') {
                    Ok(SymExpSrc {
                        src: span,
                        exp: SymExp::Keyword(Ident(ident.to_owned())),
                    })
                } else {
                    Ok(SymExpSrc {
                        src: span,
                        exp: SymExp::Variable(Ident(ident.to_owned())),
                    })
                }
            }
            _ => Err(ParseError::Unexpected {
                expected: vec![
                    Token::ParenOpen,
                    Token::Int,
                    Token::Float,
                    Token::Rational,
                    Token::String,
                    Token::Ident,
                ],
                actual: token,
                span,
            }),
        }
    }

    // Parsers for turning single tokens into values

    fn parse_int(&self, span: Span) -> ParseResult<i64> {
        let s = self.get_span(span);
        s.parse()
            .map_err(|error| ParseError::InvalidInt { error, span })
    }

    fn parse_float(&self, span: Span) -> ParseResult<f64> {
        let s = self.get_span(span);
        s.parse()
            .map_err(|error| ParseError::InvalidFloat { error, span })
    }

    fn parse_rational(&self, span: Span) -> ParseResult<Rational> {
        let s = self.get_span(span);
        s.parse()
            .map_err(|error| ParseError::InvalidRational { error, span })
    }

    fn parse_string(&self, span: Span) -> ParseResult<String> {
        let s = self.get_span(span);
        let mut chars = s.char_indices();

        // Ensure it starts with a quote
        if chars.next().map(|(_, ch)| ch) != Some('"') {
            return Err(ParseError::InvalidString { span, offset: 0 });
        }

        // Process escape sequences
        let mut out = String::new();
        let mut escaped = false;
        let mut terminated = false;

        while let Some((_pos, ch)) = chars.next() {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
                continue;
            } else if ch == '"' {
                terminated = true;
                break;
            }
            out.push(ch);
        }

        // Must end with quote
        if !terminated {
            return Err(ParseError::InvalidString {
                span,
                offset: s.len() - 1,
            });
        }

        Ok(out)
    }

    // Manipulating/Inspecting the token stream

    fn pop_token(&mut self) -> Option<(Span, Token)> {
        if self.current < self.tokens.len() {
            let tok = self.tokens[self.current];
            self.current += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn peek_token(&self) -> Option<(Span, Token)> {
        if self.current < self.tokens.len() {
            Some(self.tokens[self.current])
        } else {
            None
        }
    }

    fn get_span(&self, span: Span) -> &'a str {
        &self.source[span.begin..span.end]
    }

    fn parse_token(&mut self) -> ParseResult<(Span, Token)> {
        self.pop_token().ok_or(ParseError::EOF)
    }

    fn expect_token(&mut self, expected: Token) -> ParseResult<Span> {
        if let Some((span, token)) = self.pop_token() {
            if token == expected {
                Ok(span)
            } else {
                Err(ParseError::Unexpected {
                    expected: vec![expected],
                    actual: token,
                    span,
                })
            }
        } else {
            Err(ParseError::EOF)
        }
    }

    /// Check if the next token is the expected terminator or EOF.
    fn is_terminated(&self, terminator: Token) -> bool {
        if let Some((_, token)) = self.peek_token() {
            token == terminator
        } else {
            true
        }
    }
}
