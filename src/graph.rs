//! A graph describes the audio equipment.
//! The events fed into the graph define the song in an abstract way.
//! The output is music.

use std::cell::RefCell;
use std::rc::Rc;

use crate::note::{Note, Velocity};
use crate::wave::AudioBuffer;

/// Time measured in samples.
pub type Sample = usize;

pub struct NodeId(usize);

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

    pub fn add_node(&mut self, node: Box<dyn Node>) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(NodeHolder::new(node, self.buffer_size));
        id
    }

    pub fn step(&mut self) {
        for n in self.nodes.iter_mut() {
            let rio = RenderIo {
                start: self.time,
                length: self.buffer_size,
                events: &[],
                inputs: &[],
                outputs: &n.output_buffers,
            };
            n.node.render(&rio);
        }

        self.time += self.buffer_size;
    }

    // pub fn connect(&mut self, from: NodeId, to: NodeId, )
}

struct NodeHolder {
    node: Box<dyn Node>,
    output_buffers: Vec<Rc<RefCell<AudioBuffer>>>,
}

impl NodeHolder {
    fn new(node: Box<dyn Node>, buffer_size: Sample) -> Self {
        let output_buffers = std::iter::repeat_with(|| Rc::new(RefCell::new(AudioBuffer::new(buffer_size)))).take(node.num_outputs()).collect();
        Self {
            node,
            output_buffers,
        }
    }
}

pub trait Node {
    fn num_outputs(&self) -> usize;
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
