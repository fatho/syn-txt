//! A graph describes the audio equipment.
//! The events fed into the graph define the song in an abstract way.
//! The output is music.

use std::cell::RefCell;
use std::rc::Rc;

use crate::note::{Note, Velocity};
use crate::wave::AudioBuffer;

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

/// Construct an audio graph.
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

        // Connect the output buffers to the inputs and prepare topological sorting
        for (output, input) in self.edges {
            nodes[input.node.0].input_buffers[input.index] =
                Rc::clone(&nodes[output.node.0].output_buffers[output.index]);

            if !nodes[input.node.0].incoming.contains(&output.node) {
                nodes[input.node.0].incoming.push(output.node);
            }
            if !nodes[output.node.0].outgoing.contains(&input.node) {
                nodes[output.node.0].outgoing.push(input.node);
            }
        }

        // Topological sort Kahn's algorithm
        let mut sorted_nodes = Vec::new();
        let mut nodes_without_incoming_edges: Vec<_> = nodes
            .iter()
            .enumerate()
            .filter(|(_, holder)| holder.incoming.is_empty())
            .map(|(id, _)| NodeId(id))
            .collect();

        while let Some(n) = nodes_without_incoming_edges.pop() {
            sorted_nodes.push(n);

            for m in std::mem::replace(&mut nodes[n.0].outgoing, Vec::new()) {
                nodes[m.0].incoming.retain(|x| *x != n);
                if nodes[m.0].incoming.is_empty() {
                    nodes_without_incoming_edges.push(m);
                }
            }
        }

        // TODO: validate that there are no circles, otherwise, we might get runtime panics
        // (Can we actually have circles based on how things are built?)

        Graph {
            nodes,
            evaluation_order: sorted_nodes,
            time: 0,
            buffer_size,
        }
    }
}

/// Construct the connections between nodes.
pub struct NodeBuilder<'a> {
    graph_builder: &'a mut GraphBuilder,
    node: NodeId,
}

impl<'a> NodeBuilder<'a> {
    /// Feed the output of this node with the given index to the input of another node.
    pub fn output_to(self, output_index: usize, input: InputRef) -> NodeBuilder<'a> {
        self.graph_builder
            .edges
            .push((self.node.output(output_index), input));
        self
    }

    /// Receive the output of another node at the given input of this node.
    pub fn input_from(self, input_index: usize, output: OutputRef) -> NodeBuilder<'a> {
        self.graph_builder
            .edges
            .push((output, self.node.input(input_index)));
        self
    }

    /// Stop building this node, returning its ID for future references.
    pub fn build(self) -> NodeId {
        self.node
    }
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
                events: &[],
                inputs: &holder.input_buffers,
                outputs: &holder.output_buffers,
            };
            holder.node.render(&rio);
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

    // TODO: do we need those still? They are only used temporarily for topological sorting
    incoming: Vec<NodeId>,
    outgoing: Vec<NodeId>,
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
            // TODO: These are only used during topological sorting
            incoming: Vec::new(),
            outgoing: Vec::new(),
        }
    }
}

pub trait Node {
    /// Static description of the node inputs.
    /// TODO: support dynamic number of inputs (useful for a mixer)
    fn inputs(&self) -> &'static [&'static str];

    /// Static description of the node outputs.
    fn outputs(&self) -> &'static [&'static str];

    fn render(&mut self, rio: &RenderIo);
}

/// References to inputs and outputs while rendering a node.
pub struct RenderIo<'a> {
    /// Sample time of the first sample in these buffers.
    start: Sample,
    /// Number of samples in these buffers
    length: Sample,
    events: &'a [Event],
    /// One buffer for each input of the node.
    /// TODO: make non-connected buffers `None` instead of the empty buffer,
    /// so that the nodes can detect that.
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

/// An event can influence a note at a discrete time.
/// TODO: Actually do something with events
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
