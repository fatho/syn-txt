//! Evaluating how the graph interface could be used

use syn_txt::{graph::*, wave::Stereo};

fn main() {
    let mut builder = GraphBuilder::new();
    let sine = builder
        .add_node(Sine {
            samples_per_second: 44100,
            amplitude: 0.5,
            frequency: 440.0,
        })
        .build();
    // let sink = builder
    //     .add_node(DebugSink)
    //     .input_from(0, sine.output(0))
    //     .build();
    let sink = builder
        .add_node(sox::SoxSink::new(44100, sox::SoxTarget::Play).unwrap())
        .input_from(0, sine.output(0))
        .build();

    let sine2 = builder
        .add_node(Sine {
            samples_per_second: 44100,
            amplitude: 0.5,
            frequency: 440.0 * 2.0,
        })
        .build();
    let _sum = builder
        .add_node(Sum { num_inputs: 2 })
        .input_from(0, sine.output(0))
        .input_from(1, sine2.output(0))
        .output_to(0, sink.input(0))
        .build();

    let mut graph = builder.build(1024).unwrap();
    for _ in 0..40 {
        graph.step();
    }
}

/// Add audio streams together.
pub struct Sum {
    num_inputs: usize,
}

impl Node for Sum {
    fn num_inputs(&self) -> usize {
        self.num_inputs
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn render(&mut self, rio: &RenderIo) {
        let mut out = rio.output(0);
        out.fill_zero();
        let outsamples = out.samples_mut();

        for i in 0..self.num_inputs {
            let in_ref = rio.input(i);
            let in_samples = in_ref.samples();

            for (i, o) in in_samples.iter().zip(outsamples.iter_mut()) {
                *o += *i;
            }
        }
    }
}

/// Render all the incoming audio data on the terminal.
pub struct DebugSink;

impl Node for DebugSink {
    fn num_outputs(&self) -> usize {
        0
    }

    fn num_inputs(&self) -> usize {
        1
    }

    fn render(&mut self, rio: &RenderIo) {
        let input = rio.input(0);

        println!("[{}]", rio.start());
        println!("|{: ^41}|{: ^41}|", "left", "right");
        println!(
            "|-----------------------------------------|-----------------------------------------|"
        );
        let half_width = 20;
        let width = half_width * 2;
        for s in input.samples() {
            let lpos = half_width + (s.left * half_width as f64).round() as i64;
            let rpos = half_width + (s.right * half_width as f64).round() as i64;

            print!("|");
            (0..lpos).for_each(|_| print!(" "));
            print!("*");
            (0..width - lpos).for_each(|_| print!(" "));
            print!("|");
            (0..rpos).for_each(|_| print!(" "));
            print!("*");
            (0..width - rpos).for_each(|_| print!(" "));
            println!("|");
        }
    }
}

/// Continuously generate a simple sine wave.
pub struct Sine {
    samples_per_second: Sample,
    amplitude: f64,
    frequency: f64,
}

impl Node for Sine {
    fn num_outputs(&self) -> usize {
        1
    }

    fn num_inputs(&self) -> usize {
        0
    }

    fn render(&mut self, rio: &RenderIo) {
        let mut out = rio.output(0);
        let buf = out.samples_mut();
        for i in 0..rio.length() {
            let t = (i + rio.start()) as f64 / self.samples_per_second as f64;
            buf[i] = Stereo::mono(
                (2.0 * std::f64::consts::PI * t * self.frequency).sin() * self.amplitude,
            );
        }
    }
}
