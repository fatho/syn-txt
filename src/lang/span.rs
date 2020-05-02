//! Bits and pieces for working with ranges of text.

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
}
