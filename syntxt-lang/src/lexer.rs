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

use logos::Logos;

// Re-exports
pub use logos::Span;

#[derive(Logos, Debug, PartialEq, Clone, Copy)]
#[logos(subpattern decimal = r"[0-9][_0-9]*")]
pub enum Token {
    // Operators
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("not")]
    Not,
    #[token("and")]
    And,
    #[token("or")]
    Or,

    // Punctuation
    #[token(":")]
    Colon,
    #[token(";")]
    Semi,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,

    // Grouping
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,

    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,

    // Entities
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*")]
    Ident,

    // Literals
    #[regex(r#""([^"\\\n]|\\[^\u0000-\u001F])*""#)]
    LitString,
    #[regex(r"[+-]?(?&decimal)")]
    LitInt,
    #[regex(r"[+-]?(?&decimal)\.(?&decimal)")]
    LitFloat,
    #[regex(r"[+-]?(?&decimal)/(?&decimal)")]
    LitRatio,
    #[token("true")]
    #[token("false")]
    LitBool,

    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    #[regex(r"//[^\n]*", logos::skip)]
    Error,
}

#[cfg(test)]
mod tests {
    use super::Token;
    use expect_test::{expect, Expect};
    use logos::Logos;

    fn check(input: &str, output: Expect) {
        let lexer = Token::lexer(input);
        let tokens = lexer.spanned().collect::<Vec<_>>();
        let token_str = format!("{:?}", tokens);
        output.assert_eq(&token_str);
    }

    fn assert_lexable(input: &str) {
        let mut lexer = Token::lexer(input);
        while let Some(tok) = lexer.next() {
            if let Token::Error = tok {
                panic!("Could not lex {:?} at {:?}", lexer.slice(), lexer.span())
            }
        }
    }

    #[test]
    fn it_works() {
        assert_lexable(
            r#"
            Song {
                sampleRate: 44_100;
                meta: Meta {
                    author: "John Doe";
                    year: 2021;
                };

                tracks: [
                    Track {
                        filters: [
                            Lowpass {
                                cutoff: (0.5 * sin(time / 10) + 0.5) * 1000 + 1000;
                            }
                        ];
                        // more to come \o/
                    }
                ];
            }
        "#,
        )
    }

    #[test]
    fn empty_string() {
        check(r#""""#, expect![[r#"[(LitString, 0..2)]"#]]);
    }

    #[test]
    fn regular_string() {
        check(r#""hello world""#, expect![[r#"[(LitString, 0..13)]"#]]);
    }

    #[test]
    fn escaped_string() {
        check(
            r#""hello, so called \"world\"""#,
            expect![[r#"[(LitString, 0..28)]"#]],
        );
    }

    #[test]
    fn super_escaped_string() {
        check(
            r#""hello, so called \\\"world\\\" \w\p\:\t\n""#,
            expect![[r#"[(LitString, 0..43)]"#]],
        );
    }

    #[test]
    fn errorneous_multi_line_string() {
        check(
            r#""hello, so called\
\\\"world\\\"""#,
            expect![[
                r#"[(Error, 0..17), (Error, 17..18), (Error, 19..20), (Error, 20..21), (Error, 21..22), (LitString, 22..33)]"#
            ]],
        );
    }

    #[test]
    fn ints() {
        check("1337", expect![[r#"[(LitInt, 0..4)]"#]]);
        check("1_337", expect![[r#"[(LitInt, 0..5)]"#]]);
        check("-1_337", expect![[r#"[(LitInt, 0..6)]"#]]);
        check("+1_337", expect![[r#"[(LitInt, 0..6)]"#]]);
    }

    #[test]
    fn floats() {
        check("1.337", expect![[r#"[(LitFloat, 0..5)]"#]]);
        check("13.37", expect![[r#"[(LitFloat, 0..5)]"#]]);
        check("-1.337", expect![[r#"[(LitFloat, 0..6)]"#]]);
        check("+0.1337", expect![[r#"[(LitFloat, 0..7)]"#]]);
    }

    #[test]
    fn ratios() {
        check("1/337", expect![[r#"[(LitRatio, 0..5)]"#]]);
        check("13/3_7", expect![[r#"[(LitRatio, 0..6)]"#]]);
        check("-1/337", expect![[r#"[(LitRatio, 0..6)]"#]]);
        check("+0/1337", expect![[r#"[(LitRatio, 0..7)]"#]]);
    }

    #[test]
    fn bools() {
        check("true", expect![[r#"[(LitBool, 0..4)]"#]]);
        check("false", expect![[r#"[(LitBool, 0..5)]"#]]);
    }
}
