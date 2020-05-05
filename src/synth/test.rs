//! Exemplary implementation of a synthesizer.

use super::envelope::*;
use super::event::*;
use super::oscillator::*;
use super::tuning::*;

use crate::note::*;
use crate::wave::*;

pub struct TestSynth {
    /// Used for synchronizing events with playback.
    current_time: usize,
    /// Samples per second rate of the generated audio signal.
    sample_rate: f64,
    /// Reference note and frequency, determining the pitch of all other notes.
    tuning: Tuning,

    /// Evenlope for played notes
    envelope: ADSR,

    // /// Output gain of the synthesizer
    // gain: f64,

    // /// Number of oscillators per voice
    // unison: usize,
    // /// Detune offset for each additional unison voice
    // unison_detune: f64,

    active_voices: Vec<TestSynthVoice>,
    fading_voices: Vec<TestSynthVoice>,
}

struct TestSynthVoice {
    note: Note,
    amplitude: f64,
    sine: Oscillator,
    saw1: Oscillator,
    saw2: Oscillator,
    envelope: EvalADSR,
}

impl TestSynth {
    pub fn new(epoch: usize, sample_rate: f64) -> Self {
        TestSynth {
            current_time: epoch,
            active_voices: vec![],
            fading_voices: vec![],
            tuning: Tuning::default(),
            envelope: ADSR {
                attack: 0.05,
                decay: 1.0,
                sustain: 0.5,
                release: 0.25,
            },
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
                                self.envelope.instantiate(self.sample_rate),
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
                                voice.envelope.release();
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
    fn new(note: Note, amplitude: f64, frequency: f64, sample_rate: f64, envelope: EvalADSR) -> Self {
        let detune = 2.0f64.powf(5.0 / 1200.0);
        Self {
            note,
            amplitude,
            sine: Oscillator::new(WaveShape::Sine, sample_rate, frequency),
            saw1: Oscillator::new(WaveShape::Saw, sample_rate, frequency * 0.5 * detune),
            saw2: Oscillator::new(WaveShape::Saw, sample_rate, frequency * 0.5 / detune),
            envelope,
        }
    }

    fn sample(&mut self) -> f64 {
        let sine = self.sine.next_sample();
        let saw1 = self.saw1.next_sample();
        let saw2 = self.saw2.next_sample();

        let shape = sine * 0.5 + saw1 * 0.25 + saw2 * 0.25;
        let envelope = self.envelope.step();

        shape * envelope * self.amplitude
    }

    fn faded(&self) -> bool {
        self.envelope.faded()
    }
}
