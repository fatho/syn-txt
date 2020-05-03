use std::fmt;

use super::ast::*;
use super::lexer::Token;
use super::span::Span;
use crate::rational::{ParseRationalError, Rational};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    location: Span,
    info: ParseErrorInfo,
}

impl ParseError {
    pub fn new(location: Span, info: ParseErrorInfo) -> Self {
        Self { location, info }
    }

    pub fn location(&self) -> Span {
        self.location
    }

    pub fn info(&self) -> &ParseErrorInfo {
        &self.info
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorInfo {
    InvalidInt(std::num::ParseIntError),
    InvalidFloat(std::num::ParseFloatError),
    InvalidRational(ParseRationalError),
    /// This would indicate a lexer bug, as it should sort out those cases
    InvalidString,
    /// Invalid escape squence in a string
    InvalidEscape,
    Unexpected {
        /// One of these tokens was excpected
        expected: Vec<Token>,
        /// But this was the actual next token
        actual: Token,
    },
    /// The end of the input was reached, but the parser was expecting more.
    EOF,
}

impl fmt::Display for ParseErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseErrorInfo::InvalidInt(err) => write!(f, "invalid integer literal: {}", err),
            ParseErrorInfo::InvalidFloat(err) => write!(f, "invalid float literal: {}", err),
            ParseErrorInfo::InvalidRational(err) => write!(f, "invalid rational literal: {}", err),
            ParseErrorInfo::InvalidString => write!(f, "invalid string literal"),
            ParseErrorInfo::InvalidEscape => write!(f, "invalid escape sequence"),
            ParseErrorInfo::Unexpected { expected, actual } => write!(
                f,
                "expected one of {:?}, but got {:?}",
                &expected[..],
                actual
            ),
            ParseErrorInfo::EOF => write!(f, "end of file reached"),
        }
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

pub struct Parser<'a> {
    tokens: &'a [(Span, Token)],
    source: &'a str,
    current_token: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, tokens: &'a [(Span, Token)]) -> Self {
        Self {
            tokens,
            source,
            current_token: 0,
        }
    }

    /// Parse a document consisting of zero or more symbolic expressions.
    pub fn parse(&mut self) -> ParseResult<Vec<SymExpSrc>> {
        let mut out = Vec::new();
        while let Some(sym) = self.parse_next()? {
            out.push(sym)
        }
        Ok(out)
    }

    /// Parse only the next self-contained expression from the input.
    pub fn parse_next(&mut self) -> ParseResult<Option<SymExpSrc>> {
        match self.parse_exp() {
            Ok(symexpr) => Ok(Some(symexpr)),
            Err(err) => {
                if let ParseErrorInfo::EOF = err.info {
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
            _ => Err(ParseError::new(
                span,
                ParseErrorInfo::Unexpected {
                    expected: vec![
                        Token::ParenOpen,
                        Token::Int,
                        Token::Float,
                        Token::Rational,
                        Token::String,
                        Token::Ident,
                    ],
                    actual: token,
                },
            )),
        }
    }

    // Parsers for turning single tokens into values

    fn parse_int(&self, span: Span) -> ParseResult<i64> {
        let s = self.get_span(span);
        s.parse()
            .map_err(|error| ParseError::new(span, ParseErrorInfo::InvalidInt(error)))
    }

    fn parse_float(&self, span: Span) -> ParseResult<f64> {
        let s = self.get_span(span);
        s.parse()
            .map_err(|error| ParseError::new(span, ParseErrorInfo::InvalidFloat(error)))
    }

    fn parse_rational(&self, span: Span) -> ParseResult<Rational> {
        let s = self.get_span(span);
        s.parse()
            .map_err(|error| ParseError::new(span, ParseErrorInfo::InvalidRational(error)))
    }

    fn parse_string(&self, span: Span) -> ParseResult<String> {
        let s = self.get_span(span);
        let mut chars = s.char_indices();

        // Ensure it starts with a quote
        if chars.next().map(|(_, ch)| ch) != Some('"') {
            return Err(ParseError::new(span, ParseErrorInfo::InvalidString));
        }

        // Process escape sequences
        let mut out = String::new();
        let mut escaped = false;
        let mut terminated = false;
        let mut escape_start: usize = 0;

        while let Some((pos, ch)) = chars.next() {
            if escaped {
                escaped = false;
                match ch {
                    // TODO: add support for arbitrary unicode characters
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    '0' => out.push('\0'),
                    '"' => out.push('\"'),
                    '\\' => out.push('\\'),
                    // Disallow everything else to ensure backwards compatibility when adding new escape sequences
                    _ => {
                        let location = Span {
                            begin: escape_start,
                            end: pos + ch.len_utf8(),
                        };
                        return Err(ParseError::new(location, ParseErrorInfo::InvalidEscape));
                    }
                }
            } else if ch == '\\' {
                escape_start = pos;
                escaped = true;
            } else if ch == '"' {
                terminated = true;
                break;
            } else {
                out.push(ch);
            }
        }

        // Must end with quote
        if !terminated {
            return Err(ParseError::new(span, ParseErrorInfo::InvalidString));
        }

        Ok(out)
    }

    // Manipulating/Inspecting the token stream

    fn pop_token(&mut self) -> Option<(Span, Token)> {
        if self.current_token < self.tokens.len() {
            let tok = self.tokens[self.current_token];
            self.current_token += 1;
            Some(tok)
        } else {
            None
        }
    }

    fn peek_token(&self) -> Option<(Span, Token)> {
        if self.current_token < self.tokens.len() {
            Some(self.tokens[self.current_token])
        } else {
            None
        }
    }

    fn get_span(&self, span: Span) -> &'a str {
        &self.source[span.begin..span.end]
    }

    fn parse_token(&mut self) -> ParseResult<(Span, Token)> {
        self.pop_token().ok_or(ParseError::new(
            Span {
                begin: self.source.len(),
                end: self.source.len(),
            },
            ParseErrorInfo::EOF,
        ))
    }

    fn expect_token(&mut self, expected: Token) -> ParseResult<Span> {
        if let Some((span, token)) = self.pop_token() {
            if token == expected {
                Ok(span)
            } else {
                Err(ParseError::new(
                    span,
                    ParseErrorInfo::Unexpected {
                        expected: vec![expected],
                        actual: token,
                    },
                ))
            }
        } else {
            Err(self.eof_error())
        }
    }

    fn eof_error(&self) -> ParseError {
        ParseError::new(
            Span {
                begin: self.source.len(),
                end: self.source.len(),
            },
            ParseErrorInfo::EOF,
        )
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

#[cfg(test)]
mod test {
    use super::super::lexer::*;
    use super::*;

    // TODO: test some error cases as well

    fn expect_single_expression(input: &str, expected: SymExp) {
        let tokens = Lexer::new(input)
            .collect::<Result<Vec<(Span, Token)>, _>>()
            .unwrap();
        let mut parser = Parser::new(input, &tokens);
        let src = Span {
            begin: 0,
            end: input.len(),
        };
        assert_eq!(
            parser.parse_next(),
            Ok(Some(SymExpSrc { src, exp: expected }))
        );
        assert_eq!(parser.parse_next(), Ok(None));
    }

    #[test]
    fn test_int() {
        expect_single_expression("123", SymExp::Int(123));
        expect_single_expression("-123", SymExp::Int(-123));
        expect_single_expression("+123", SymExp::Int(123));
    }

    #[test]
    fn test_ratio() {
        expect_single_expression("12/4", SymExp::Ratio(Rational::new(12, 4)));
        expect_single_expression("-3/7", SymExp::Ratio(Rational::new(-3, 7)));
    }

    #[test]
    fn test_float() {
        expect_single_expression("12.125", SymExp::Float(12.125));
        expect_single_expression("-3.", SymExp::Float(-3.0));
    }

    #[test]
    fn test_string() {
        expect_single_expression(r#""hello world""#, SymExp::Str("hello world".to_owned()));
        expect_single_expression(r#""""#, SymExp::Str("".to_owned()));
        expect_single_expression(
            r#""hello \"world\"""#,
            SymExp::Str("hello \"world\"".to_owned()),
        );
        expect_single_expression(
            r#""C:\\Users\\Foo""#,
            SymExp::Str("C:\\Users\\Foo".to_owned()),
        );
        expect_single_expression(r#""Line1\nLine2""#, SymExp::Str("Line1\nLine2".to_owned()));
    }

    #[test]
    fn test_ident() {
        expect_single_expression(
            "a-variable",
            SymExp::Variable(Ident("a-variable".to_owned())),
        );
        expect_single_expression(
            ":a-keyword",
            SymExp::Keyword(Ident(":a-keyword".to_owned())),
        );
    }

    #[test]
    fn test_list() {
        expect_single_expression(
            r#"(a-list "that \"really\"" :contains 12. -4 3/2 (everything at-once))"#,
            SymExp::List(vec![
                SymExpSrc {
                    src: Span { begin: 1, end: 7 },
                    exp: SymExp::Variable(Ident("a-list".to_owned())),
                },
                SymExpSrc {
                    src: Span { begin: 8, end: 25 },
                    exp: SymExp::Str("that \"really\"".to_owned()),
                },
                SymExpSrc {
                    src: Span { begin: 26, end: 35 },
                    exp: SymExp::Keyword(Ident(":contains".to_owned())),
                },
                SymExpSrc {
                    src: Span { begin: 36, end: 39 },
                    exp: SymExp::Float(12.0),
                },
                SymExpSrc {
                    src: Span { begin: 40, end: 42 },
                    exp: SymExp::Int(-4),
                },
                SymExpSrc {
                    src: Span { begin: 43, end: 46 },
                    exp: SymExp::Ratio(Rational::new(3, 2)),
                },
                SymExpSrc {
                    src: Span { begin: 47, end: 67 },
                    exp: SymExp::List(vec![
                        SymExpSrc {
                            src: Span { begin: 48, end: 58 },
                            exp: SymExp::Variable(Ident("everything".to_owned())),
                        },
                        SymExpSrc {
                            src: Span { begin: 59, end: 66 },
                            exp: SymExp::Variable(Ident("at-once".to_owned())),
                        },
                    ]),
                },
            ]),
        );

        expect_single_expression(
            "(define\n  x\n  \"on a new line\"\n)",
            SymExp::List(vec![
                SymExpSrc {
                    src: Span { begin: 1, end: 7 },
                    exp: SymExp::Variable(Ident("define".to_owned())),
                },
                SymExpSrc {
                    src: Span { begin: 10, end: 11 },
                    exp: SymExp::Variable(Ident("x".to_owned())),
                },
                SymExpSrc {
                    src: Span { begin: 14, end: 29 },
                    exp: SymExp::Str("on a new line".to_owned()),
                },
            ]),
        )
    }
}
