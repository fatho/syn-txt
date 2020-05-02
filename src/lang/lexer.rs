#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Span<T> {
    pub value: T,
    /// The byte-offset of the first character of the span.
    pub begin: usize,
    /// The byte-offset of the first character *after* the span.
    pub end: usize,
}

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

// TODO: create proper error type for lexer

pub struct Lexer<'a> {
    input: &'a str,
    stream: std::str::CharIndices<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let stream = input.char_indices();
        Self {
            input,
            stream,
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

    /// Indentify the next token
    pub fn next_token(&mut self) -> Option<Span<Token>> {
        macro_rules! token_char {
            ($pos:expr, $ch:expr, $tok:expr) => {
                return Some(Span { value: $tok, begin: $pos, end: $pos + $ch.len_utf8() })
            }
        }

        while let Some((pos, ch)) = self.next_char() {
            match ch {
                // Single-character tokens
                '(' => token_char!(pos, ch, Token::ParenOpen),
                ')' => token_char!(pos, ch, Token::ParenClose),

                // Multi-character tokens

                // Identifiers
                _ if charsets::is_ident_start(ch) => return Some(self.lex_ident(pos)),

                // Strings
                '"' => return Some(self.lex_string(pos)),

                // Numbers
                _ if ch.is_ascii_digit() || ch == '+' || ch == '-' => return Some(self.lex_number(pos)),

                // Line comments
                ';' => self.skip_to_next_line(),

                // Ignore whitespace between tokens
                _ if ch.is_whitespace() => continue,
                _ => token_char!(pos, ch, Token::ErrUnrecognized),
            }
        }
        // If we exhausted the stream without producing a token, we're done
        None
    }

    fn lex_string(&mut self, start: usize) -> Span<Token> {
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

        Span {
            value: tok,
            begin: start,
            end: self.current_offset(),
        }
    }


    fn lex_while<P: Fn(char) -> bool>(&mut self, start: usize, token: Token, predicate: P) -> Span<Token> {
        while let Some((_, ch)) = self.peek_char() {
            if predicate(ch) {
                self.next_char();
            } else {
                break;
            }
        }
        Span {
            value: token,
            begin: start,
            end: self.current_offset(),
        }
    }

    fn lex_ident(&mut self, start: usize) -> Span<Token> {
        self.lex_while(start, Token::Ident, charsets::is_ident_cont)
    }

    fn lex_number(&mut self, start: usize) -> Span<Token> {
        while let Some((_, ch)) = self.peek_char() {
            if ch.is_ascii_digit() {
                self.next_char();
            } else if ch == '/' {
                // Only consume the '/' if followed by a digit again
                if let Some((_, after)) = self.peek_char_skip(1) {
                    if after.is_ascii_digit() {
                        self.next_char();
                        return self.lex_rational_denominator(start);
                    }
                }
                break;
            } else if ch == '.' {
                self.next_char();
                return self.lex_float_fractional(start);
            } else {
                break;
            }
        }
        // If we made it out, it's an int
        Span {
            value: Token::Int,
            begin: start,
            end: self.current_offset(),
        }
    }

    fn lex_rational_denominator(&mut self, start: usize) -> Span<Token> {
        while let Some((_, ch)) = self.peek_char() {
            if ch.is_ascii_digit() {
                self.next_char();
            } else {
                break;
            }
        }
        Span {
            value: Token::Rational,
            begin: start,
            end: self.current_offset(),
        }
    }

    /// Lex the part of a float after the dot.
    /// `start` refers to the start of the number itself,
    /// not to the start of the fractional part.
    /// TODO: add support for scientific notation.
    fn lex_float_fractional(&mut self, start: usize) -> Span<Token> {
        while let Some((_, ch)) = self.peek_char() {
            if ch.is_ascii_digit() {
                self.next_char();
            } else {
                break;
            }
        }
        Span {
            value: Token::Float,
            begin: start,
            end: self.current_offset(),
        }
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
