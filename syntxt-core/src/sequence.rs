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

//! A musical sequence.

use crate::{note::Note, rational::Rational};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sequence {
    duration: Rational,
    items: Vec<SeqItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeqItem {
    Note {
        note: Note,
        duration: Rational,
    },
    Rest {
        duration: Rational,
    },
    /// Several sequences played at once.
    Stack {
        sequences: Vec<Sequence>,
    },
}

impl Sequence {
    pub fn new() -> Self {
        Self {
            duration: Rational::ZERO,
            items: Vec::new(),
        }
    }

    pub fn add(&mut self, item: SeqItem) {
        self.duration += match &item {
            SeqItem::Note { duration, .. } => *duration,
            SeqItem::Rest { duration } => *duration,
            SeqItem::Stack { sequences } => sequences
                .iter()
                .map(|seq| seq.duration)
                .max()
                .unwrap_or(Rational::ZERO),
        };
        self.items.push(item);
    }

    pub fn extend(&mut self, mut other: Sequence) {
        self.duration += other.duration;
        self.items.extend(other.items.drain(..));
    }

    pub fn duration(&self) -> Rational {
        self.duration
    }

    pub fn items(&self) -> &[SeqItem] {
        &self.items
    }
}
