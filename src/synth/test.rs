//! Exemplary implementation of a synthesizer.

use super::envelope::*;
use super::event::*;
use super::oscillator::*;
use super::tuning::*;

use crate::note::*;
use crate::wave::*;

pub struct TestSynth {
    current_time: usize,
    sample_rate: f64,
    tuning: Tuning,
    active_voices: Vec<TestSynthVoice>,
    fading_voices: Vec<TestSynthVoice>,
}

struct TestSynthVoice {
    note: Note,
    amplitude: f64,
    sine: Oscillator,
    saw1: Oscillator,
    saw2: Oscillator,
    envelope: ADSR,
    time: EnvelopeTime,
    time_increment: f64,
}

impl TestSynth {
    pub fn new(epoch: usize, sample_rate: f64) -> Self {
        TestSynth {
            current_time: epoch,
            active_voices: vec![],
            fading_voices: vec![],
            tuning: Tuning::default(),
            sample_rate,
        }
    }
}

impl super::Synthesizer for TestSynth {
    fn play(&mut self, mut events: &[Event], output: &mut [Stereo<f64>]) {
        for (i, out_sample) in output.iter_mut().enumerate() {
            let t = self.current_time + i;
            // Process starting and stopping notes before or at this sample
            while let Some(event) = events.first() {
                if event.time <= t {
                    match event.action {
                        NoteAction::Play { note, velocity } => {
                            let new_voice = TestSynthVoice::new(
                                note,
                                velocity.amplitude(),
                                self.tuning.frequency(note),
                                self.sample_rate,
                            );
                            self.active_voices.push(new_voice);
                        }
                        NoteAction::Release { note } => {
                            if let Some(note_voice) = self
                                .active_voices
                                .iter()
                                .position(|voice| voice.note == note)
                            {
                                let mut voice = self.active_voices.swap_remove(note_voice);
                                voice.time = EnvelopeTime::release();
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
                wave += voice.sample()
            }
            let fading_voice_count = self.fading_voices.len();
            for voice_index in 0..fading_voice_count {
                let reverse_index = fading_voice_count - 1 - voice_index;
                wave += self.fading_voices[reverse_index].sample();
                if self.fading_voices[reverse_index].faded() {
                    self.fading_voices.swap_remove(reverse_index);
                }
            }

            out_sample.left += wave;
            out_sample.right += wave;
        }
        self.current_time += output.len()
    }
}

impl TestSynthVoice {
    fn new(note: Note, amplitude: f64, frequency: f64, sample_rate: f64) -> Self {
        let detune = 2.0f64.powf(5.0 / 1200.0);
        Self {
            note,
            amplitude,
            sine: Oscillator::new(WaveShape::Sine, sample_rate, frequency),
            saw1: Oscillator::new(WaveShape::Saw, sample_rate, frequency * 0.5 * detune),
            saw2: Oscillator::new(WaveShape::Saw, sample_rate, frequency * 0.5 / detune),
            envelope: ADSR {
                attack: 0.05,
                decay: 1.0,
                sustain: 0.5,
                release: 0.25,
            },
            time: EnvelopeTime::press(),
            time_increment: 1.0 / sample_rate,
        }
    }

    fn sample(&mut self) -> f64 {
        self.time.advance(self.time_increment);
        let sine = self.sine.next_sample();
        let saw1 = self.saw1.next_sample();
        let saw2 = self.saw2.next_sample();

        let shape = sine * 0.5 + saw1 * 0.25 + saw2 * 0.25;
        let envelope = self.envelope.eval(self.time);

        shape * envelope * self.amplitude
    }

    fn faded(&self) -> bool {
        if let EnvelopeTime::SinceRelease(t) = self.time {
            t >= self.envelope.release
        } else {
            false
        }
    }
}
