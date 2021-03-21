use std::{fmt::format, iter::Peekable, ops::Deref, sync::Arc, thread::spawn};

use ast::BinaryOp;
use logos::{internal::CallbackResult, Logos, SpannedIter};

use crate::lexer::{Span, Token};

#[derive(Debug, Clone)]
pub struct Node<T> {
    pub span: Span,
    pub data: T,
}

pub type NodePtr<T> = Arc<Node<T>>;

pub mod ast {
    use super::{Node, NodePtr};

    #[derive(Debug, Clone)]
    pub struct Root {
        pub object: Node<Object>,
    }

    #[derive(Debug, Clone)]
    pub struct Object {
        pub name: Node<String>,
        pub attrs: Vec<Node<Attribute>>,
        pub children: Vec<Node<Object>>,
    }

    #[derive(Debug, Clone)]
    pub struct Attribute {
        pub name: Node<String>,
    }

    #[derive(Debug, Clone)]
    pub enum Expr {
        String(String),
        Int(i64),
        Float(f64),
        Unary {
            operator: Node<UnaryOp>,
            operand: NodePtr<Expr>,
        },
        Binary {
            left: NodePtr<Expr>,
            operator: Node<BinaryOp>,
            right: NodePtr<Expr>,
        },
        Paren {
            lparen: Node<()>,
            expr: NodePtr<Expr>,
            rparen: Node<()>,
        },
    }

    #[derive(Debug, Clone)]
    pub enum UnaryOp {
        Plus,
        Minus,
    }

    #[derive(Debug, Clone)]
    pub enum BinaryOp {
        Add,
        Sub,
        Mult,
        Div,
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    span: Span,
    message: String,
}

impl ParseError {
    pub fn expected_but_got(span: Span, expected: &[Token], got: Token) -> ParseError {
        ParseError {
            span,
            message: format!("Expected one of {:?}, but got {:?}", expected, got),
        }
    }
    pub fn expected_str_but_got(span: Span, expected: &str, got: Token) -> ParseError {
        ParseError {
            span,
            message: format!("Expected one of {}, but got {:?}", expected, got),
        }
    }

    pub fn unexpected_eof(span: Span, expected: &[Token]) -> ParseError {
        ParseError {
            span,
            message: format!("Expected one of {:?}, but reached end of file", expected),
        }
    }

    pub fn unexpected_str_eof(span: Span, expected: &str) -> ParseError {
        ParseError {
            span,
            message: format!("Expected one of {}, but reached end of file", expected),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Prec(u16);

impl Prec {
    const LOWEST: Prec = Prec(0);
    const ADDITIVE: Prec = Prec(1);
    const MULTIPLICATIVE: Prec = Prec(2);
    const UNARY: Prec = Prec(3);
    const HIGHEST: Prec = Prec(4);

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
}

impl<'a> Parser<'a> {
    // Public interface
    pub fn parse(source: &'a str) -> Parse<ast::Root> {
        Parser::new(source).parse_root()
    }

    // Private helpers

    fn new(source: &'a str) -> Self {
        Parser {
            source,
            stream: Token::lexer(source).spanned().peekable(),
            consumed: 0,
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
                return Ok(Node { span, data: () });
            } else {
                return Err(ParseError::expected_but_got(span, &[expected], token));
            }
        } else {
            return Err(ParseError::unexpected_eof(self.eof(), &[expected]));
        }
    }

    fn parse_any_token(&mut self) -> Parse<()> {
        if let Some((token, span)) = self.consume() {
            return Ok(Node { span, data: () });
        } else {
            return Err(ParseError::unexpected_str_eof(self.eof(), "any token"));
        }
    }

    fn eof(&self) -> Span {
        self.source.len()..self.source.len()
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
        self.parse_object_body(name)
    }

    fn parse_object_body(&mut self, name: Node<String>) -> Parse<ast::Object> {
        self.parse_expect_token(Token::LBrace)?;

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
                            // attribute
                            // TODO: parse value
                            todo!("attribute")
                        }
                        Some(Token::LBrace) => {
                            let child_object = self.parse_object_body(inner_name)?;
                            children.push(child_object);
                        }
                        Some(other) => {
                            return Err(ParseError::expected_but_got(span, EXPECTATION, other))
                        }
                        None => {
                            return Err(ParseError::unexpected_eof(span, EXPECTATION));
                        }
                    }
                }
                _ => break,
            }
        }

        let rbrace = self.parse_expect_token(Token::RBrace)?;
        Ok(Node {
            span: name.span.start..rbrace.span.end,
            data: ast::Object {
                name,
                attrs,
                children,
            },
        })
    }

    fn parse_ident(&mut self) -> Parse<String> {
        let node = self.parse_expect_token(Token::Ident)?;
        Ok(Node {
            span: node.span.clone(),
            data: self.source[node.span].into(),
        })
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

            // Supported infix/postfix operators
            left = match token {
                Token::Plus if min_prec <= Prec::ADDITIVE => {
                    self.parse_binary_operand(left, Node { span, data: BinaryOp::Add }, Prec::ADDITIVE.succ())?
                }
                Token::Minus if min_prec <= Prec::ADDITIVE => {
                    self.parse_binary_operand(left, Node { span, data: BinaryOp::Sub }, Prec::ADDITIVE.succ())?
                },
                Token::Star if min_prec <= Prec::MULTIPLICATIVE =>  {
                    self.parse_binary_operand(left, Node { span, data: BinaryOp::Mult }, Prec::MULTIPLICATIVE.succ())?
                },
                Token::Slash if min_prec <= Prec::MULTIPLICATIVE =>  {
                    self.parse_binary_operand(left, Node { span, data: BinaryOp::Div }, Prec::MULTIPLICATIVE.succ())?
                },
                // TODO: parse function calls here
                // any unexpected token is not consumed, this is a problem for the caller
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_prefix_expr(&mut self) -> Parse<ast::Expr> {
        let (token, span) = match self.peek() {
            (None, span) => return Err(ParseError::unexpected_str_eof(span, "expression")),
            (Some(token), span) => (token, span),
        };
        match token {
            Token::Plus => self.parse_unary_operand(ast::UnaryOp::Plus, span),
            Token::Minus => self.parse_unary_operand(ast::UnaryOp::Minus, span),
            Token::LParen => self.parse_paren_expr(),
            Token::LitInt => self.parse_int_expr(),
            Token::LitFloat => todo!("float literal"),
            Token::LitRatio => todo!("ratio literal"),
            Token::LitString => todo!("string literal"),
            _ => Err(ParseError::expected_str_but_got(span, "expression", token)),
        }
    }

    fn parse_unary_operand(&mut self, op: ast::UnaryOp, op_span: Span) -> Parse<ast::Expr> {
        // assumes that the caller did not consume the operand yet
        self.consume();
        let expr = self.parse_prec_expr(Prec::UNARY)?;
        Ok(Node {
            span: op_span.start..expr.span.end,
            data: ast::Expr::Unary {
                operator: Node {
                    span: op_span,
                    data: op,
                },
                operand: Arc::new(expr),
            },
        })
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
        Ok(Node {
            span: left.span.start..expr.span.end,
            data: ast::Expr::Binary {
                left: Arc::new(left),
                operator,
                right: Arc::new(expr),
            },
        })
    }

    fn parse_paren_expr(&mut self) -> Parse<ast::Expr> {
        let lparen = self.parse_expect_token(Token::LParen)?;
        let expr = Arc::new(self.parse_expr()?);
        let rparen = self.parse_expect_token(Token::RParen)?;
        Ok(Node {
            span: lparen.span.start..rparen.span.end,
            data: ast::Expr::Paren {
                lparen,
                expr,
                rparen,
            },
        })
    }

    fn parse_int(&mut self) -> Parse<i64> {
        let node = self.parse_expect_token(Token::LitInt)?;
        match self.source[node.span.clone()].parse::<i64>() {
            Ok(int) => Ok(Node {
                span: node.span,
                data: int,
            }),
            Err(err) => Err(ParseError {
                span: node.span,
                message: format!("{}", err),
            }),
        }
    }

    fn parse_int_expr(&mut self) -> Parse<ast::Expr> {
        let int = self.parse_int()?;
        Ok(Node {
            span: int.span,
            data: ast::Expr::Int(int.data),
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

    fn check_expr(input: &str, output: Expect) {
        let mut parser = Parser::new(input);
        let result = parser.parse_expr();
        let debug = format!("{:#?}", result);
        output.assert_eq(&debug);
    }

    #[test]
    fn parse_empty() {
        check(
            "",
            expect![[r#"
                Err(
                    ParseError {
                        span: 0..0,
                        message: "Expected one of [Ident], but reached end of file",
                    },
                )"#]],
        );
    }

    #[test]
    fn parse_empty_object() {
        check(
            r"Song {
            // super awesome song, eventually
        }",
            expect![[r#"
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
                                    attrs: [],
                                    children: [],
                                },
                            },
                        },
                    },
                )"#]],
        );
    }

    #[test]
    fn parse_nested_objects() {
        check(
            r"Song {
            // super awesome song with several tracks
            Track {

            }
            Track {

            }
        }",
            expect![[r#"
                Ok(
                    Node {
                        span: 0..140,
                        data: Root {
                            object: Node {
                                span: 0..140,
                                data: Object {
                                    name: Node {
                                        span: 0..4,
                                        data: "Song",
                                    },
                                    attrs: [],
                                    children: [
                                        Node {
                                            span: 73..95,
                                            data: Object {
                                                name: Node {
                                                    span: 73..78,
                                                    data: "Track",
                                                },
                                                attrs: [],
                                                children: [],
                                            },
                                        },
                                        Node {
                                            span: 108..130,
                                            data: Object {
                                                name: Node {
                                                    span: 108..113,
                                                    data: "Track",
                                                },
                                                attrs: [],
                                                children: [],
                                            },
                                        },
                                    ],
                                },
                            },
                        },
                    },
                )"#]],
        );
    }

    #[test]
    fn parse_expr_int_lit() {
        check_expr(
            "-+1337",
            expect![[r#"
                Ok(
                    Node {
                        span: 0..6,
                        data: Unary {
                            operator: Node {
                                span: 0..1,
                                data: Minus,
                            },
                            operand: Node {
                                span: 1..6,
                                data: Int(
                                    1337,
                                ),
                            },
                        },
                    },
                )"#]],
        );
    }

    #[test]
    fn parse_expr_unary_parens() {
        check_expr(
            "-(-(1337))",
            expect![[r#"
                Ok(
                    Node {
                        span: 0..10,
                        data: Unary {
                            operator: Node {
                                span: 0..1,
                                data: Minus,
                            },
                            operand: Node {
                                span: 1..10,
                                data: Paren {
                                    lparen: Node {
                                        span: 1..2,
                                        data: (),
                                    },
                                    expr: Node {
                                        span: 2..9,
                                        data: Unary {
                                            operator: Node {
                                                span: 2..3,
                                                data: Minus,
                                            },
                                            operand: Node {
                                                span: 3..9,
                                                data: Paren {
                                                    lparen: Node {
                                                        span: 3..4,
                                                        data: (),
                                                    },
                                                    expr: Node {
                                                        span: 4..8,
                                                        data: Int(
                                                            1337,
                                                        ),
                                                    },
                                                    rparen: Node {
                                                        span: 8..9,
                                                        data: (),
                                                    },
                                                },
                                            },
                                        },
                                    },
                                    rparen: Node {
                                        span: 9..10,
                                        data: (),
                                    },
                                },
                            },
                        },
                    },
                )"#]],
        );
    }

    #[test]
    fn parse_expr_int_lit_too_big() {
        check_expr(
            "2305972057823905702935709237509237509237509237509237059273057",
            expect![[r#"
                Err(
                    ParseError {
                        span: 0..61,
                        message: "number too large to fit in target type",
                    },
                )"#]],
        );
    }

    #[test]
    fn parse_expr_infix() {
        check_expr(
            "2 * 4 + 17 * (1 + 1)",
            expect![[r#"
                Ok(
                    Node {
                        span: 0..20,
                        data: Binary {
                            left: Node {
                                span: 0..5,
                                data: Binary {
                                    left: Node {
                                        span: 0..1,
                                        data: Int(
                                            2,
                                        ),
                                    },
                                    operator: Node {
                                        span: 2..3,
                                        data: Mult,
                                    },
                                    right: Node {
                                        span: 4..5,
                                        data: Int(
                                            4,
                                        ),
                                    },
                                },
                            },
                            operator: Node {
                                span: 6..7,
                                data: Add,
                            },
                            right: Node {
                                span: 8..20,
                                data: Binary {
                                    left: Node {
                                        span: 8..10,
                                        data: Int(
                                            17,
                                        ),
                                    },
                                    operator: Node {
                                        span: 11..12,
                                        data: Mult,
                                    },
                                    right: Node {
                                        span: 13..20,
                                        data: Paren {
                                            lparen: Node {
                                                span: 13..14,
                                                data: (),
                                            },
                                            expr: Node {
                                                span: 14..19,
                                                data: Binary {
                                                    left: Node {
                                                        span: 14..15,
                                                        data: Int(
                                                            1,
                                                        ),
                                                    },
                                                    operator: Node {
                                                        span: 16..17,
                                                        data: Add,
                                                    },
                                                    right: Node {
                                                        span: 18..19,
                                                        data: Int(
                                                            1,
                                                        ),
                                                    },
                                                },
                                            },
                                            rparen: Node {
                                                span: 19..20,
                                                data: (),
                                            },
                                        },
                                    },
                                },
                            },
                        },
                    },
                )"#]],
        );
    }
}
