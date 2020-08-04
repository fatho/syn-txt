//! Evaluating how the graph interface could be used

use syn_txt::{wave::Stereo, graph::*};

fn main() {
    let mut graph = Graph::new();
    let n = graph.add_node(Box::new(Sine { samples_per_second: 44100, frequency: 440.0 }));
    graph.step();
}



pub struct Sine {
    samples_per_second: Sample,
    frequency: f64,
}

impl Node for Sine {
    fn num_outputs(&self) -> usize {
        1
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
