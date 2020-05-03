use crate::note::{Note, Velocity};
use crate::rational::Rational;

/// Time in measures, can be fractional, e.g. a note taking 1/4.
/// The time is relative until the music is put into a song with a specific measure.
pub type Time = Rational;

/// A piano roll is a set of notes that are played in a specific arrangement.
#[derive(Debug, Clone, PartialEq)]
pub struct PianoRoll {
    /// Nominal length of the piano roll. This determines at what time new notes are appended.
    length: Time,
    /// The notes on this piano roll in the order of their start times.
    notes: Vec<PlayedNote>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayedNote {
    /// Which key was pressed
    pub note: Note,
    /// How hard the key was pressed
    pub velocity: Velocity,
    /// Time when the key was pressed
    pub start: Time,
    /// Time when the key was released
    pub duration: Time,
}

impl PianoRoll {
    pub fn new() -> Self {
        PianoRoll {
            length: Time::zero(),
            notes: Vec::new(),
        }
    }

    fn last_note_start_time(&self) -> Time {
        self.notes.last().map_or(Time::zero(), |note| note.start)
    }

    /// Add a note at the same time as the last played note started.
    pub fn add_stack(&mut self, note: Note, duration: Time, velocity: Velocity) {
        let start = self.last_note_start_time();
        self.notes.push(PlayedNote {
            note,
            velocity,
            start,
            duration,
        });
        self.length = self.length.max(start + duration)
    }

    /// Add a note at the same time as the last played note started.
    /// The offset must be positive.
    pub fn add_stack_offset(
        &mut self,
        note: Note,
        duration: Time,
        velocity: Velocity,
        offset: Time,
    ) {
        // TODO: allow any offset here
        assert!(offset >= Rational::zero());
        let start = self.last_note_start_time();
        self.notes.push(PlayedNote {
            note,
            velocity,
            start: start + offset,
            duration,
        });
        self.length = self.length.max(start + offset + duration)
    }

    /// Add a note at the end of the piano roll, extending its length.
    pub fn add_after(&mut self, note: Note, duration: Time, velocity: Velocity) {
        self.notes.push(PlayedNote {
            note,
            velocity,
            start: self.length,
            duration,
        });
        self.length += duration;
    }

    /// Extend the length of the piano roll without playing notes.
    pub fn rest(&mut self, duration: Time) {
        self.length += duration;
    }

    /// Append the contents of another piano roll at the end of this one.
    pub fn append(&mut self, other: &PianoRoll) {
        for note in other.notes.iter() {
            let mut new_note: PlayedNote = note.clone();
            new_note.start += self.length;
            self.notes.push(new_note);
        }
        self.length += other.length;
    }

    /// Iterate all notes on this piano roll in the order they are played.
    pub fn iter(&self) -> impl Iterator<Item = &PlayedNote> {
        self.notes.iter()
    }

    pub fn length(&self) -> Rational {
        self.length
    }
}

impl Default for PianoRoll {
    fn default() -> Self {
        Self::new()
    }
}
