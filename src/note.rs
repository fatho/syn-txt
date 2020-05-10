// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! Definitions of what a note is.

/// A "note" is just an index on the synthesizers keyboard.
/// This definition follows the MIDI standard where C4 corresponds to index 60.
///
/// Note indices range from 0 to 127. At 12 semitones per octave,
/// this corresponds to a dynamic range of more then 10 octaves,
/// or a frequency ratio of about 1625 between the lowest and the
/// highest frequency.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Note(u8);

/// The name of a note in standard notation.
pub enum NoteName {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

/// Any offset applied to a note in standard notation.
pub enum NoteOffset {
    /// The note is a half-tone lower then indicated by its name.
    Flat,
    /// The note is left unchanged.
    Base,
    /// The note is a half-tone higher then indicated by its name.
    Sharp,
}

impl Note {
    /// Convert a note from standard notation to a MIDI note index.
    /// Note that different names may refer to the same note, e.g. a G♯ is the same as a A♭.
    /// Returns `None` if the note is not representable in the MIDI note system.
    ///
    /// # Examples
    ///
    /// ```
    /// use syn_txt::note::*;
    ///
    /// assert_eq!(Note::try_named(NoteName::A, NoteOffset::Base, 4), Some(Note::from_midi(69)));
    /// assert_eq!(Note::try_named(NoteName::C, NoteOffset::Sharp, 6), Some(Note::from_midi(85)));
    /// assert_eq!(Note::try_named(NoteName::G, NoteOffset::Flat, 2), Some(Note::from_midi(42)));
    /// ```
    pub fn try_named(name: NoteName, offset: NoteOffset, octave: i32) -> Option<Note> {
        let name_index = match name {
            NoteName::C => 0,
            NoteName::D => 2,
            NoteName::E => 4,
            NoteName::F => 5,
            NoteName::G => 7,
            NoteName::A => 9,
            NoteName::B => 11,
        };
        let offset_index = match offset {
            NoteOffset::Base => 0,
            NoteOffset::Flat => -1,
            NoteOffset::Sharp => 1,
        };
        // C4 is MIDI note number 60
        let normalize_index = 60 - 4 * 12;
        let note_index = octave * 12 + name_index + offset_index + normalize_index;
        if name_index >= 0 && name_index <= 127 {
            Some(Note(note_index as u8))
        } else {
            None
        }
    }

    /// Convert a note from standard notation to a MIDI note index.
    /// Note that different names may refer to the same note, e.g. a G♯ is the same as a A♭.
    ///
    /// # Panics
    ///
    /// - If the note is not representable in the MIDI note system.
    ///
    /// # Examples
    ///
    /// ```
    /// # use syn_txt::note::*;
    ///
    /// assert_eq!(Note::named(NoteName::A, NoteOffset::Base, 4), Note::from_midi(69));
    /// assert_eq!(Note::named(NoteName::C, NoteOffset::Sharp, 6), Note::from_midi(85));
    /// assert_eq!(Note::named(NoteName::G, NoteOffset::Flat, 2), Note::from_midi(42));
    /// ```
    pub fn named(name: NoteName, offset: NoteOffset, octave: i32) -> Note {
        Note::try_named(name, offset, octave).expect("Note not representable in MIDI system.")
    }

    /// Parse a name string of the format `<letter><offset><octave>`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use syn_txt::note::*;
    ///
    /// assert_eq!(Note::named_str("A4"), Some(Note::from_midi(69)));
    /// assert_eq!(Note::named_str("a4"), Some(Note::from_midi(69)));
    /// assert_eq!(Note::named_str("Csharp6"), Some(Note::from_midi(85)));
    /// assert_eq!(Note::named_str("C♯6"), Some(Note::from_midi(85)));
    /// assert_eq!(Note::named_str("Gb2"), Some(Note::from_midi(42)));
    /// ```
    pub fn named_str(name_str: &str) -> Option<Note> {
        let mut name_chars = name_str.chars();
        let name_ch = name_chars.next()?;
        let name = match name_ch.to_ascii_uppercase() {
            'A' => NoteName::A,
            'B' => NoteName::B,
            'C' => NoteName::C,
            'D' => NoteName::D,
            'E' => NoteName::E,
            'F' => NoteName::F,
            'G' => NoteName::G,
            _ => return None,
        };

        let offset_str = name_chars
            .as_str()
            .trim_end_matches(|ch: char| ch.is_ascii_digit());
        let offset = match offset_str {
            "sharp" | "♯" | "#" => NoteOffset::Sharp,
            "flat" | "♭" | "b" => NoteOffset::Flat,
            "" => NoteOffset::Base,
            _ => return None,
        };

        let octave_str = &name_chars.as_str()[offset_str.len()..];
        let octave = octave_str.parse().ok()?;
        Note::try_named(name, offset, octave)
    }

    pub fn from_midi(midi_note: u8) -> Note {
        assert!(midi_note < 128, "MIDI only has notes 0 - 127");
        Note(midi_note)
    }

    pub fn try_from_midi(midi_note: i64) -> Option<Note> {
        if midi_note >= 0 && midi_note < 128 {
            Some(Note(midi_note as u8))
        } else {
            None
        }
    }

    pub fn to_midi(self) -> u8 {
        self.0
    }

    /// Return the note index in a signed type, convenient for further calculations.
    pub fn index(self) -> i32 {
        self.0 as i32
    }
}

/// The velocity of a voice indicates how hard/fast the key was pressed down.
/// A normalized float between 0.0 and 1.0 inclusive.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Velocity(f64);

impl Eq for Velocity {}
impl Ord for Velocity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // we know that velocities can only be constructed as
        self.partial_cmp(other).unwrap()
    }
}

impl Velocity {
    pub const MAX: Velocity = Velocity(1.0);
    pub const MIN: Velocity = Velocity(0.0);

    pub fn as_f64(self) -> f64 {
        self.0
    }

    /// Convert a floating point value in the interval [0, 1] to a velocity.
    ///
    /// # Panics
    ///
    /// This function panics if `velocity` is not in the inclusive interval [0, 1].
    ///
    /// # Examples
    ///
    /// ```
    /// use syn_txt::note::*;
    ///
    /// assert_eq!(Velocity::from_f64(1.0), Velocity::MAX);
    /// ```
    pub fn from_f64(velocity: f64) -> Velocity {
        if let Some(v) = Self::try_from_f64(velocity) {
            v
        } else {
            panic!("{} out of range", velocity);
        }
    }

    pub fn try_from_f64(velocity: f64) -> Option<Velocity> {
        if velocity.is_nan() || velocity < 0.0 || velocity > 1.0 {
            return None
        }
        Some(Velocity(velocity))
    }
}
