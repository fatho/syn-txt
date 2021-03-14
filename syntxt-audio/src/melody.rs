// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! A simple textual format for writing melodies while testing things.

use crate::note::{Accidental, Note, NoteName, Velocity};
use crate::song::{PlayedNote, Time};

/// Parse a melody described in a textual format.
pub fn parse_melody(input: &str) -> Result<Vec<PlayedNote>, ParseError> {
    let mut p = Parser::new(input);
    p.parse_root()
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoteSym {
    /// Which key was pressed
    pub note: Note,
    /// How long the note is held
    pub duration: Time,
}

pub enum Sym {
    Note(NoteSym),
    Rest(Time),
    GroupStart,
    GroupEnd,
}

struct Parser<'a> {
    stream: Scan<'a>,
}

#[derive(Debug)]
pub enum ParseError {
    EOF,
    NoNote(char),
    /// A note could not be represented using MIDI encoding.
    UnrepresentableNote,
    Other(&'static str),
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        let mut stream = Scan::new(input);
        stream.skip_whitespace();
        Self { stream }
    }

    pub fn is_eof(&mut self) -> bool {
        self.stream.is_eof()
    }

    /// Parse the whole string as a melody, failing if not everything was consumed.
    pub fn parse_root(&mut self) -> Result<Vec<PlayedNote>, ParseError> {
        let mut notes = Vec::new();
        self.parse_sequential(true, Time::zero(), &mut notes)?;
        Ok(notes)
    }

    /// Parse the notes and nested groups inside the current group as a sequence.
    pub fn parse_sequential(
        &mut self,
        top_level: bool,
        start: Time,
        notes: &mut Vec<PlayedNote>,
    ) -> Result<Time, ParseError> {
        let mut time = start;
        // When parsing the top-level sequence, parse until EOF, otherwise, parse until group end symbol
        while !top_level || !self.is_eof() {
            match self.parse_sym()? {
                Sym::Rest(duration) => time += duration,
                Sym::Note(sym) => {
                    notes.push(PlayedNote {
                        note: sym.note,
                        duration: sym.duration,
                        start: time,
                        velocity: Velocity::from_f64(0.5),
                    });
                    time += sym.duration;
                }
                Sym::GroupStart => {
                    // A group inside a sequence becomes a stack
                    time = self.parse_stack(time, notes)?;
                }
                Sym::GroupEnd => {
                    if top_level {
                        return Err(ParseError::Other("Unmatched group end symbol"));
                    } else {
                        break;
                    }
                }
            }
        }
        Ok(time)
    }

    /// Parse the notes and nested groups inside the current group as a stack (playing all at the same time).
    /// Returns the new time after the stack. The duration of the stack is determined by the last note in the stack.
    ///
    /// TODO: how to reduce clipping of stacks? automatically adjust velocity based on stack size? easier annotation of velocity?
    pub fn parse_stack(
        &mut self,
        start: Time,
        notes: &mut Vec<PlayedNote>,
    ) -> Result<Time, ParseError> {
        let mut time = start;
        loop {
            // Will fail on EOF because each nested sequence must be terminated by a group end,
            // which exits the loop.
            match self.parse_sym()? {
                Sym::Rest(duration) => time = time.max(start + duration),
                Sym::Note(sym) => {
                    notes.push(PlayedNote {
                        note: sym.note,
                        duration: sym.duration,
                        start,
                        velocity: Velocity::from_f64(0.5),
                    });
                    time = time.max(start + sym.duration);
                }
                Sym::GroupStart => {
                    // A group inside a stack becomes a sequence
                    let new_time = self.parse_sequential(false, time, notes)?;
                    time = time.max(new_time);
                }
                Sym::GroupEnd => break,
            }
        }
        Ok(time)
    }

    pub fn parse_sym(&mut self) -> Result<Sym, ParseError> {
        match self.peek_char()? {
            'r' | 'R' => {
                self.expect_char()?;
                let duration = self.parse_duration()?;
                self.stream.skip_whitespace();
                Ok(Sym::Rest(duration))
            }
            'a'..='g' | 'A'..='G' => self.parse_note_sym().map(Sym::Note),
            '{' => {
                self.expect_char()?;
                self.stream.skip_whitespace();
                Ok(Sym::GroupStart)
            }
            '}' => {
                self.expect_char()?;
                self.stream.skip_whitespace();
                Ok(Sym::GroupEnd)
            }
            _ => Err(ParseError::Other("Unknown symbol")),
        }
    }

    pub fn parse_note_sym(&mut self) -> Result<NoteSym, ParseError> {
        let note = self.parse_note()?;
        let duration = self.parse_duration()?;
        self.stream.skip_whitespace();

        Ok(NoteSym { note, duration })
    }

    fn parse_note(&mut self) -> Result<Note, ParseError> {
        // First comes the name
        let name = match self.expect_char()? {
            'a' | 'A' => NoteName::A,
            'b' | 'B' => NoteName::B,
            'c' | 'C' => NoteName::C,
            'd' | 'D' => NoteName::D,
            'e' | 'E' => NoteName::E,
            'f' | 'F' => NoteName::F,
            'g' | 'G' => NoteName::G,
            ch => return Err(ParseError::NoNote(ch)),
        };
        // Then any accidental
        let accidental = match self.peek_char_optional() {
            Some(ch) => {
                if ch == '♯' || ch == '#' {
                    self.stream.advance();
                    Accidental::Sharp
                } else if ch == '♭' || ch == 'b' {
                    self.stream.advance();
                    Accidental::Flat
                } else {
                    Accidental::Base
                }
            }
            _ => Accidental::Base,
        };
        // Then the octave
        let octave = match self.stream.current() {
            Some((_, ch)) if ch.is_ascii_digit() => {
                self.stream.advance();
                ch.to_digit(10).unwrap() as i32
            }
            _ => 4,
        };
        Note::try_named(name, accidental, octave).ok_or(ParseError::UnrepresentableNote)
    }

    fn parse_duration(&mut self) -> Result<Time, ParseError> {
        // Then comes the duration,
        // first in powers of two
        let mut power: i64 = -2; // quarters
        loop {
            match self.stream.current() {
                Some((_, '+')) => {
                    self.stream.advance();
                    power += 1;
                }
                Some((_, '-')) => {
                    self.stream.advance();
                    power -= 1;
                }
                _ => break,
            }
        }
        // then the dots
        let mut dots = 0;
        while let Some('.') = self.peek_char_optional() {
            self.expect_char()?;
            dots += 1;
        }
        // Then put everything together
        let mut duration = Time::int(2).powi(power);
        for i in 0..dots {
            // each dot is worth half of the previous note duration
            duration += Time::int(2).powi(power - i - 1);
        }
        Ok(duration)
    }

    fn expect_char(&mut self) -> Result<char, ParseError> {
        if let Some((_, ch)) = self.stream.next() {
            Ok(ch)
        } else {
            Err(ParseError::EOF)
        }
    }

    fn peek_char(&mut self) -> Result<char, ParseError> {
        if let Some((_, ch)) = self.stream.current() {
            Ok(ch)
        } else {
            Err(ParseError::EOF)
        }
    }

    fn peek_char_optional(&mut self) -> Option<char> {
        if let Some((_, ch)) = self.stream.current() {
            Some(ch)
        } else {
            None
        }
    }
}

struct Scan<'a> {
    stream: std::iter::Peekable<std::str::CharIndices<'a>>,
    position: usize,
}

impl<'a> Scan<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            stream: input.char_indices().peekable(),
            position: 0,
        }
    }

    pub fn is_eof(&mut self) -> bool {
        self.current().is_none()
    }

    pub fn current(&mut self) -> Option<(usize, char)> {
        self.stream.peek().cloned()
    }

    pub fn next(&mut self) -> Option<(usize, char)> {
        let current = self.stream.next();
        if let Some((pos, _)) = current {
            self.position = pos;
        }
        current
    }

    pub fn advance(&mut self) {
        self.stream.next();
    }

    pub fn skip_whitespace(&mut self) {
        while let Some((_, ch)) = self.current() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parser() {
        let mel = parse_melody(
            r"
            c-d-e-f- g g
            a-a-a-a- g+
            a-a-a-a- g+
            f-f-f-f- e e
            d-d-d-d- c+",
        )
        .unwrap();
        assert_eq!(mel.len(), 27);
    }

    #[test]
    fn parse_duration() {
        let mut p = Parser::new("--+-++...");
        assert_eq!(p.parse_duration().unwrap(), Time::new(15, 32));
    }

    #[test]
    fn note_parser() {
        let mut p = Parser::new("a b a++ a- a-. a#--");
        let a = Note::try_named(NoteName::A, Accidental::Base, 4).unwrap();
        let b = Note::try_named(NoteName::B, Accidental::Base, 4).unwrap();
        let a_sharp = Note::try_named(NoteName::A, Accidental::Sharp, 4).unwrap();
        assert_eq!(
            p.parse_note_sym().unwrap(),
            NoteSym {
                note: a,
                duration: Time::new(1, 4)
            }
        );
        assert_eq!(
            p.parse_note_sym().unwrap(),
            NoteSym {
                note: b,
                duration: Time::new(1, 4)
            }
        );
        assert_eq!(
            p.parse_note_sym().unwrap(),
            NoteSym {
                note: a,
                duration: Time::new(1, 1)
            }
        );
        assert_eq!(
            p.parse_note_sym().unwrap(),
            NoteSym {
                note: a,
                duration: Time::new(1, 8)
            }
        );
        assert_eq!(
            p.parse_note_sym().unwrap(),
            NoteSym {
                note: a,
                duration: Time::new(3, 16)
            }
        );
        assert_eq!(
            p.parse_note_sym().unwrap(),
            NoteSym {
                note: a_sharp,
                duration: Time::new(1, 16)
            }
        );
    }
}
