use std::{fmt::Display, iter::Peekable, str::FromStr, sync::Arc};

use ast::BinaryOp;
use logos::{Logos, SpannedIter};

use crate::lexer::{Span, Token};

#[cfg(test)]
mod expect_tests;

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
        pub lbrace: Node<()>,
        pub attrs: Vec<Node<Attribute>>,
        pub children: Vec<Node<Object>>,
        pub rbrace: Node<()>,
    }

    #[derive(Debug, Clone)]
    pub struct Attribute {
        pub name: Node<String>,
        pub colon: Node<()>,
        pub value: Node<Expr>,
    }

    #[derive(Debug, Clone)]
    pub enum Expr {
        String(String),
        Int(i64),
        Float(f64),
        Bool(bool),
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
        Object(Node<Object>),
        Var(String),
        Accessor { expr: NodePtr<Expr>, dot: Node<()>, attribute: Node<String> }
    }

    #[derive(Debug, Clone)]
    pub enum UnaryOp {
        Plus,
        Minus,
        Not,
    }

    #[derive(Debug, Clone)]
    pub enum BinaryOp {
        Add,
        Sub,
        Mult,
        Div,
        And,
        Or,
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
                            let value = self.parse_expr()?;
                            attrs.push(Node {
                                span: inner_name.span.start..value.span.end,
                                data: ast::Attribute {
                                    name: inner_name,
                                    colon,
                                    value,
                                },
                            })
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
                lbrace,
                attrs,
                children,
                rbrace,
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

            left = match token {
                // Infix operations
                Token::Or if min_prec <= Prec::DISJUNCTIVE => self.parse_binary_operand(
                    left,
                    Node {
                        span,
                        data: BinaryOp::Or,
                    },
                    Prec::DISJUNCTIVE.succ(),
                )?,
                Token::And if min_prec <= Prec::CONJUNCTIVE => self.parse_binary_operand(
                    left,
                    Node {
                        span,
                        data: BinaryOp::And,
                    },
                    Prec::CONJUNCTIVE.succ(),
                )?,
                Token::Plus if min_prec <= Prec::ADDITIVE => self.parse_binary_operand(
                    left,
                    Node {
                        span,
                        data: BinaryOp::Add,
                    },
                    Prec::ADDITIVE.succ(),
                )?,
                Token::Minus if min_prec <= Prec::ADDITIVE => self.parse_binary_operand(
                    left,
                    Node {
                        span,
                        data: BinaryOp::Sub,
                    },
                    Prec::ADDITIVE.succ(),
                )?,
                Token::Star if min_prec <= Prec::MULTIPLICATIVE => self.parse_binary_operand(
                    left,
                    Node {
                        span,
                        data: BinaryOp::Mult,
                    },
                    Prec::MULTIPLICATIVE.succ(),
                )?,
                Token::Slash if min_prec <= Prec::MULTIPLICATIVE => self.parse_binary_operand(
                    left,
                    Node {
                        span,
                        data: BinaryOp::Div,
                    },
                    Prec::MULTIPLICATIVE.succ(),
                )?,
                // Postfix operations
                Token::Dot if min_prec <= Prec::DOT => {
                    let dot = self.parse_expect_token(Token::Dot)?;
                    let attribute = self.parse_ident()?;
                    Node {
                        span: left.span.start .. attribute.span.end,
                        data: ast::Expr::Accessor {
                            expr: Arc::new(left),
                            dot,
                            attribute,
                        }
                    }
                }
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
            Token::LitFloat => self.parse_float_expr(),
            Token::LitRatio => todo!("ratio literal"),
            Token::LitString => self.parse_string_expr(),
            Token::LitBool => self.parse_bool_expr(),
            Token::Ident => {
                let name = self.parse_ident()?;

                // Can either be an object or a normal identifier, depending on what follows
                if let Some(Token::LBrace) = self.peek().0 {
                    let object = self.parse_object_body(name)?;
                    Ok(Node {
                        span: object.span.clone(),
                        data: ast::Expr::Object(object),
                    })
                } else {
                    Ok(Node {
                        span: name.span,
                        data: ast::Expr::Var(name.data),
                    })
                }
            }
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

    fn parse_native<T: FromStr>(&mut self, token: Token) -> Parse<T>
    where
        T::Err: Display,
    {
        let node = self.parse_expect_token(token)?;
        match self.source[node.span.clone()].parse::<T>() {
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
        let int = self.parse_native(Token::LitInt)?;
        Ok(Node {
            span: int.span,
            data: ast::Expr::Int(int.data),
        })
    }

    fn parse_float_expr(&mut self) -> Parse<ast::Expr> {
        let float = self.parse_native(Token::LitFloat)?;
        Ok(Node {
            span: float.span,
            data: ast::Expr::Float(float.data),
        })
    }

    fn parse_bool_expr(&mut self) -> Parse<ast::Expr> {
        let bool = self.parse_native(Token::LitBool)?;
        Ok(Node {
            span: bool.span,
            data: ast::Expr::Bool(bool.data),
        })
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
                        return Err(ParseError {
                            span: start_index..start_index + 1,
                            message: "unterminated escape sequence".into(),
                        })
                    }
                    Some((index, ch)) => {
                        let end_index = node.span.start + 1 + index + ch.len_utf8();
                        match ch {
                            'n' => data.push('\n'),
                            't' => data.push('\t'),
                            'r' => data.push('\r'),
                            '\\' => data.push('\\'),
                            _ => {
                                return Err(ParseError {
                                    span: start_index..end_index,
                                    message: "unknown escape sequence".into(),
                                })
                            }
                        }
                    }
                }
            } else {
                data.push(ch);
            }
        }
        Ok(Node {
            span: node.span,
            data,
        })
    }

    fn parse_string_expr(&mut self) -> Parse<ast::Expr> {
        let string = self.parse_string()?;
        Ok(Node {
            span: string.span,
            data: ast::Expr::String(string.data),
        })
    }
}
