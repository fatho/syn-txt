//! Definitions of what a note is.

/// A "note" is just an index on the synthesizers keyboard.
/// This definition follows the MIDI standard where C4 corresponds to index 60.
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
    G
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
    ///
    /// # Examples
    ///
    /// ```
    /// use syn_txt::note::*;
    ///
    /// assert_eq!(Note::named(NoteName::A, NoteOffset::Base, 4), Some(Note::from_midi(69)));
    /// assert_eq!(Note::named(NoteName::A, NoteOffset::Base, 4), Some(Note::from_midi(69)));
    /// assert_eq!(Note::named(NoteName::C, NoteOffset::Sharp, 6), Some(Note::from_midi(85)));
    /// assert_eq!(Note::named(NoteName::G, NoteOffset::Flat, 2), Some(Note::from_midi(42)));
    /// ```
    pub fn named(name: NoteName, offset: NoteOffset, octave: i32) -> Option<Note> {
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
        if name_index >= std::u8::MIN as i32 && name_index < std::u8::MAX as i32 {
            Some(Note(note_index as u8))
        } else {
            None
        }
    }

    pub fn from_midi(midi_note: u8) -> Note {
        Note(midi_note)
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
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Velocity(pub u8);

impl Velocity {
    pub fn amplitude(self) -> f64 {
        self.0 as f64 / std::u8::MAX as f64
    }
}

/// Notes can be pressed down, and released.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NoteAction {
    Play { note: Note, velocity: Velocity },
    Release { note: Note },
}
