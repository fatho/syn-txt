#[derive(Debug, Copy, Clone)]
pub enum WaveShape {
    Sine,
    Saw,
    SuperSaw,
}

/// An oscillator sampling a wave of some shape at a fixed sample rate.
#[derive(Debug)]
pub struct Oscillator {
    shape: WaveShape,
    sample_rate: f64,
    frequency: f64,
    phase_offset: f64,
}

impl Oscillator {
    pub fn new(shape: WaveShape, sample_rate: f64, frequency: f64) -> Self {
        Self {
            shape,
            sample_rate,
            frequency,
            phase_offset: 0.0,
        }
    }

    pub fn next_sample(&mut self) -> f64 {
        let phase = self.phase_offset;
        // Increment phase
        let phase_increment = self.frequency / self.sample_rate;
        self.phase_offset += phase_increment;
        while self.phase_offset > 1.0 {
            self.phase_offset -= 1.0;
        }
        // Compute wave
        use std::f64::consts::PI;
        match self.shape {
            WaveShape::Sine => (phase * 2.0 * PI).sin(),
            WaveShape::Saw => 2.0 * phase - 1.0,
            WaveShape::SuperSaw => {
                let slope = 3.0;
                if phase < 0.5 {
                    slope * phase - 1.0
                } else {
                    1.0 + slope * (phase - 1.0)
                }
            }
        }
    }
}
