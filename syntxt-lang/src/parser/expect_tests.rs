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
    let result = Parser::parse(input);
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
            Ok(
                Node {
                    span: 0..0,
                    pos: 1:1..1:1,
                    data: Root {
                        objects: [],
                    },
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
                    pos: 1:1..3:2,
                    data: Root {
                        objects: [
                            Node {
                                span: 0..46,
                                pos: 1:1..3:2,
                                data: Object {
                                    name: Node {
                                        span: 0..4,
                                        pos: 1:1..1:5,
                                        data: "Song",
                                    },
                                    lbrace: Node {
                                        span: 5..6,
                                        pos: 1:6..1:7,
                                        data: (),
                                    },
                                    attrs: [],
                                    children: [],
                                    rbrace: Node {
                                        span: 45..46,
                                        pos: 3:1..3:2,
                                        data: (),
                                    },
                                },
                            },
                        ],
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
                    pos: 1:1..9:2,
                    data: Root {
                        objects: [
                            Node {
                                span: 0..92,
                                pos: 1:1..9:2,
                                data: Object {
                                    name: Node {
                                        span: 0..4,
                                        pos: 1:1..1:5,
                                        data: "Song",
                                    },
                                    lbrace: Node {
                                        span: 5..6,
                                        pos: 1:6..1:7,
                                        data: (),
                                    },
                                    attrs: [],
                                    children: [
                                        Node {
                                            span: 57..71,
                                            pos: 3:5..5:6,
                                            data: Object {
                                                name: Node {
                                                    span: 57..62,
                                                    pos: 3:5..3:10,
                                                    data: "Track",
                                                },
                                                lbrace: Node {
                                                    span: 63..64,
                                                    pos: 3:11..3:12,
                                                    data: (),
                                                },
                                                attrs: [],
                                                children: [],
                                                rbrace: Node {
                                                    span: 70..71,
                                                    pos: 5:5..5:6,
                                                    data: (),
                                                },
                                            },
                                        },
                                        Node {
                                            span: 76..90,
                                            pos: 6:5..8:6,
                                            data: Object {
                                                name: Node {
                                                    span: 76..81,
                                                    pos: 6:5..6:10,
                                                    data: "Track",
                                                },
                                                lbrace: Node {
                                                    span: 82..83,
                                                    pos: 6:11..6:12,
                                                    data: (),
                                                },
                                                attrs: [],
                                                children: [],
                                                rbrace: Node {
                                                    span: 89..90,
                                                    pos: 8:5..8:6,
                                                    data: (),
                                                },
                                            },
                                        },
                                    ],
                                    rbrace: Node {
                                        span: 91..92,
                                        pos: 9:1..9:2,
                                        data: (),
                                    },
                                },
                            },
                        ],
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
                    pos: 1:1..12:2,
                    data: Root {
                        objects: [
                            Node {
                                span: 0..174,
                                pos: 1:1..12:2,
                                data: Object {
                                    name: Node {
                                        span: 0..4,
                                        pos: 1:1..1:5,
                                        data: "Song",
                                    },
                                    lbrace: Node {
                                        span: 5..6,
                                        pos: 1:6..1:7,
                                        data: (),
                                    },
                                    attrs: [
                                        Node {
                                            span: 11..19,
                                            pos: 2:5..2:13,
                                            data: Attribute {
                                                name: Node {
                                                    span: 11..14,
                                                    pos: 2:5..2:8,
                                                    data: "bpm",
                                                },
                                                colon: Node {
                                                    span: 14..15,
                                                    pos: 2:8..2:9,
                                                    data: (),
                                                },
                                                value: Node {
                                                    span: 16..19,
                                                    pos: 2:10..2:13,
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
                                            pos: 4:5..7:6,
                                            data: Object {
                                                name: Node {
                                                    span: 70..75,
                                                    pos: 4:5..4:10,
                                                    data: "Track",
                                                },
                                                lbrace: Node {
                                                    span: 76..77,
                                                    pos: 4:11..4:12,
                                                    data: (),
                                                },
                                                attrs: [
                                                    Node {
                                                        span: 86..94,
                                                        pos: 5:9..5:17,
                                                        data: Attribute {
                                                            name: Node {
                                                                span: 86..88,
                                                                pos: 5:9..5:11,
                                                                data: "id",
                                                            },
                                                            colon: Node {
                                                                span: 88..89,
                                                                pos: 5:11..5:12,
                                                                data: (),
                                                            },
                                                            value: Node {
                                                                span: 90..94,
                                                                pos: 5:13..5:17,
                                                                data: Var(
                                                                    "lead",
                                                                ),
                                                            },
                                                        },
                                                    },
                                                    Node {
                                                        span: 103..111,
                                                        pos: 6:9..6:17,
                                                        data: Attribute {
                                                            name: Node {
                                                                span: 103..107,
                                                                pos: 6:9..6:13,
                                                                data: "frob",
                                                            },
                                                            colon: Node {
                                                                span: 107..108,
                                                                pos: 6:13..6:14,
                                                                data: (),
                                                            },
                                                            value: Node {
                                                                span: 109..111,
                                                                pos: 6:15..6:17,
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
                                                    pos: 7:5..7:6,
                                                    data: (),
                                                },
                                            },
                                        },
                                        Node {
                                            span: 122..172,
                                            pos: 8:5..11:6,
                                            data: Object {
                                                name: Node {
                                                    span: 122..127,
                                                    pos: 8:5..8:10,
                                                    data: "Track",
                                                },
                                                lbrace: Node {
                                                    span: 128..129,
                                                    pos: 8:11..8:12,
                                                    data: (),
                                                },
                                                attrs: [
                                                    Node {
                                                        span: 138..147,
                                                        pos: 9:9..9:18,
                                                        data: Attribute {
                                                            name: Node {
                                                                span: 138..140,
                                                                pos: 9:9..9:11,
                                                                data: "id",
                                                            },
                                                            colon: Node {
                                                                span: 140..141,
                                                                pos: 9:11..9:12,
                                                                data: (),
                                                            },
                                                            value: Node {
                                                                span: 142..147,
                                                                pos: 9:13..9:18,
                                                                data: Var(
                                                                    "drums",
                                                                ),
                                                            },
                                                        },
                                                    },
                                                    Node {
                                                        span: 156..166,
                                                        pos: 10:9..10:19,
                                                        data: Attribute {
                                                            name: Node {
                                                                span: 156..160,
                                                                pos: 10:9..10:13,
                                                                data: "frob",
                                                            },
                                                            colon: Node {
                                                                span: 160..161,
                                                                pos: 10:13..10:14,
                                                                data: (),
                                                            },
                                                            value: Node {
                                                                span: 162..166,
                                                                pos: 10:15..10:19,
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
                                                    pos: 11:5..11:6,
                                                    data: (),
                                                },
                                            },
                                        },
                                    ],
                                    rbrace: Node {
                                        span: 173..174,
                                        pos: 12:1..12:2,
                                        data: (),
                                    },
                                },
                            },
                        ],
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
                    pos: 1:1..9:2,
                    data: Root {
                        objects: [
                            Node {
                                span: 0..160,
                                pos: 1:1..9:2,
                                data: Object {
                                    name: Node {
                                        span: 0..4,
                                        pos: 1:1..1:5,
                                        data: "Song",
                                    },
                                    lbrace: Node {
                                        span: 5..6,
                                        pos: 1:6..1:7,
                                        data: (),
                                    },
                                    attrs: [
                                        Node {
                                            span: 11..19,
                                            pos: 2:5..2:13,
                                            data: Attribute {
                                                name: Node {
                                                    span: 11..14,
                                                    pos: 2:5..2:8,
                                                    data: "bpm",
                                                },
                                                colon: Node {
                                                    span: 14..15,
                                                    pos: 2:8..2:9,
                                                    data: (),
                                                },
                                                value: Node {
                                                    span: 16..19,
                                                    pos: 2:10..2:13,
                                                    data: Int(
                                                        120,
                                                    ),
                                                },
                                            },
                                        },
                                        Node {
                                            span: 24..158,
                                            pos: 3:5..8:6,
                                            data: Attribute {
                                                name: Node {
                                                    span: 24..28,
                                                    pos: 3:5..3:9,
                                                    data: "meta",
                                                },
                                                colon: Node {
                                                    span: 28..29,
                                                    pos: 3:9..3:10,
                                                    data: (),
                                                },
                                                value: Node {
                                                    span: 30..158,
                                                    pos: 3:11..8:6,
                                                    data: Object(
                                                        Node {
                                                            span: 30..158,
                                                            pos: 3:11..8:6,
                                                            data: Object {
                                                                name: Node {
                                                                    span: 30..34,
                                                                    pos: 3:11..3:15,
                                                                    data: "Meta",
                                                                },
                                                                lbrace: Node {
                                                                    span: 35..36,
                                                                    pos: 3:16..3:17,
                                                                    data: (),
                                                                },
                                                                attrs: [
                                                                    Node {
                                                                        span: 45..65,
                                                                        pos: 4:9..4:29,
                                                                        data: Attribute {
                                                                            name: Node {
                                                                                span: 45..49,
                                                                                pos: 4:9..4:13,
                                                                                data: "name",
                                                                            },
                                                                            colon: Node {
                                                                                span: 49..50,
                                                                                pos: 4:13..4:14,
                                                                                data: (),
                                                                            },
                                                                            value: Node {
                                                                                span: 51..65,
                                                                                pos: 4:15..4:29,
                                                                                data: String(
                                                                                    "Example Song",
                                                                                ),
                                                                            },
                                                                        },
                                                                    },
                                                                    Node {
                                                                        span: 74..92,
                                                                        pos: 5:9..5:27,
                                                                        data: Attribute {
                                                                            name: Node {
                                                                                span: 74..80,
                                                                                pos: 5:9..5:15,
                                                                                data: "author",
                                                                            },
                                                                            colon: Node {
                                                                                span: 80..81,
                                                                                pos: 5:15..5:16,
                                                                                data: (),
                                                                            },
                                                                            value: Node {
                                                                                span: 82..92,
                                                                                pos: 5:17..5:27,
                                                                                data: String(
                                                                                    "John Doe",
                                                                                ),
                                                                            },
                                                                        },
                                                                    },
                                                                    Node {
                                                                        span: 101..111,
                                                                        pos: 6:9..6:19,
                                                                        data: Attribute {
                                                                            name: Node {
                                                                                span: 101..105,
                                                                                pos: 6:9..6:13,
                                                                                data: "year",
                                                                            },
                                                                            colon: Node {
                                                                                span: 105..106,
                                                                                pos: 6:13..6:14,
                                                                                data: (),
                                                                            },
                                                                            value: Node {
                                                                                span: 107..111,
                                                                                pos: 6:15..6:19,
                                                                                data: Int(
                                                                                    2021,
                                                                                ),
                                                                            },
                                                                        },
                                                                    },
                                                                    Node {
                                                                        span: 120..152,
                                                                        pos: 7:9..7:41,
                                                                        data: Attribute {
                                                                            name: Node {
                                                                                span: 120..131,
                                                                                pos: 7:9..7:20,
                                                                                data: "description",
                                                                            },
                                                                            colon: Node {
                                                                                span: 131..132,
                                                                                pos: 7:20..7:21,
                                                                                data: (),
                                                                            },
                                                                            value: Node {
                                                                                span: 133..152,
                                                                                pos: 7:22..7:41,
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
                                                                    pos: 8:5..8:6,
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
                                        pos: 9:1..9:2,
                                        data: (),
                                    },
                                },
                            },
                        ],
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
                    pos: 1:1..1:7,
                    data: Unary {
                        operator: Node {
                            span: 0..1,
                            pos: 1:1..1:2,
                            data: Minus,
                        },
                        operand: Node {
                            span: 1..6,
                            pos: 1:2..1:7,
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
                    pos: 1:1..1:6,
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
                    pos: 1:1..1:9,
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
                    pos: 1:1..1:10,
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
fn parse_expr_float_lit() {
    check_expr(
        "-+1.337_424",
        expect![[r#"
            Ok(
                Node {
                    span: 0..11,
                    pos: 1:1..1:12,
                    data: Unary {
                        operator: Node {
                            span: 0..1,
                            pos: 1:1..1:2,
                            data: Minus,
                        },
                        operand: Node {
                            span: 1..11,
                            pos: 1:2..1:12,
                            data: Float(
                                F64N(
                                    1.337424,
                                ),
                            ),
                        },
                    },
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
                    pos: 1:1..1:9,
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
                    pos: 1:1..1:11,
                    data: Unary {
                        operator: Node {
                            span: 0..1,
                            pos: 1:1..1:2,
                            data: Minus,
                        },
                        operand: Node {
                            span: 1..10,
                            pos: 1:2..1:11,
                            data: Paren {
                                lparen: Node {
                                    span: 1..2,
                                    pos: 1:2..1:3,
                                    data: (),
                                },
                                expr: Node {
                                    span: 2..9,
                                    pos: 1:3..1:10,
                                    data: Unary {
                                        operator: Node {
                                            span: 2..3,
                                            pos: 1:3..1:4,
                                            data: Minus,
                                        },
                                        operand: Node {
                                            span: 3..9,
                                            pos: 1:4..1:10,
                                            data: Paren {
                                                lparen: Node {
                                                    span: 3..4,
                                                    pos: 1:4..1:5,
                                                    data: (),
                                                },
                                                expr: Node {
                                                    span: 4..8,
                                                    pos: 1:5..1:9,
                                                    data: Int(
                                                        1337,
                                                    ),
                                                },
                                                rparen: Node {
                                                    span: 8..9,
                                                    pos: 1:9..1:10,
                                                    data: (),
                                                },
                                            },
                                        },
                                    },
                                },
                                rparen: Node {
                                    span: 9..10,
                                    pos: 1:10..1:11,
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
                    pos: 1:1..1:62,
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
                    pos: 1:1..1:21,
                    data: Binary {
                        left: Node {
                            span: 0..5,
                            pos: 1:1..1:6,
                            data: Binary {
                                left: Node {
                                    span: 0..1,
                                    pos: 1:1..1:2,
                                    data: Int(
                                        2,
                                    ),
                                },
                                operator: Node {
                                    span: 2..3,
                                    pos: 1:3..1:4,
                                    data: Mult,
                                },
                                right: Node {
                                    span: 4..5,
                                    pos: 1:5..1:6,
                                    data: Int(
                                        4,
                                    ),
                                },
                            },
                        },
                        operator: Node {
                            span: 6..7,
                            pos: 1:7..1:8,
                            data: Add,
                        },
                        right: Node {
                            span: 8..20,
                            pos: 1:9..1:21,
                            data: Binary {
                                left: Node {
                                    span: 8..10,
                                    pos: 1:9..1:11,
                                    data: Int(
                                        17,
                                    ),
                                },
                                operator: Node {
                                    span: 11..12,
                                    pos: 1:12..1:13,
                                    data: Mult,
                                },
                                right: Node {
                                    span: 13..20,
                                    pos: 1:14..1:21,
                                    data: Paren {
                                        lparen: Node {
                                            span: 13..14,
                                            pos: 1:14..1:15,
                                            data: (),
                                        },
                                        expr: Node {
                                            span: 14..19,
                                            pos: 1:15..1:20,
                                            data: Binary {
                                                left: Node {
                                                    span: 14..15,
                                                    pos: 1:15..1:16,
                                                    data: Int(
                                                        1,
                                                    ),
                                                },
                                                operator: Node {
                                                    span: 16..17,
                                                    pos: 1:17..1:18,
                                                    data: Add,
                                                },
                                                right: Node {
                                                    span: 18..19,
                                                    pos: 1:19..1:20,
                                                    data: Int(
                                                        1,
                                                    ),
                                                },
                                            },
                                        },
                                        rparen: Node {
                                            span: 19..20,
                                            pos: 1:20..1:21,
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
                    pos: 1:1..1:55,
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
                    pos: 1:9..1:11,
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
                    pos: 1:1..1:33,
                    data: Binary {
                        left: Node {
                            span: 0..14,
                            pos: 1:1..1:15,
                            data: Binary {
                                left: Node {
                                    span: 0..4,
                                    pos: 1:1..1:5,
                                    data: Bool(
                                        true,
                                    ),
                                },
                                operator: Node {
                                    span: 5..8,
                                    pos: 1:6..1:9,
                                    data: And,
                                },
                                right: Node {
                                    span: 9..14,
                                    pos: 1:10..1:15,
                                    data: Bool(
                                        false,
                                    ),
                                },
                            },
                        },
                        operator: Node {
                            span: 15..17,
                            pos: 1:16..1:18,
                            data: Or,
                        },
                        right: Node {
                            span: 18..32,
                            pos: 1:19..1:33,
                            data: Binary {
                                left: Node {
                                    span: 18..23,
                                    pos: 1:19..1:24,
                                    data: Bool(
                                        false,
                                    ),
                                },
                                operator: Node {
                                    span: 24..27,
                                    pos: 1:25..1:28,
                                    data: And,
                                },
                                right: Node {
                                    span: 28..32,
                                    pos: 1:29..1:33,
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
                    pos: 1:1..1:12,
                    data: Accessor {
                        expr: Node {
                            span: 0..7,
                            pos: 1:1..1:8,
                            data: Accessor {
                                expr: Node {
                                    span: 0..3,
                                    pos: 1:1..1:4,
                                    data: Var(
                                        "foo",
                                    ),
                                },
                                dot: Node {
                                    span: 3..4,
                                    pos: 1:4..1:5,
                                    data: (),
                                },
                                attribute: Node {
                                    span: 4..7,
                                    pos: 1:5..1:8,
                                    data: "bar",
                                },
                            },
                        },
                        dot: Node {
                            span: 7..8,
                            pos: 1:8..1:9,
                            data: (),
                        },
                        attribute: Node {
                            span: 8..11,
                            pos: 1:9..1:12,
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
                    pos: 1:1..1:14,
                    data: Accessor {
                        expr: Node {
                            span: 0..9,
                            pos: 1:1..1:10,
                            data: Paren {
                                lparen: Node {
                                    span: 0..1,
                                    pos: 1:1..1:2,
                                    data: (),
                                },
                                expr: Node {
                                    span: 1..8,
                                    pos: 1:2..1:9,
                                    data: Accessor {
                                        expr: Node {
                                            span: 1..4,
                                            pos: 1:2..1:5,
                                            data: Var(
                                                "foo",
                                            ),
                                        },
                                        dot: Node {
                                            span: 4..5,
                                            pos: 1:5..1:6,
                                            data: (),
                                        },
                                        attribute: Node {
                                            span: 5..8,
                                            pos: 1:6..1:9,
                                            data: "bar",
                                        },
                                    },
                                },
                                rparen: Node {
                                    span: 8..9,
                                    pos: 1:9..1:10,
                                    data: (),
                                },
                            },
                        },
                        dot: Node {
                            span: 9..10,
                            pos: 1:10..1:11,
                            data: (),
                        },
                        attribute: Node {
                            span: 10..13,
                            pos: 1:11..1:14,
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
                    pos: 1:5..1:6,
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
                    pos: 1:1..1:39,
                    data: Binary {
                        left: Node {
                            span: 0..13,
                            pos: 1:1..1:14,
                            data: Call {
                                callee: Node {
                                    span: 0..5,
                                    pos: 1:1..1:6,
                                    data: Var(
                                        "atan2",
                                    ),
                                },
                                lparen: Node {
                                    span: 5..6,
                                    pos: 1:6..1:7,
                                    data: (),
                                },
                                arguments: [
                                    Node {
                                        span: 6..7,
                                        pos: 1:7..1:8,
                                        data: Int(
                                            1,
                                        ),
                                    },
                                    Node {
                                        span: 9..10,
                                        pos: 1:10..1:11,
                                        data: Int(
                                            2,
                                        ),
                                    },
                                ],
                                rparen: Node {
                                    span: 12..13,
                                    pos: 1:13..1:14,
                                    data: (),
                                },
                            },
                        },
                        operator: Node {
                            span: 14..15,
                            pos: 1:15..1:16,
                            data: Add,
                        },
                        right: Node {
                            span: 16..38,
                            pos: 1:17..1:39,
                            data: Unary {
                                operator: Node {
                                    span: 16..17,
                                    pos: 1:17..1:18,
                                    data: Minus,
                                },
                                operand: Node {
                                    span: 17..38,
                                    pos: 1:18..1:39,
                                    data: Call {
                                        callee: Node {
                                            span: 17..25,
                                            pos: 1:18..1:26,
                                            data: Accessor {
                                                expr: Node {
                                                    span: 17..20,
                                                    pos: 1:18..1:21,
                                                    data: Var(
                                                        "foo",
                                                    ),
                                                },
                                                dot: Node {
                                                    span: 20..21,
                                                    pos: 1:21..1:22,
                                                    data: (),
                                                },
                                                attribute: Node {
                                                    span: 21..25,
                                                    pos: 1:22..1:26,
                                                    data: "frob",
                                                },
                                            },
                                        },
                                        lparen: Node {
                                            span: 25..26,
                                            pos: 1:26..1:27,
                                            data: (),
                                        },
                                        arguments: [
                                            Node {
                                                span: 26..28,
                                                pos: 1:27..1:29,
                                                data: Int(
                                                    42,
                                                ),
                                            },
                                            Node {
                                                span: 30..37,
                                                pos: 1:31..1:38,
                                                data: Call {
                                                    callee: Node {
                                                        span: 30..33,
                                                        pos: 1:31..1:34,
                                                        data: Var(
                                                            "sin",
                                                        ),
                                                    },
                                                    lparen: Node {
                                                        span: 33..34,
                                                        pos: 1:34..1:35,
                                                        data: (),
                                                    },
                                                    arguments: [
                                                        Node {
                                                            span: 34..36,
                                                            pos: 1:35..1:37,
                                                            data: Var(
                                                                "pi",
                                                            ),
                                                        },
                                                    ],
                                                    rparen: Node {
                                                        span: 36..37,
                                                        pos: 1:37..1:38,
                                                        data: (),
                                                    },
                                                },
                                            },
                                        ],
                                        rparen: Node {
                                            span: 37..38,
                                            pos: 1:38..1:39,
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
fn parse_recover_from_errors() {
    check(
        r#"Song {
    bpm: 120
    sampleRate: 44_100
    meta: Meta {
        name: "Example Song"
        author: "John Doe"
        year: 2021
        description: "Simply.\nAwesome."
        awesome: true and not false
    }
    Track {
      name: "Lead" "bla"

      {}

      wtf: call(1, 2, 3 "fpp", 3, 4, 5 "Bla")

      Sequence {
        start: 8/4
      }
    }
    // Test for comments
    Track {
      name: "Drums"
    }
}"#,
        expect![[r#"
            Err(
                (
                    Node {
                        span: 0..427,
                        pos: 1:1..26:2,
                        data: Root {
                            objects: [
                                Node {
                                    span: 0..427,
                                    pos: 1:1..26:2,
                                    data: Object {
                                        name: Node {
                                            span: 0..4,
                                            pos: 1:1..1:5,
                                            data: "Song",
                                        },
                                        lbrace: Node {
                                            span: 5..6,
                                            pos: 1:6..1:7,
                                            data: (),
                                        },
                                        attrs: [
                                            Node {
                                                span: 11..19,
                                                pos: 2:5..2:13,
                                                data: Attribute {
                                                    name: Node {
                                                        span: 11..14,
                                                        pos: 2:5..2:8,
                                                        data: "bpm",
                                                    },
                                                    colon: Node {
                                                        span: 14..15,
                                                        pos: 2:8..2:9,
                                                        data: (),
                                                    },
                                                    value: Node {
                                                        span: 16..19,
                                                        pos: 2:10..2:13,
                                                        data: Int(
                                                            120,
                                                        ),
                                                    },
                                                },
                                            },
                                            Node {
                                                span: 24..42,
                                                pos: 3:5..3:23,
                                                data: Attribute {
                                                    name: Node {
                                                        span: 24..34,
                                                        pos: 3:5..3:15,
                                                        data: "sampleRate",
                                                    },
                                                    colon: Node {
                                                        span: 34..35,
                                                        pos: 3:15..3:16,
                                                        data: (),
                                                    },
                                                    value: Node {
                                                        span: 36..42,
                                                        pos: 3:17..3:23,
                                                        data: Int(
                                                            44100,
                                                        ),
                                                    },
                                                },
                                            },
                                            Node {
                                                span: 47..217,
                                                pos: 4:5..10:6,
                                                data: Attribute {
                                                    name: Node {
                                                        span: 47..51,
                                                        pos: 4:5..4:9,
                                                        data: "meta",
                                                    },
                                                    colon: Node {
                                                        span: 51..52,
                                                        pos: 4:9..4:10,
                                                        data: (),
                                                    },
                                                    value: Node {
                                                        span: 53..217,
                                                        pos: 4:11..10:6,
                                                        data: Object(
                                                            Node {
                                                                span: 53..217,
                                                                pos: 4:11..10:6,
                                                                data: Object {
                                                                    name: Node {
                                                                        span: 53..57,
                                                                        pos: 4:11..4:15,
                                                                        data: "Meta",
                                                                    },
                                                                    lbrace: Node {
                                                                        span: 58..59,
                                                                        pos: 4:16..4:17,
                                                                        data: (),
                                                                    },
                                                                    attrs: [
                                                                        Node {
                                                                            span: 68..88,
                                                                            pos: 5:9..5:29,
                                                                            data: Attribute {
                                                                                name: Node {
                                                                                    span: 68..72,
                                                                                    pos: 5:9..5:13,
                                                                                    data: "name",
                                                                                },
                                                                                colon: Node {
                                                                                    span: 72..73,
                                                                                    pos: 5:13..5:14,
                                                                                    data: (),
                                                                                },
                                                                                value: Node {
                                                                                    span: 74..88,
                                                                                    pos: 5:15..5:29,
                                                                                    data: String(
                                                                                        "Example Song",
                                                                                    ),
                                                                                },
                                                                            },
                                                                        },
                                                                        Node {
                                                                            span: 97..115,
                                                                            pos: 6:9..6:27,
                                                                            data: Attribute {
                                                                                name: Node {
                                                                                    span: 97..103,
                                                                                    pos: 6:9..6:15,
                                                                                    data: "author",
                                                                                },
                                                                                colon: Node {
                                                                                    span: 103..104,
                                                                                    pos: 6:15..6:16,
                                                                                    data: (),
                                                                                },
                                                                                value: Node {
                                                                                    span: 105..115,
                                                                                    pos: 6:17..6:27,
                                                                                    data: String(
                                                                                        "John Doe",
                                                                                    ),
                                                                                },
                                                                            },
                                                                        },
                                                                        Node {
                                                                            span: 124..134,
                                                                            pos: 7:9..7:19,
                                                                            data: Attribute {
                                                                                name: Node {
                                                                                    span: 124..128,
                                                                                    pos: 7:9..7:13,
                                                                                    data: "year",
                                                                                },
                                                                                colon: Node {
                                                                                    span: 128..129,
                                                                                    pos: 7:13..7:14,
                                                                                    data: (),
                                                                                },
                                                                                value: Node {
                                                                                    span: 130..134,
                                                                                    pos: 7:15..7:19,
                                                                                    data: Int(
                                                                                        2021,
                                                                                    ),
                                                                                },
                                                                            },
                                                                        },
                                                                        Node {
                                                                            span: 143..175,
                                                                            pos: 8:9..8:41,
                                                                            data: Attribute {
                                                                                name: Node {
                                                                                    span: 143..154,
                                                                                    pos: 8:9..8:20,
                                                                                    data: "description",
                                                                                },
                                                                                colon: Node {
                                                                                    span: 154..155,
                                                                                    pos: 8:20..8:21,
                                                                                    data: (),
                                                                                },
                                                                                value: Node {
                                                                                    span: 156..175,
                                                                                    pos: 8:22..8:41,
                                                                                    data: String(
                                                                                        "Simply.\nAwesome.",
                                                                                    ),
                                                                                },
                                                                            },
                                                                        },
                                                                        Node {
                                                                            span: 184..211,
                                                                            pos: 9:9..9:36,
                                                                            data: Attribute {
                                                                                name: Node {
                                                                                    span: 184..191,
                                                                                    pos: 9:9..9:16,
                                                                                    data: "awesome",
                                                                                },
                                                                                colon: Node {
                                                                                    span: 191..192,
                                                                                    pos: 9:16..9:17,
                                                                                    data: (),
                                                                                },
                                                                                value: Node {
                                                                                    span: 193..211,
                                                                                    pos: 9:18..9:36,
                                                                                    data: Binary {
                                                                                        left: Node {
                                                                                            span: 193..197,
                                                                                            pos: 9:18..9:22,
                                                                                            data: Bool(
                                                                                                true,
                                                                                            ),
                                                                                        },
                                                                                        operator: Node {
                                                                                            span: 198..201,
                                                                                            pos: 9:23..9:26,
                                                                                            data: And,
                                                                                        },
                                                                                        right: Node {
                                                                                            span: 202..211,
                                                                                            pos: 9:27..9:36,
                                                                                            data: Unary {
                                                                                                operator: Node {
                                                                                                    span: 202..205,
                                                                                                    pos: 9:27..9:30,
                                                                                                    data: Not,
                                                                                                },
                                                                                                operand: Node {
                                                                                                    span: 206..211,
                                                                                                    pos: 9:31..9:36,
                                                                                                    data: Bool(
                                                                                                        false,
                                                                                                    ),
                                                                                                },
                                                                                            },
                                                                                        },
                                                                                    },
                                                                                },
                                                                            },
                                                                        },
                                                                    ],
                                                                    children: [],
                                                                    rbrace: Node {
                                                                        span: 216..217,
                                                                        pos: 10:5..10:6,
                                                                        data: (),
                                                                    },
                                                                },
                                                            },
                                                        ),
                                                    },
                                                },
                                            },
                                        ],
                                        children: [
                                            Node {
                                                span: 222..362,
                                                pos: 11:5..21:6,
                                                data: Object {
                                                    name: Node {
                                                        span: 222..227,
                                                        pos: 11:5..11:10,
                                                        data: "Track",
                                                    },
                                                    lbrace: Node {
                                                        span: 228..229,
                                                        pos: 11:11..11:12,
                                                        data: (),
                                                    },
                                                    attrs: [
                                                        Node {
                                                            span: 236..248,
                                                            pos: 12:7..12:19,
                                                            data: Attribute {
                                                                name: Node {
                                                                    span: 236..240,
                                                                    pos: 12:7..12:11,
                                                                    data: "name",
                                                                },
                                                                colon: Node {
                                                                    span: 240..241,
                                                                    pos: 12:11..12:12,
                                                                    data: (),
                                                                },
                                                                value: Node {
                                                                    span: 242..248,
                                                                    pos: 12:13..12:19,
                                                                    data: String(
                                                                        "Lead",
                                                                    ),
                                                                },
                                                            },
                                                        },
                                                        Node {
                                                            span: 272..311,
                                                            pos: 16:7..16:46,
                                                            data: Attribute {
                                                                name: Node {
                                                                    span: 272..275,
                                                                    pos: 16:7..16:10,
                                                                    data: "wtf",
                                                                },
                                                                colon: Node {
                                                                    span: 275..276,
                                                                    pos: 16:10..16:11,
                                                                    data: (),
                                                                },
                                                                value: Node {
                                                                    span: 277..311,
                                                                    pos: 16:12..16:46,
                                                                    data: Call {
                                                                        callee: Node {
                                                                            span: 277..281,
                                                                            pos: 16:12..16:16,
                                                                            data: Var(
                                                                                "call",
                                                                            ),
                                                                        },
                                                                        lparen: Node {
                                                                            span: 281..282,
                                                                            pos: 16:16..16:17,
                                                                            data: (),
                                                                        },
                                                                        arguments: [
                                                                            Node {
                                                                                span: 282..283,
                                                                                pos: 16:17..16:18,
                                                                                data: Int(
                                                                                    1,
                                                                                ),
                                                                            },
                                                                            Node {
                                                                                span: 285..286,
                                                                                pos: 16:20..16:21,
                                                                                data: Int(
                                                                                    2,
                                                                                ),
                                                                            },
                                                                            Node {
                                                                                span: 288..289,
                                                                                pos: 16:23..16:24,
                                                                                data: Int(
                                                                                    3,
                                                                                ),
                                                                            },
                                                                            Node {
                                                                                span: 297..298,
                                                                                pos: 16:32..16:33,
                                                                                data: Int(
                                                                                    3,
                                                                                ),
                                                                            },
                                                                            Node {
                                                                                span: 300..301,
                                                                                pos: 16:35..16:36,
                                                                                data: Int(
                                                                                    4,
                                                                                ),
                                                                            },
                                                                            Node {
                                                                                span: 303..304,
                                                                                pos: 16:38..16:39,
                                                                                data: Int(
                                                                                    5,
                                                                                ),
                                                                            },
                                                                        ],
                                                                        rparen: Node {
                                                                            span: 310..311,
                                                                            pos: 16:45..16:46,
                                                                            data: (),
                                                                        },
                                                                    },
                                                                },
                                                            },
                                                        },
                                                    ],
                                                    children: [
                                                        Node {
                                                            span: 319..356,
                                                            pos: 18:7..20:8,
                                                            data: Object {
                                                                name: Node {
                                                                    span: 319..327,
                                                                    pos: 18:7..18:15,
                                                                    data: "Sequence",
                                                                },
                                                                lbrace: Node {
                                                                    span: 328..329,
                                                                    pos: 18:16..18:17,
                                                                    data: (),
                                                                },
                                                                attrs: [
                                                                    Node {
                                                                        span: 338..348,
                                                                        pos: 19:9..19:19,
                                                                        data: Attribute {
                                                                            name: Node {
                                                                                span: 338..343,
                                                                                pos: 19:9..19:14,
                                                                                data: "start",
                                                                            },
                                                                            colon: Node {
                                                                                span: 343..344,
                                                                                pos: 19:14..19:15,
                                                                                data: (),
                                                                            },
                                                                            value: Node {
                                                                                span: 345..348,
                                                                                pos: 19:16..19:19,
                                                                                data: Ratio(
                                                                                    Rational {
                                                                                        num: 2,
                                                                                        denom: 1,
                                                                                    },
                                                                                ),
                                                                            },
                                                                        },
                                                                    },
                                                                ],
                                                                children: [],
                                                                rbrace: Node {
                                                                    span: 355..356,
                                                                    pos: 20:7..20:8,
                                                                    data: (),
                                                                },
                                                            },
                                                        },
                                                    ],
                                                    rbrace: Node {
                                                        span: 361..362,
                                                        pos: 21:5..21:6,
                                                        data: (),
                                                    },
                                                },
                                            },
                                            Node {
                                                span: 392..425,
                                                pos: 23:5..25:6,
                                                data: Object {
                                                    name: Node {
                                                        span: 392..397,
                                                        pos: 23:5..23:10,
                                                        data: "Track",
                                                    },
                                                    lbrace: Node {
                                                        span: 398..399,
                                                        pos: 23:11..23:12,
                                                        data: (),
                                                    },
                                                    attrs: [
                                                        Node {
                                                            span: 406..419,
                                                            pos: 24:7..24:20,
                                                            data: Attribute {
                                                                name: Node {
                                                                    span: 406..410,
                                                                    pos: 24:7..24:11,
                                                                    data: "name",
                                                                },
                                                                colon: Node {
                                                                    span: 410..411,
                                                                    pos: 24:11..24:12,
                                                                    data: (),
                                                                },
                                                                value: Node {
                                                                    span: 412..419,
                                                                    pos: 24:13..24:20,
                                                                    data: String(
                                                                        "Drums",
                                                                    ),
                                                                },
                                                            },
                                                        },
                                                    ],
                                                    children: [],
                                                    rbrace: Node {
                                                        span: 424..425,
                                                        pos: 25:5..25:6,
                                                        data: (),
                                                    },
                                                },
                                            },
                                        ],
                                        rbrace: Node {
                                            span: 426..427,
                                            pos: 26:1..26:2,
                                            data: (),
                                        },
                                    },
                                },
                            ],
                        },
                    },
                    [
                        ParseError {
                            span: 249..254,
                            pos: 12:20..12:25,
                            message: "Expected one of [Ident, RBrace], but got LitString",
                        },
                        ParseError {
                            span: 262..263,
                            pos: 14:7..14:8,
                            message: "Expected one of [Ident, RBrace], but got LBrace",
                        },
                        ParseError {
                            span: 290..295,
                            pos: 16:25..16:30,
                            message: "Expected one of [RParen, Comma], but got LitString",
                        },
                        ParseError {
                            span: 305..310,
                            pos: 16:40..16:45,
                            message: "Expected one of [RParen, Comma], but got LitString",
                        },
                    ],
                ),
            )"#]],
    );
}

#[test]
fn parse_expr_sequence() {
    check_expr(
        r#"[[
            a4 a4 a4 a4
            r++
            a4+_.. a4--
          ]]"#,
        expect![[r#"
            Ok(
                Node {
                    span: 0..79,
                    pos: 1:1..5:13,
                    data: Sequence(
                        Node {
                            span: 0..79,
                            pos: 1:1..5:13,
                            data: Sequence {
                                llbracket: Node {
                                    span: 0..2,
                                    pos: 1:1..1:3,
                                    data: (),
                                },
                                symbols: [
                                    Node {
                                        span: 15..17,
                                        pos: 2:13..2:15,
                                        data: Note {
                                            note: Note(
                                                69,
                                            ),
                                            duration: Rational {
                                                num: 1,
                                                denom: 4,
                                            },
                                        },
                                    },
                                    Node {
                                        span: 18..20,
                                        pos: 2:16..2:18,
                                        data: Note {
                                            note: Note(
                                                69,
                                            ),
                                            duration: Rational {
                                                num: 1,
                                                denom: 4,
                                            },
                                        },
                                    },
                                    Node {
                                        span: 21..23,
                                        pos: 2:19..2:21,
                                        data: Note {
                                            note: Note(
                                                69,
                                            ),
                                            duration: Rational {
                                                num: 1,
                                                denom: 4,
                                            },
                                        },
                                    },
                                    Node {
                                        span: 24..26,
                                        pos: 2:22..2:24,
                                        data: Note {
                                            note: Note(
                                                69,
                                            ),
                                            duration: Rational {
                                                num: 1,
                                                denom: 4,
                                            },
                                        },
                                    },
                                    Node {
                                        span: 39..42,
                                        pos: 3:13..3:16,
                                        data: Rest {
                                            duration: Rational {
                                                num: 1,
                                                denom: 1,
                                            },
                                        },
                                    },
                                    Node {
                                        span: 55..61,
                                        pos: 4:13..4:19,
                                        data: Note {
                                            note: Note(
                                                69,
                                            ),
                                            duration: Rational {
                                                num: 15,
                                                denom: 16,
                                            },
                                        },
                                    },
                                    Node {
                                        span: 62..66,
                                        pos: 4:20..4:24,
                                        data: Note {
                                            note: Note(
                                                69,
                                            ),
                                            duration: Rational {
                                                num: 1,
                                                denom: 16,
                                            },
                                        },
                                    },
                                ],
                                rrbracket: Node {
                                    span: 77..79,
                                    pos: 5:11..5:13,
                                    data: (),
                                },
                            },
                        },
                    ),
                },
            )"#]],
    );
}

#[test]
fn parse_expr_sequence_nested_exprs() {
    check_expr(
        r#"[[chord(a4, "min") a4 chord(a4, "min") a4]]"#,
        expect![[r#"
            Ok(
                Node {
                    span: 0..43,
                    pos: 1:1..1:44,
                    data: Sequence(
                        Node {
                            span: 0..43,
                            pos: 1:1..1:44,
                            data: Sequence {
                                llbracket: Node {
                                    span: 0..2,
                                    pos: 1:1..1:3,
                                    data: (),
                                },
                                symbols: [
                                    Node {
                                        span: 2..18,
                                        pos: 1:3..1:19,
                                        data: Expr(
                                            Node {
                                                span: 2..18,
                                                pos: 1:3..1:19,
                                                data: Call {
                                                    callee: Node {
                                                        span: 2..7,
                                                        pos: 1:3..1:8,
                                                        data: Var(
                                                            "chord",
                                                        ),
                                                    },
                                                    lparen: Node {
                                                        span: 7..8,
                                                        pos: 1:8..1:9,
                                                        data: (),
                                                    },
                                                    arguments: [
                                                        Node {
                                                            span: 12..17,
                                                            pos: 1:13..1:18,
                                                            data: String(
                                                                "min",
                                                            ),
                                                        },
                                                    ],
                                                    rparen: Node {
                                                        span: 17..18,
                                                        pos: 1:18..1:19,
                                                        data: (),
                                                    },
                                                },
                                            },
                                        ),
                                    },
                                    Node {
                                        span: 19..21,
                                        pos: 1:20..1:22,
                                        data: Note {
                                            note: Note(
                                                69,
                                            ),
                                            duration: Rational {
                                                num: 1,
                                                denom: 4,
                                            },
                                        },
                                    },
                                    Node {
                                        span: 22..38,
                                        pos: 1:23..1:39,
                                        data: Expr(
                                            Node {
                                                span: 22..38,
                                                pos: 1:23..1:39,
                                                data: Call {
                                                    callee: Node {
                                                        span: 22..27,
                                                        pos: 1:23..1:28,
                                                        data: Var(
                                                            "chord",
                                                        ),
                                                    },
                                                    lparen: Node {
                                                        span: 27..28,
                                                        pos: 1:28..1:29,
                                                        data: (),
                                                    },
                                                    arguments: [
                                                        Node {
                                                            span: 32..37,
                                                            pos: 1:33..1:38,
                                                            data: String(
                                                                "min",
                                                            ),
                                                        },
                                                    ],
                                                    rparen: Node {
                                                        span: 37..38,
                                                        pos: 1:38..1:39,
                                                        data: (),
                                                    },
                                                },
                                            },
                                        ),
                                    },
                                    Node {
                                        span: 39..41,
                                        pos: 1:40..1:42,
                                        data: Note {
                                            note: Note(
                                                69,
                                            ),
                                            duration: Rational {
                                                num: 1,
                                                denom: 4,
                                            },
                                        },
                                    },
                                ],
                                rrbracket: Node {
                                    span: 41..43,
                                    pos: 1:42..1:44,
                                    data: (),
                                },
                            },
                        },
                    ),
                },
            )"#]],
    );
}
