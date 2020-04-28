//! A synthesizer turns events into sounds.

use crate::note::NoteAction;

/// An event influencing the behavior of a synthesizer.
/// Currently, only note press and release events are supported.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Event {
    /// Time of the event in samples since playback started.
    pub time: usize,
    /// What kind of event happened at this time.
    pub action: NoteAction,
}
