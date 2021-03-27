// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2021  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! A graph describes the audio equipment.
//! The events fed into the graph define the song in an abstract way.
//! The output is music.

use std::cell::RefCell;
use std::rc::Rc;

use crate::wave::AudioBuffer;

mod builder;
mod instrument;
mod sox;
mod transducers;

pub use builder::{GraphBuildError, GraphBuilder};
pub use instrument::InstrumentSource;
pub use sox::{SoxSink, SoxTarget};
pub use transducers::*;

/// Time measured in samples.
pub type Sample = usize;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub struct NodeId(usize);

impl NodeId {
    pub fn input(self, index: usize) -> InputRef {
        InputRef { node: self, index }
    }

    pub fn output(self, index: usize) -> OutputRef {
        OutputRef { node: self, index }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct InputRef {
    node: NodeId,
    index: usize,
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct OutputRef {
    node: NodeId,
    index: usize,
}

pub struct Graph {
    nodes: Vec<NodeHolder>,
    evaluation_order: Vec<NodeId>,
    time: Sample,
    buffer_size: Sample,
}

impl Graph {
    pub fn step(&mut self) {
        for id in self.evaluation_order.iter() {
            let holder = &mut self.nodes[id.0];

            let rio = RenderIo {
                start: self.time,
                length: self.buffer_size,
                inputs: &holder.input_buffers,
                outputs: &holder.output_buffers,
            };
            holder.node.render(&rio);
        }
        self.time += self.buffer_size;
    }

    // REVIEW: allow getting a reference back to a node and downcast?

    // pub fn connect(&mut self, from: NodeId, to: NodeId, )
}

struct NodeHolder {
    node: Box<dyn Node>,
    input_buffers: Vec<Rc<RefCell<AudioBuffer>>>,
    output_buffers: Vec<Rc<RefCell<AudioBuffer>>>,
}

impl NodeHolder {
    fn new(node: Box<dyn Node>, buffer_size: Sample) -> Self {
        // TODO: It is wasteful that we create input buffers here that are most likely
        // immediately deallocated again when overwritten in the GraphBuilder
        let input_buffers =
            std::iter::repeat_with(|| Rc::new(RefCell::new(AudioBuffer::new(buffer_size))))
                .take(node.num_inputs())
                .collect();
        let output_buffers =
            std::iter::repeat_with(|| Rc::new(RefCell::new(AudioBuffer::new(buffer_size))))
                .take(node.num_outputs())
                .collect();

        Self {
            node,
            input_buffers,
            output_buffers,
        }
    }
}

pub trait Node {
    /// Number of input nodes.
    fn num_inputs(&self) -> usize;

    /// Number of ouput nodes.
    fn num_outputs(&self) -> usize;

    fn render(&mut self, rio: &RenderIo);
}

/// References to inputs and outputs while rendering a node.
pub struct RenderIo<'a> {
    /// Sample time of the first sample in these buffers.
    start: Sample,
    /// Number of samples in these buffers
    length: Sample,
    /// One buffer for each input of the node.
    /// TODO: make non-connected buffers `None` instead of the empty buffer, so that the nodes can detect that.
    inputs: &'a [Rc<RefCell<AudioBuffer>>],
    outputs: &'a [Rc<RefCell<AudioBuffer>>],
}

impl<'a> RenderIo<'a> {
    pub fn start(&self) -> Sample {
        self.start
    }

    pub fn length(&self) -> Sample {
        self.length
    }

    pub fn input(&self, index: usize) -> std::cell::Ref<AudioBuffer> {
        self.inputs[index].borrow()
    }

    pub fn output(&self, index: usize) -> std::cell::RefMut<AudioBuffer> {
        self.outputs[index].borrow_mut()
    }
}
