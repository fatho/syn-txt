/// An Attack-Decay-Sustain-Release envelope.
/// When a key is pressed, the amplitude first rises from zero to one over `attack` seconds,
/// then decays over an additional `decay` seconds to the `sustain` level where it is held
/// as long as the key is pressed. When the key is released, the volume falls back to zero
/// over the next `release` seconds.
///
/// # Example
///
/// ```
/// use syn_txt::synth::envelope::*;
/// let e = ADSR {
///     attack: 0.25,
///     decay: 0.5,
///     sustain: 0.75,
///     release: 1.0,
/// };
/// assert_eq!(e.eval(EnvelopeTime::SincePress(0.0)), 0.0);
/// assert_eq!(e.eval(EnvelopeTime::SincePress(0.25)), 1.0);
/// assert_eq!(e.eval(EnvelopeTime::SincePress(0.5)), 0.875);
/// assert_eq!(e.eval(EnvelopeTime::SincePress(0.75)), 0.75);
/// assert_eq!(e.eval(EnvelopeTime::SincePress(20.0)), 0.75);
///
/// assert_eq!(e.eval(EnvelopeTime::SinceRelease(0.0)), 0.75);
/// assert_eq!(e.eval(EnvelopeTime::SinceRelease(0.5)), 0.375);
/// assert_eq!(e.eval(EnvelopeTime::SinceRelease(1.0)), 0.0);
/// ```
#[derive(Debug, Clone)]
pub struct ADSR {
    /// Time in seconds to go from 0.0 to 1.0
    pub attack: f64,
    /// Time in seconds to go from 1.0 to `sustain`.
    pub decay: f64,
    /// Constant amplitude while key is held.
    pub sustain: f64,
    /// Time in seconds to go from `sustain` to 0.0.
    pub release: f64,
}


#[derive(Debug, Clone, Copy)]
pub enum EnvelopeTime {
    /// Time elapsed since the key was pressed but not yet released.
    SincePress(f64),
    /// Time elapsed since the key was released.
    SinceRelease(f64)
}

impl EnvelopeTime {
    pub fn press() -> Self {
        EnvelopeTime::SincePress(0.0)
    }

    pub fn release() -> Self {
        EnvelopeTime::SinceRelease(0.0)
    }

    pub fn advance(&mut self, dt: f64) {
        match self {
            EnvelopeTime::SincePress(t) => *t += dt,
            EnvelopeTime::SinceRelease(t) => *t += dt,
        }
    }
}


impl ADSR {
    /// Evaluate the envelope curve at this point in time.
    pub fn eval(&self, time: EnvelopeTime) -> f64 {
        match time {
            EnvelopeTime::SincePress(t) =>
                if t < self.attack {
                    t / self.attack
                } else if t < self.attack + self.decay {
                    self.sustain + (1.0 - self.sustain) * (1.0 - (t - self.attack) / self.decay)
                } else {
                    self.sustain
                }
            EnvelopeTime::SinceRelease(t) =>
                if t < self.release {
                    self.sustain * (1.0 - t/ self.release)
                } else {
                    0.0
                }
        }
    }
}
