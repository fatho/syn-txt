//! A graph describes the audio equipment.
//! The events fed into the graph define the song in an abstract way.
//! The output is music.

use std::cell::RefCell;
use std::rc::Rc;

use crate::note::{Note, Velocity};
use crate::wave::AudioBuffer;

/// Time measured in samples.
pub type Sample = usize;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
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

pub struct GraphBuilder {
    nodes: Vec<Box<dyn Node>>,
    edges: Vec<(OutputRef, InputRef)>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node<'a, N: Node + 'static>(&'a mut self, node: N) -> NodeBuilder<'a> {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Box::new(node));
        NodeBuilder {
            graph_builder: self,
            node: id,
        }
    }

    pub fn build(self, buffer_size: Sample) -> Graph {
        let mut nodes: Vec<NodeHolder> = self
            .nodes
            .into_iter()
            .map(|node| NodeHolder::new(node, buffer_size))
            .collect();
        // TODO: validate that there are no circles, otherwise, we might get runtime panics

        // Connect the output buffers to the inputs
        for (output, input) in self.edges {
            nodes[input.node.0].input_buffers[input.index] =
                Rc::clone(&nodes[output.node.0].output_buffers[output.index]);
        }

        Graph {
            nodes,
            time: 0,
            buffer_size,
        }
    }
}

pub struct NodeBuilder<'a> {
    graph_builder: &'a mut GraphBuilder,
    node: NodeId,
}

impl<'a> NodeBuilder<'a> {
    pub fn output_to(self, output_index: usize, input: InputRef) -> NodeBuilder<'a> {
        self.graph_builder
            .edges
            .push((self.node.output(output_index), input));
        self
    }

    pub fn input_from(self, input_index: usize, output: OutputRef) -> NodeBuilder<'a> {
        self.graph_builder
            .edges
            .push((output, self.node.input(input_index)));
        self
    }

    pub fn build(self) -> NodeId {
        self.node
    }
}

pub struct Graph {
    nodes: Vec<NodeHolder>,
    time: Sample,
    buffer_size: Sample,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            time: 0,
            buffer_size: 1024,
        }
    }

    pub fn step(&mut self) {
        for n in self.nodes.iter_mut() {
            let rio = RenderIo {
                start: self.time,
                length: self.buffer_size,
                events: &[],
                inputs: &n.input_buffers,
                outputs: &n.output_buffers,
            };
            n.node.render(&rio);
        }

        self.time += self.buffer_size;
    }

    // TODO: allow getting a reference back to a node and downcast?

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
                .take(node.inputs().len())
                .collect();
        let output_buffers =
            std::iter::repeat_with(|| Rc::new(RefCell::new(AudioBuffer::new(buffer_size))))
                .take(node.outputs().len())
                .collect();

        Self {
            node,
            input_buffers,
            output_buffers,
        }
    }
}

pub trait Node {
    fn inputs(&self) -> &'static [&'static str];
    fn outputs(&self) -> &'static [&'static str];
    fn render(&mut self, rio: &RenderIo);
}

pub struct RenderIo<'a> {
    start: Sample,
    length: Sample,
    events: &'a [Event],
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

    pub fn events(&self) -> &'a [Event] {
        self.events
    }

    pub fn input(&self, index: usize) -> std::cell::Ref<AudioBuffer> {
        self.inputs[index].borrow()
    }

    pub fn output(&self, index: usize) -> std::cell::RefMut<AudioBuffer> {
        self.outputs[index].borrow_mut()
    }
}

pub struct Event {
    pub when: Sample,
    pub payload: EventPayload,
}

pub enum EventPayload {
    Note {
        note: Note,
        duration: Sample,
        velocity: Velocity,
    },
}
