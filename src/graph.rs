// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

//! A graph describes the audio equipment.
//! The events fed into the graph define the song in an abstract way.
//! The output is music.

use std::cell::RefCell;
use std::rc::Rc;

use snafu::Snafu;

use crate::note::{Note, Velocity};
use crate::wave::AudioBuffer;

pub mod instrument;
pub mod sox;

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

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node<N: Node + 'static>(&mut self, node: N) -> NodeBuilder<'_> {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Box::new(node));
        NodeBuilder {
            graph_builder: self,
            node: id,
        }
    }

    /// Consume the GraphBuilder and turn it into a graph, provided that the graph structure has no cycles.
    ///
    /// NOTE: Currently, failure to build the graph means that all nodes are lost.
    /// Do we need a way to recover those?
    pub fn build(self, buffer_size: Sample) -> Result<Graph, GraphBuildError> {
        let mut nodes: Vec<NodeHolder> = self
            .nodes
            .into_iter()
            .map(|node| NodeHolder::new(node, buffer_size))
            .collect();

        // Connect the output buffers to the inputs and prepare topological sorting
        for (output, input) in self.edges {
            let buffer = Rc::clone(
                nodes
                    .get(output.node.0)
                    .ok_or(GraphBuildError::InvalidNode { node: output.node })?
                    .output_buffers
                    .get(output.index)
                    .ok_or(GraphBuildError::InvalidOutput { output })?,
            );
            *(nodes
                .get_mut(input.node.0)
                .ok_or(GraphBuildError::InvalidNode { node: input.node })?
                .input_buffers
                .get_mut(input.index)
                .ok_or(GraphBuildError::InvalidInput { input })?) = buffer;

            if !nodes[input.node.0].incoming.contains(&output.node) {
                nodes[input.node.0].incoming.push(output.node);
            }
            if !nodes[output.node.0].outgoing.contains(&input.node) {
                nodes[output.node.0].outgoing.push(input.node);
            }
        }

        // Topological sort using Kahn's algorithm
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

        // Cycles are bad because then the order is undefined and makes a difference.
        // Better solution: add explicit support for feedback loops to the graph builder if ever necessary.
        if nodes.iter().any(|n| !n.incoming.is_empty()) {
            Err(GraphBuildError::Cycle)
        } else {
            Ok(Graph {
                nodes,
                evaluation_order: sorted_nodes,
                time: 0,
                buffer_size,
            })
        }
    }
}

/// Possible errors when building a graph.
#[derive(Debug, PartialEq, Snafu)]
pub enum GraphBuildError {
    #[snafu(display("There is a cycle in the graph"))]
    Cycle,
    #[snafu(display("Referenced node {:?} does not exist", node))]
    InvalidNode { node: NodeId },
    #[snafu(display("Referenced input {:?} does not exist", input))]
    InvalidInput { input: InputRef },
    #[snafu(display("Referenced output {:?} does not exist", output))]
    InvalidOutput { output: OutputRef },
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

    // REVIEW: allow getting a reference back to a node and downcast?

    // pub fn connect(&mut self, from: NodeId, to: NodeId, )
}

struct NodeHolder {
    node: Box<dyn Node>,
    input_buffers: Vec<Rc<RefCell<AudioBuffer>>>,
    output_buffers: Vec<Rc<RefCell<AudioBuffer>>>,

    // REVIEW: do we need those here? They are only used temporarily for topological sorting
    incoming: Vec<NodeId>,
    outgoing: Vec<NodeId>,
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
            incoming: Vec::new(),
            outgoing: Vec::new(),
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
    events: &'a [Event],
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
/// IDEA: Events are but an illusion. They can actually just be baked into their respective nodes without the graph knowing.
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

#[cfg(test)]
mod tests {

    use super::*;

    /// Check that the builder correctly errors out on a cycle.
    #[test]
    fn cycle_detection() {
        let mut b = GraphBuilder::new();
        let sink = b.add_node(Sink).build();
        let x = b.add_node(FanOut).output_to(0, sink.input(0)).build();
        let y = b
            .add_node(FanIn)
            .output_to(0, x.input(0))
            .input_from(1, x.output(1))
            .build();
        let _source = b.add_node(Source).output_to(0, y.input(0)).build();

        match b.build(10) {
            Ok(_) => panic!("Expected graph build to fail due to cycle"),
            Err(err) => assert_eq!(err, GraphBuildError::Cycle),
        }
    }

    /// Check that nodes are evaluated from source to sink,
    /// even when they're added in a different order.
    #[test]
    fn correct_order() {
        let mut b = GraphBuilder::new();
        let sink = b.add_node(Sink).build();
        let x = b.add_node(FanOut).build();
        let y = b
            .add_node(FanIn)
            .output_to(0, sink.input(0))
            .input_from(0, x.output(1))
            .input_from(1, x.output(0))
            .build();
        let source = b.add_node(Source).output_to(0, x.input(0)).build();

        let graph = b.build(10).unwrap();

        assert_eq!(graph.evaluation_order, vec![source, x, y, sink]);
    }

    pub struct Source;
    impl Node for Source {
        fn num_inputs(&self) -> usize {
            0
        }
        fn num_outputs(&self) -> usize {
            1
        }
        fn render(&mut self, _rio: &RenderIo) {}
    }

    pub struct FanOut;
    impl Node for FanOut {
        fn num_inputs(&self) -> usize {
            1
        }
        fn num_outputs(&self) -> usize {
            2
        }
        fn render(&mut self, _rio: &RenderIo) {}
    }

    pub struct FanIn;
    impl Node for FanIn {
        fn num_inputs(&self) -> usize {
            2
        }
        fn num_outputs(&self) -> usize {
            1
        }
        fn render(&mut self, _rio: &RenderIo) {}
    }

    pub struct Sink;
    impl Node for Sink {
        fn num_inputs(&self) -> usize {
            1
        }
        fn num_outputs(&self) -> usize {
            0
        }
        fn render(&mut self, _rio: &RenderIo) {}
    }
}
