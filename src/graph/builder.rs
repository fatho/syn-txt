// syn.txt -- a text based synthesizer and audio workstation
// Copyright (C) 2020  Fabian Thorand
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation.
//
// A copy of the license can be found in the LICENSE file in the root of
// this repository.

use std::rc::Rc;

use snafu::Snafu;

use super::*;
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


        let mut incoming: Vec<Vec<NodeId>> = std::iter::repeat(Vec::new()).take(nodes.len()).collect();
        let mut outgoing: Vec<Vec<NodeId>> = std::iter::repeat(Vec::new()).take(nodes.len()).collect();

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

            if !incoming[input.node.0].contains(&output.node) {
                incoming[input.node.0].push(output.node);
            }
            if !outgoing[output.node.0].contains(&input.node) {
                outgoing[output.node.0].push(input.node);
            }
        }

        // Topological sort using Kahn's algorithm
        let mut sorted_nodes = Vec::new();
        let mut nodes_without_incoming_edges: Vec<_> = incoming
            .iter()
            .enumerate()
            .filter(|(_, from)| from.is_empty())
            .map(|(id, _)| NodeId(id))
            .collect();

        while let Some(n) = nodes_without_incoming_edges.pop() {
            sorted_nodes.push(n);

            for m in std::mem::replace(&mut outgoing[n.0], Vec::new()) {
                incoming[m.0].retain(|x| *x != n);
                if incoming[m.0].is_empty() {
                    nodes_without_incoming_edges.push(m);
                }
            }
        }

        // Cycles are bad because then the order is undefined and makes a difference.
        // Better solution: add explicit support for feedback loops to the graph builder if ever necessary.
        if incoming.iter().any(|from| !from.is_empty()) {
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
