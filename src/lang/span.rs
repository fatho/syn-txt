//! Bits and pieces for working with ranges of text.

use std::fmt::{self, Write};

/// A region with a text.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Span {
    /// The byte-offset of the first character of the span.
    pub begin: usize,
    /// The byte-offset of the first character *after* the span.
    pub end: usize,
}

/// Position inside a text in a form that's useful for human readers.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Pos {
    /// Line number, starting at 1
    pub line: usize,
    /// Position within the line, in characters, starting at 1
    pub column: usize,
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// A data structure for mapping byte offsets to line/column based positions.
pub struct LineMap<'a> {
    /// Ordered vector of the position of line breaks (`\n`)
    line_offsets: Vec<usize>,
    /// The original string, needed for obtaining the column indices.
    source: &'a str,
}

impl<'a> LineMap<'a> {
    pub fn new(s: &'a str) -> Self {
        Self {
            line_offsets: s
                .char_indices()
                .filter_map(|(pos, ch)| if ch == '\n' { Some(pos) } else { None })
                .collect(),
            source: s,
        }
    }

    /// # Examples
    ///
    /// ```
    /// # use syn_txt::lang::span::{LineMap,Pos};
    /// let s = "abc\ndefg\naäb\n";
    /// let m = LineMap::new(s);
    /// assert_eq!(m.offset_to_pos(0), Pos { line: 1, column: 1 });
    /// assert_eq!(m.offset_to_pos(3), Pos { line: 1, column: 4 });
    /// assert_eq!(m.offset_to_pos(4), Pos { line: 2, column: 1 });
    /// assert_eq!(m.offset_to_pos(10), Pos { line: 3, column: 2 });
    /// assert_eq!(m.offset_to_pos(12), Pos { line: 3, column: 3 });
    /// assert_eq!(m.offset_to_pos(13), Pos { line: 3, column: 4 });
    /// ```
    pub fn offset_to_pos(&self, offset: usize) -> Pos {
        match self.line_offsets.binary_search(&offset) {
            // We hit exactly the `line`th line-break.
            Ok(line) => {
                let previous_line_start = if line > 0 {
                    self.line_offsets[line - 1] + 1
                } else {
                    0
                };
                let column = self.source[previous_line_start..offset].chars().count() + 1;
                Pos {
                    line: line + 1,
                    column,
                }
            }
            Err(line) => {
                let previous_line_start = if line > 0 {
                    self.line_offsets[line - 1] + 1
                } else {
                    0
                };
                let column = self.source[previous_line_start..offset].chars().count() + 1;
                Pos {
                    line: line + 1,
                    column,
                }
            }
        }
    }

    pub fn pos_to_offset(&self, pos: Pos) -> usize {
        let line_begin = if pos.line <= 1 {
            0
        } else if pos.line - 1 >= self.line_offsets.len() {
            self.source.len()
        } else {
            self.line_offsets[pos.line - 2] + 1
        };
        let offset = self.source[line_begin..].char_indices().skip(pos.column - 1).next().map_or(0, |(pos, _)| pos);
        line_begin + offset
    }

    /// Return the extends of the given line (starting at 1)
    pub fn line_span(&self, line: usize) -> Span {
        let begin = if line <= 1 {
            0
        } else if line - 1 >= self.line_offsets.len() {
            self.source.len()
        } else {
            self.line_offsets[line - 2] + 1
        };

        let end = if line - 1 < self.line_offsets.len() {
            self.line_offsets[line - 1]
        } else {
            self.source.len()
        };
        Span { begin, end }
    }

    pub fn highlight(&self, start: Pos, end: Pos, colored: bool) -> String {
        let mut out = String::new();
        let display_start = 1.max(start.line - 1);
        let display_end = self.line_offsets.len().min(start.line + 1);
        for line in display_start..=display_end {
            let line_span = self.line_span(line);
            let line_str = &self.source[line_span.begin..line_span.end];

            let red = "\x1b[31;1m";
            let reset = "\x1b[0m";

            if colored && (line == start.line) {
                out.push_str(red);
            }
            write!(&mut out, "{:4}|", line).unwrap();
            if colored && (line == start.line) {
                out.push_str(reset);
            }

            if colored {
                for (i, ch) in line_str.chars().enumerate() {
                    if i + 1 == start.column && line == start.line {
                        out.push_str(red);
                    }
                    if i + 1 == end.column && line == end.line {
                        out.push_str(reset);
                    }
                    out.push(ch);
                }
            } else {
                out.push_str(line_str);
            }
            out.push('\n');

            if line >= start.line && line <= end.line {
                let col_start = if line == start.line { start.column } else { 1 };
                let col_end = if line == end.line {
                    end.column
                } else {
                    line_str.chars().count() + 1
                };
                out.push_str("     ");
                for _ in 1..col_start {
                    out.push(' ');
                }
                for _ in col_start..col_end {
                    out.push('^');
                }
                out.push('\n');
            }
        }
        out
    }
}