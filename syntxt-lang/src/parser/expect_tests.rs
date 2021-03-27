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
        id: lead
        frob: 42
    }
    Track {
        id: drums
        frob: 1337
    }
}"#,
        expect![[r#"
            Ok(
                Node {
                    span: 0..174,
                    data: Root {
                        object: Node {
                            span: 0..174,
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
                                        span: 70..117,
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
                                                            span: 86..88,
                                                            data: "id",
                                                        },
                                                        colon: Node {
                                                            span: 88..89,
                                                            data: (),
                                                        },
                                                        value: Node {
                                                            span: 90..94,
                                                            data: Var(
                                                                "lead",
                                                            ),
                                                        },
                                                    },
                                                },
                                                Node {
                                                    span: 103..111,
                                                    data: Attribute {
                                                        name: Node {
                                                            span: 103..107,
                                                            data: "frob",
                                                        },
                                                        colon: Node {
                                                            span: 107..108,
                                                            data: (),
                                                        },
                                                        value: Node {
                                                            span: 109..111,
                                                            data: Int(
                                                                42,
                                                            ),
                                                        },
                                                    },
                                                },
                                            ],
                                            children: [],
                                            rbrace: Node {
                                                span: 116..117,
                                                data: (),
                                            },
                                        },
                                    },
                                    Node {
                                        span: 122..172,
                                        data: Object {
                                            name: Node {
                                                span: 122..127,
                                                data: "Track",
                                            },
                                            lbrace: Node {
                                                span: 128..129,
                                                data: (),
                                            },
                                            attrs: [
                                                Node {
                                                    span: 138..147,
                                                    data: Attribute {
                                                        name: Node {
                                                            span: 138..140,
                                                            data: "id",
                                                        },
                                                        colon: Node {
                                                            span: 140..141,
                                                            data: (),
                                                        },
                                                        value: Node {
                                                            span: 142..147,
                                                            data: Var(
                                                                "drums",
                                                            ),
                                                        },
                                                    },
                                                },
                                                Node {
                                                    span: 156..166,
                                                    data: Attribute {
                                                        name: Node {
                                                            span: 156..160,
                                                            data: "frob",
                                                        },
                                                        colon: Node {
                                                            span: 160..161,
                                                            data: (),
                                                        },
                                                        value: Node {
                                                            span: 162..166,
                                                            data: Int(
                                                                1337,
                                                            ),
                                                        },
                                                    },
                                                },
                                            ],
                                            children: [],
                                            rbrace: Node {
                                                span: 171..172,
                                                data: (),
                                            },
                                        },
                                    },
                                ],
                                rbrace: Node {
                                    span: 173..174,
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
fn parse_attr_objects() {
    check(
        r#"Song {
    bpm: 120
    meta: Meta {
        name: "Example Song"
        author: "John Doe"
        year: 2021
        description: "Simply.\nAwesome."
    }
}"#,
        expect![[r#"
            Ok(
                Node {
                    span: 0..160,
                    data: Root {
                        object: Node {
                            span: 0..160,
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
                                    Node {
                                        span: 24..158,
                                        data: Attribute {
                                            name: Node {
                                                span: 24..28,
                                                data: "meta",
                                            },
                                            colon: Node {
                                                span: 28..29,
                                                data: (),
                                            },
                                            value: Node {
                                                span: 30..158,
                                                data: Object(
                                                    Node {
                                                        span: 30..158,
                                                        data: Object {
                                                            name: Node {
                                                                span: 30..34,
                                                                data: "Meta",
                                                            },
                                                            lbrace: Node {
                                                                span: 35..36,
                                                                data: (),
                                                            },
                                                            attrs: [
                                                                Node {
                                                                    span: 45..65,
                                                                    data: Attribute {
                                                                        name: Node {
                                                                            span: 45..49,
                                                                            data: "name",
                                                                        },
                                                                        colon: Node {
                                                                            span: 49..50,
                                                                            data: (),
                                                                        },
                                                                        value: Node {
                                                                            span: 51..65,
                                                                            data: String(
                                                                                "Example Song",
                                                                            ),
                                                                        },
                                                                    },
                                                                },
                                                                Node {
                                                                    span: 74..92,
                                                                    data: Attribute {
                                                                        name: Node {
                                                                            span: 74..80,
                                                                            data: "author",
                                                                        },
                                                                        colon: Node {
                                                                            span: 80..81,
                                                                            data: (),
                                                                        },
                                                                        value: Node {
                                                                            span: 82..92,
                                                                            data: String(
                                                                                "John Doe",
                                                                            ),
                                                                        },
                                                                    },
                                                                },
                                                                Node {
                                                                    span: 101..111,
                                                                    data: Attribute {
                                                                        name: Node {
                                                                            span: 101..105,
                                                                            data: "year",
                                                                        },
                                                                        colon: Node {
                                                                            span: 105..106,
                                                                            data: (),
                                                                        },
                                                                        value: Node {
                                                                            span: 107..111,
                                                                            data: Int(
                                                                                2021,
                                                                            ),
                                                                        },
                                                                    },
                                                                },
                                                                Node {
                                                                    span: 120..152,
                                                                    data: Attribute {
                                                                        name: Node {
                                                                            span: 120..131,
                                                                            data: "description",
                                                                        },
                                                                        colon: Node {
                                                                            span: 131..132,
                                                                            data: (),
                                                                        },
                                                                        value: Node {
                                                                            span: 133..152,
                                                                            data: String(
                                                                                "Simply.\nAwesome.",
                                                                            ),
                                                                        },
                                                                    },
                                                                },
                                                            ],
                                                            children: [],
                                                            rbrace: Node {
                                                                span: 157..158,
                                                                data: (),
                                                            },
                                                        },
                                                    },
                                                ),
                                            },
                                        },
                                    },
                                ],
                                children: [],
                                rbrace: Node {
                                    span: 159..160,
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
fn parse_expr_int_lit_underscores() {
    check_expr(
        "1_337",
        expect![[r#"
            Ok(
                Node {
                    span: 0..5,
                    data: Int(
                        1337,
                    ),
                },
            )"#]],
    );
}

#[test]
fn parse_expr_ratio_lit() {
    check_expr(
        "-1337/42",
        expect![[r#"
            Ok(
                Node {
                    span: 0..8,
                    data: Ratio(
                        Rational {
                            num: -191,
                            denom: 6,
                        },
                    ),
                },
            )"#]],
    );
}

#[test]
fn parse_expr_ratio_lit_underscores() {
    check_expr(
        "+1_337/42",
        expect![[r#"
            Ok(
                Node {
                    span: 0..9,
                    data: Ratio(
                        Rational {
                            num: 191,
                            denom: 6,
                        },
                    ),
                },
            )"#]],
    );
}

#[test]
fn parse_expr_invalid_ratio() {
    check_expr(
        "+1_337/0",
        expect![[r#"
            Err(
                ParseError {
                    span: 0..8,
                    message: "denominator is zero",
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

#[test]
fn parse_expr_string() {
    check_expr(
        r#""beautiful \n escape\t sequences\\ don't you \tthink?""#,
        expect![[r#"
            Ok(
                Node {
                    span: 0..54,
                    data: String(
                        "beautiful \n escape\t sequences\\ don\'t you \tthink?",
                    ),
                },
            )"#]],
    );
}

#[test]
fn parse_invalid_escape() {
    check_expr(
        r#""Broken \escape sequences are the worst""#,
        expect![[r#"
            Err(
                ParseError {
                    span: 8..10,
                    message: "unknown escape sequence",
                },
            )"#]],
    );
}

#[test]
fn parse_expr_bool() {
    check_expr(
        "true and false or false and true",
        expect![[r#"
            Ok(
                Node {
                    span: 0..32,
                    data: Binary {
                        left: Node {
                            span: 0..14,
                            data: Binary {
                                left: Node {
                                    span: 0..4,
                                    data: Bool(
                                        true,
                                    ),
                                },
                                operator: Node {
                                    span: 5..8,
                                    data: And,
                                },
                                right: Node {
                                    span: 9..14,
                                    data: Bool(
                                        false,
                                    ),
                                },
                            },
                        },
                        operator: Node {
                            span: 15..17,
                            data: Or,
                        },
                        right: Node {
                            span: 18..32,
                            data: Binary {
                                left: Node {
                                    span: 18..23,
                                    data: Bool(
                                        false,
                                    ),
                                },
                                operator: Node {
                                    span: 24..27,
                                    data: And,
                                },
                                right: Node {
                                    span: 28..32,
                                    data: Bool(
                                        true,
                                    ),
                                },
                            },
                        },
                    },
                },
            )"#]],
    );
}

#[test]
fn parse_dot_exprs() {
    check_expr(
        "foo.bar.baz",
        expect![[r#"
            Ok(
                Node {
                    span: 0..11,
                    data: Accessor {
                        expr: Node {
                            span: 0..7,
                            data: Accessor {
                                expr: Node {
                                    span: 0..3,
                                    data: Var(
                                        "foo",
                                    ),
                                },
                                dot: Node {
                                    span: 3..4,
                                    data: (),
                                },
                                attribute: Node {
                                    span: 4..7,
                                    data: "bar",
                                },
                            },
                        },
                        dot: Node {
                            span: 7..8,
                            data: (),
                        },
                        attribute: Node {
                            span: 8..11,
                            data: "baz",
                        },
                    },
                },
            )"#]],
    );
}

#[test]
fn parse_dot_exprs_parens() {
    check_expr(
        "(foo.bar).baz",
        expect![[r#"
            Ok(
                Node {
                    span: 0..13,
                    data: Accessor {
                        expr: Node {
                            span: 0..9,
                            data: Paren {
                                lparen: Node {
                                    span: 0..1,
                                    data: (),
                                },
                                expr: Node {
                                    span: 1..8,
                                    data: Accessor {
                                        expr: Node {
                                            span: 1..4,
                                            data: Var(
                                                "foo",
                                            ),
                                        },
                                        dot: Node {
                                            span: 4..5,
                                            data: (),
                                        },
                                        attribute: Node {
                                            span: 5..8,
                                            data: "bar",
                                        },
                                    },
                                },
                                rparen: Node {
                                    span: 8..9,
                                    data: (),
                                },
                            },
                        },
                        dot: Node {
                            span: 9..10,
                            data: (),
                        },
                        attribute: Node {
                            span: 10..13,
                            data: "baz",
                        },
                    },
                },
            )"#]],
    );
}

#[test]
fn parse_dot_exprs_invalid_parens() {
    check_expr(
        "foo.(bar.baz)",
        expect![[r#"
            Err(
                ParseError {
                    span: 4..5,
                    message: "Expected one of [Ident], but got LParen",
                },
            )"#]],
    );
}

#[test]
fn parse_call() {
    check_expr(
        "atan2(1, 2, ) + -foo.frob(42, sin(pi))",
        expect![[r#"
            Ok(
                Node {
                    span: 0..38,
                    data: Binary {
                        left: Node {
                            span: 0..13,
                            data: Call {
                                callee: Node {
                                    span: 0..5,
                                    data: Var(
                                        "atan2",
                                    ),
                                },
                                lparen: Node {
                                    span: 5..6,
                                    data: (),
                                },
                                arguments: [
                                    Node {
                                        span: 6..7,
                                        data: Int(
                                            1,
                                        ),
                                    },
                                    Node {
                                        span: 9..10,
                                        data: Int(
                                            2,
                                        ),
                                    },
                                ],
                                rparen: Node {
                                    span: 12..13,
                                    data: (),
                                },
                            },
                        },
                        operator: Node {
                            span: 14..15,
                            data: Add,
                        },
                        right: Node {
                            span: 16..38,
                            data: Unary {
                                operator: Node {
                                    span: 16..17,
                                    data: Minus,
                                },
                                operand: Node {
                                    span: 17..38,
                                    data: Call {
                                        callee: Node {
                                            span: 17..25,
                                            data: Accessor {
                                                expr: Node {
                                                    span: 17..20,
                                                    data: Var(
                                                        "foo",
                                                    ),
                                                },
                                                dot: Node {
                                                    span: 20..21,
                                                    data: (),
                                                },
                                                attribute: Node {
                                                    span: 21..25,
                                                    data: "frob",
                                                },
                                            },
                                        },
                                        lparen: Node {
                                            span: 25..26,
                                            data: (),
                                        },
                                        arguments: [
                                            Node {
                                                span: 26..28,
                                                data: Int(
                                                    42,
                                                ),
                                            },
                                            Node {
                                                span: 30..37,
                                                data: Call {
                                                    callee: Node {
                                                        span: 30..33,
                                                        data: Var(
                                                            "sin",
                                                        ),
                                                    },
                                                    lparen: Node {
                                                        span: 33..34,
                                                        data: (),
                                                    },
                                                    arguments: [
                                                        Node {
                                                            span: 34..36,
                                                            data: Var(
                                                                "pi",
                                                            ),
                                                        },
                                                    ],
                                                    rparen: Node {
                                                        span: 36..37,
                                                        data: (),
                                                    },
                                                },
                                            },
                                        ],
                                        rparen: Node {
                                            span: 37..38,
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
