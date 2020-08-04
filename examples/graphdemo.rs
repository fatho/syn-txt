//! Evaluating how the graph interface could be used

use syn_txt::{graph::*, wave::Stereo};

fn main() {
    let mut builder = GraphBuilder::new();
    let sine = builder
        .add_node(Sine {
            samples_per_second: 44100,
            frequency: 440.0,
        })
        .build();
    let sink = builder
        .add_node(DebugSink)
        .input_from(0, sine.output(0))
        .build();

    let mut graph = builder.build(1024);
    graph.step();
}

pub struct DebugSink;

impl Node for DebugSink {
    fn outputs(&self) -> &'static [&'static str] {
        &[]
    }

    fn inputs(&self) -> &'static [&'static str] {
        &["main"]
    }

    fn render(&mut self, rio: &RenderIo) {
        let input = rio.input(0);
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

pub struct Sine {
    samples_per_second: Sample,
    frequency: f64,
}

impl Node for Sine {
    fn outputs(&self) -> &'static [&'static str] {
        &["main"]
    }

    fn inputs(&self) -> &'static [&'static str] {
        &[]
    }

    fn render(&mut self, rio: &RenderIo) {
        let mut out = rio.output(0);
        let buf = out.samples_mut();
        for i in 0..rio.length() {
            let t = (i + rio.start()) as f64 / self.samples_per_second as f64;
            buf[i] = Stereo::mono((2.0 * std::f64::consts::PI * t * self.frequency).sin());
        }
    }
}
