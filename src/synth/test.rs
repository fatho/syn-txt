//! Exemplary implementation of a synthesizer.

use crate::note::*;
use crate::synth::*;
use crate::wave::*;

pub struct TestSynth {
    current_time: usize,
    sampler: SamplerInfo,
    tuning: Tuning,
    active_voices: Vec<TestSynthVoice>,
    fading_voices: Vec<TestSynthVoice>,
}

struct TestSynthVoice {
    note: Note,
    volume: f64,
    frequency: f64,
    duration: usize,
    released: Option<usize>,
    previous: f64,
}

struct Envelope {
    attack: f64,
    decay: f64,
    sustain: f64,
    release: f64,
}

impl Envelope {
    pub fn volume_held(&self, t: f64) -> f64 {
        if t < self.attack {
            t / self.attack
        } else if t < self.attack + self.decay {
            self.sustain + (1.0 - self.sustain) * (1.0 - (t - self.attack) / self.decay)
        } else {
            self.sustain
        }
    }

    pub fn volume_released(&self, t: f64) -> Option<f64> {
        if t < self.release {
            Some(self.sustain * (1.0 - t/ self.release))
        } else {
            None
        }
    }
}


impl TestSynth {
    pub fn new(epoch: usize, sampler: SamplerInfo) -> Self {
        TestSynth {
            current_time: epoch,
            active_voices: vec![],
            fading_voices: vec![],
            tuning: Tuning::default(),
            sampler,
        }
    }

    pub fn play(&mut self, mut events: &[Event], output: &mut [Stereo<f64>]) {
        for i in 0..output.len() {
            let t = self.current_time + i;
            // Process starting and stopping notes before or at this sample
            while let Some(event) = events.first() {
                if event.time <= t {
                    match event.action {
                        NoteAction::Play { note, velocity } => {
                            let new_voice = TestSynthVoice {
                                note,
                                volume: velocity.0 as f64 / std::u8::MAX as f64,
                                frequency: self.note_frequency(note),
                                duration: 0,
                                released: None,
                                previous: 0.0,
                            };
                            self.active_voices.push(new_voice);
                        }
                        NoteAction::Release { note } => {
                            if let Some(note_voice) = self.active_voices.iter().position(|voice| voice.note == note) {
                                let mut voice = self.active_voices.swap_remove(note_voice);
                                voice.released = Some(voice.duration);
                                self.fading_voices.push(voice);
                            }
                        }
                    }
                    events = &events[1..];
                } else {
                    break;
                }
            }

            let mut wave = 0.0;

            for voice in self.active_voices.iter_mut() {
                wave += voice.sample(&self.sampler).unwrap_or(0.0);
            }
            let fading_voice_count = self.fading_voices.len();
            for voice_index in 0..fading_voice_count {
                let reverse_index = fading_voice_count - 1 - voice_index;
                if let Some(sample) = self.fading_voices[reverse_index].sample(&self.sampler) {
                    wave += sample;
                } else {
                    self.fading_voices.swap_remove(reverse_index);
                }
            }

            output[i].left += wave;
            output[i].right += wave;
        }
        self.current_time += output.len()
    }

    fn note_frequency(&self, note: Note) -> f64 {
        let note_delta = note.0 as f64 - self.tuning.note.0 as f64;
        self.tuning.frequency * 2.0f64.powf(note_delta / 12.0)
    }
}

impl TestSynthVoice {
    fn sample(&mut self, sampler: &SamplerInfo) -> Option<f64> {
        let env = Envelope {
            attack: 0.05,
            decay: 1.0,
            sustain: 0.5,
            release: 0.25
        };

        let time = self.duration as f64 / sampler.sample_rate as f64;

        let sine = (time * self.frequency * 2.0 * std::f64::consts::PI).sin();
        let detune = 2.0f64.powf(5.0 / 1200.0);
        let saw1 = 2.0 * (time * self.frequency * detune * 0.5).fract() - 1.0;
        let saw2 = 2.0 * (time * self.frequency / detune * 0.5).fract() - 1.0;

        let shape = sine * 0.5 + saw1 * 0.25 + saw2 * 0.25;

        let amplitude = if let Some(released) = self.released {
            let t = (self.duration - released) as f64 / sampler.sample_rate as f64;
            env.volume_released(t)
        } else {
            let t = self.duration as f64 / sampler.sample_rate as f64;
            Some(env.volume_held(t))
        };

        self.duration += 1;

        amplitude.map(|a| {
            let current = a * shape * self.volume;
            let previous = self.previous;
            let alpha = (-2.0 * std::f64::consts::PI * 440.0).exp();
            self.previous = (1.0 - alpha) * current + alpha * previous;
            self.previous
        })
    }
}
