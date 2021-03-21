use super::Parser;
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
                    span: 0..46,
                    data: Root {
                        object: Node {
                            span: 0..46,
                            data: Object {
                                name: Node {
                                    span: 0..4,
                                    data: "Song",
                                },
                                lbrace: Node {
                                    span: 5..6,
                                    data: (),
                                },
                                attrs: [],
                                children: [],
                                rbrace: Node {
                                    span: 45..46,
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
                    span: 0..92,
                    data: Root {
                        object: Node {
                            span: 0..92,
                            data: Object {
                                name: Node {
                                    span: 0..4,
                                    data: "Song",
                                },
                                lbrace: Node {
                                    span: 5..6,
                                    data: (),
                                },
                                attrs: [],
                                children: [
                                    Node {
                                        span: 57..71,
                                        data: Object {
                                            name: Node {
                                                span: 57..62,
                                                data: "Track",
                                            },
                                            lbrace: Node {
                                                span: 63..64,
                                                data: (),
                                            },
                                            attrs: [],
                                            children: [],
                                            rbrace: Node {
                                                span: 70..71,
                                                data: (),
                                            },
                                        },
                                    },
                                    Node {
                                        span: 76..90,
                                        data: Object {
                                            name: Node {
                                                span: 76..81,
                                                data: "Track",
                                            },
                                            lbrace: Node {
                                                span: 82..83,
                                                data: (),
                                            },
                                            attrs: [],
                                            children: [],
                                            rbrace: Node {
                                                span: 89..90,
                                                data: (),
                                            },
                                        },
                                    },
                                ],
                                rbrace: Node {
                                    span: 91..92,
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
fn parse_nested_objects_with_attrs() {
    check(
        r#"Song {
    bpm: 120
    // super awesome song with several tracks
    Track {
        frob: 42
    }
    Track {
        frob: 1337
    }
}"#,
        expect![[r#"
            Ok(
                Node {
                    span: 0..139,
                    data: Root {
                        object: Node {
                            span: 0..139,
                            data: Object {
                                name: Node {
                                    span: 0..4,
                                    data: "Song",
                                },
                                lbrace: Node {
                                    span: 5..6,
                                    data: (),
                                },
                                attrs: [
                                    Node {
                                        span: 11..19,
                                        data: Attribute {
                                            name: Node {
                                                span: 11..14,
                                                data: "bpm",
                                            },
                                            colon: Node {
                                                span: 14..15,
                                                data: (),
                                            },
                                            value: Node {
                                                span: 16..19,
                                                data: Int(
                                                    120,
                                                ),
                                            },
                                        },
                                    },
                                ],
                                children: [
                                    Node {
                                        span: 70..100,
                                        data: Object {
                                            name: Node {
                                                span: 70..75,
                                                data: "Track",
                                            },
                                            lbrace: Node {
                                                span: 76..77,
                                                data: (),
                                            },
                                            attrs: [
                                                Node {
                                                    span: 86..94,
                                                    data: Attribute {
                                                        name: Node {
                                                            span: 86..90,
                                                            data: "frob",
                                                        },
                                                        colon: Node {
                                                            span: 90..91,
                                                            data: (),
                                                        },
                                                        value: Node {
                                                            span: 92..94,
                                                            data: Int(
                                                                42,
                                                            ),
                                                        },
                                                    },
                                                },
                                            ],
                                            children: [],
                                            rbrace: Node {
                                                span: 99..100,
                                                data: (),
                                            },
                                        },
                                    },
                                    Node {
                                        span: 105..137,
                                        data: Object {
                                            name: Node {
                                                span: 105..110,
                                                data: "Track",
                                            },
                                            lbrace: Node {
                                                span: 111..112,
                                                data: (),
                                            },
                                            attrs: [
                                                Node {
                                                    span: 121..131,
                                                    data: Attribute {
                                                        name: Node {
                                                            span: 121..125,
                                                            data: "frob",
                                                        },
                                                        colon: Node {
                                                            span: 125..126,
                                                            data: (),
                                                        },
                                                        value: Node {
                                                            span: 127..131,
                                                            data: Int(
                                                                1337,
                                                            ),
                                                        },
                                                    },
                                                },
                                            ],
                                            children: [],
                                            rbrace: Node {
                                                span: 136..137,
                                                data: (),
                                            },
                                        },
                                    },
                                ],
                                rbrace: Node {
                                    span: 138..139,
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
