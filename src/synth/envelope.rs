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
/// let mut eval = e.instantiate(4.0); // 4 samples per second
/// assert_eq!(eval.step(), 0.0);
/// assert_eq!(eval.step(), 1.0);
/// assert_eq!(eval.step(), 0.875);
/// assert_eq!(eval.step(), 0.75);
/// assert_eq!(eval.step(), 0.75);
/// assert_eq!(eval.step(), 0.75);
/// assert_eq!(eval.step(), 0.75);
/// eval.release();
/// assert!(! eval.faded());
///
/// assert_eq!(eval.step(), 0.75);
/// assert_eq!(eval.step(), 0.5625);
/// assert_eq!(eval.step(), 0.375);
/// assert_eq!(eval.step(), 0.1875);
/// assert_eq!(eval.step(), 0.0);
/// assert!(eval.faded());
///
/// // This time with an early release
/// let mut eval = e.instantiate(4.0); // 4 samples per second
/// assert_eq!(eval.step(), 0.0);
/// eval.release();
/// assert_eq!(eval.step(), 1.0);
/// assert_eq!(eval.step(), 0.75);
/// assert_eq!(eval.step(), 0.5);
/// assert_eq!(eval.step(), 0.25);
/// assert_eq!(eval.step(), 0.0);
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

impl ADSR {
    pub fn instantiate(&self, sample_rate: f64) -> EvalADSR {
        // TODO: what happens if result is not representable as usize?
        EvalADSR {
            attack_samples: (self.attack * sample_rate).round() as usize,
            decay_samples: (self.decay * sample_rate).round() as usize,
            release_samples: (self.release * sample_rate).round() as usize,
            sustain_level: self.sustain,
            release_level: self.sustain,
            current_sample: 0,
            released: false,
        }
    }
}

pub struct EvalADSR {
    attack_samples: usize,
    decay_samples: usize,
    release_samples: usize,
    sustain_level: f64,
    current_sample: usize,
    release_level: f64,
    released: bool,
}

impl EvalADSR {
    /// Called for every sample, returning the envelope gain at that sample.
    pub fn step(&mut self) -> f64 {
        let gain = self.compute_gain();
        if self.released {
            if self.current_sample < self.attack_samples + self.decay_samples + self.release_samples {
                self.current_sample += 1;
            }
        } else {
            if self.current_sample < self.attack_samples + self.decay_samples {
                self.current_sample += 1;
            }
        }
        gain
    }

    fn compute_gain(&self) -> f64 {
        if self.current_sample < self.attack_samples {
            // Rise from 0.0 to 1.0
            let volume = self.current_sample as f64 / self.attack_samples as f64;
            volume
        } else if self.current_sample < self.attack_samples + self.decay_samples {
            // Drop from 1.0 to `sustain_level`
            let progress = (self.current_sample - self.attack_samples) as f64 / self.decay_samples as f64;
            let volume = 1.0 - progress * (1.0 - self.sustain_level);
            volume
        } else if !self.released && self.current_sample == self.attack_samples + self.decay_samples {
            // Hold at `sustain_level` while not released
            self.sustain_level
        } else if self.current_sample < self.attack_samples + self.decay_samples + self.release_samples {
            // Drop from `sustain_level` to 0.0
            let progress = (self.current_sample - self.attack_samples - self.decay_samples) as f64 / self.release_samples as f64;
            let volume = (1.0 - progress) * self.release_level;
            volume
        } else {
            0.0
        }
    }

    pub fn released(&self) -> bool {
        self.released
    }

    /// Called when the note is released.
    pub fn release(&mut self) {
        if ! self.released {
            self.release_level = self.compute_gain();
            self.current_sample = self.attack_samples + self.decay_samples;
            self.released = true;
        }
    }

    /// When the note has been released and the envelope reached zero volume.
    pub fn faded(&self) -> bool {
        self.current_sample == self.attack_samples + self.decay_samples + self.release_samples
    }
}
