use std::{fmt::format, iter::Peekable};

use logos::{internal::CallbackResult, Logos, SpannedIter};

use crate::lexer::{Span, Token};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Node<T> {
    pub span: Span,
    pub data: T,
}

pub mod ast {
    use super::Node;

    #[derive(Debug, Eq, PartialEq, Clone)]
    pub struct Root {
        pub object: Node<Object>,
    }

    #[derive(Debug, Eq, PartialEq, Clone)]
    pub struct Object {
        pub name: Node<String>,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    span: Span,
    message: String,
}

type Parse<T> = Result<Node<T>, ParseError>;

pub struct Parser<'a> {
    source: &'a str,
    stream: Peekable<logos::SpannedIter<'a, Token>>,
    consumed: usize,
    errors: Vec<ParseError>,
}

/// Newtype wrapper for a position in the source text.
struct SourcePos(usize);

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Parser {
            source,
            stream: Token::lexer(source).spanned().peekable(),
            consumed: 0,
            errors: Vec::new(),
        }
    }

    fn peek(&mut self) -> Option<Token> {
        self.stream.peek().map(|(tok, _)| tok).copied()
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
                return Ok(Node { span, data: () });
            } else {
                return Err(ParseError {
                    span,
                    message: format!("Expected {:?} but got {:?}", expected, token),
                });
            }
        } else {
            return Err(ParseError {
                span: self.consumed..self.consumed,
                message: format!("Expected {:?} but reached end of file", expected),
            });
        }
    }

    // Parse rules

    fn parse_root(&mut self) -> Parse<ast::Root> {
        let object = self.parse_object()?;
        Ok(Node {
            span: object.span.clone(),
            data: ast::Root { object },
        })
    }

    fn parse_object(&mut self) -> Parse<ast::Object> {
        let name = self.parse_ident()?;
        self.parse_expect_token(Token::LBrace)?;
        let rbrace = self.parse_expect_token(Token::RBrace)?;
        Ok(Node {
            span: name.span.start..rbrace.span.end,
            data: ast::Object { name },
        })
    }

    fn parse_ident(&mut self) -> Parse<String> {
        let node = self.parse_expect_token(Token::Ident)?;
        Ok(Node {
            span: node.span.clone(),
            data: self.source[node.span].into(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::{ast, Node, Parser};
    use expect_test::{expect, Expect};

    fn check(input: &str, output: Expect) {
        let mut parser = Parser::new(input);
        let result = parser.parse_root();
        let debug = format!("{:#?}", result);
        output.assert_eq(&debug);
    }


    #[test]
    fn parse_empty() {
        check("", expect![[r#"
            Err(
                ParseError {
                    span: 0..0,
                    message: "Expected Ident but reached end of file",
                },
            )"#]]);
    }

    #[test]
    fn parse_empty_object() {
        check(r"Song {
            // super awesome song, eventually
        }", expect![[r#"
            Ok(
                Node {
                    span: 0..62,
                    data: Root {
                        object: Node {
                            span: 0..62,
                            data: Object {
                                name: Node {
                                    span: 0..4,
                                    data: "Song",
                                },
                            },
                        },
                    },
                },
            )"#]]);
    }
}
