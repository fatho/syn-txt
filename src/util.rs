//! Utility functions that I don't know where to put else

/// Compute a factor measured in cents (1/100 of a semitone)
///
/// # Example
///
/// ```
/// # use syn_txt::util::*;
///
/// assert_eq!(from_cents(-16.0), from_semitones(-16.0 / 100.0));
/// ```
pub fn from_cents(cents: f64) -> f64 {
    2.0f64.powf(cents / 1200.0)
}

/// Compute a factor measured in octaves (one octave corresponds to a factor of two).
///
/// # Example
///
/// ```
/// # use syn_txt::util::*;
///
/// assert_eq!(from_octaves(3.0), 8.0);
/// assert_eq!(from_octaves(-1.0), 0.5);
/// ```
pub fn from_octaves(octaves: f64) -> f64 {
    2.0f64.powf(octaves)
}

/// Compute a factor measured in semitones (one octave consists of 12 semitones)
///
/// # Example
///
/// ```
/// # use syn_txt::util::*;
///
/// assert_eq!(from_semitones(3.0), from_octaves(3.0 / 12.0));
/// ```
pub fn from_semitones(semitones: f64) -> f64 {
    2.0f64.powf(semitones / 12.0)
}
