use std::default;

use slotmap::SecondaryMap;

use crate::engine::{
    audio_context::AudioContext,
    buffer::{Buffer, Frame},
    graph::{AudioGraph, AudioNode, Connection, GraphError, NodeKey},
    node::Node,
};

// Arbitrary max init. inputs
const MAX_INITIAL_INPUTS: usize = 8;

pub struct Runtime<const AF: usize, const CF: usize, const CHANNELS: usize> {
    context: AudioContext,
    graph: AudioGraph<AF, CF>,
    port_sources: SecondaryMap<NodeKey, Vec<Buffer<AF>>>, // A of all nodes/ports, and their emitted sources
    inputs_scratch_buffer: Vec<Buffer<AF>>,
    sink_key: Option<NodeKey>,
    placeholder_ctrl_in: [Buffer<CF>; 1],
    placeholder_ctrl_out: [Buffer<CF>; 1],
}
impl<'a, const AF: usize, const CF: usize, const CHANNELS: usize> Runtime<AF, CF, CHANNELS> {
    pub fn new(context: AudioContext, graph: AudioGraph<AF, CF>) -> Self {
        let port_sources = SecondaryMap::with_capacity(graph.len());
        Self {
            context,
            graph,
            port_sources,
            inputs_scratch_buffer: vec![Buffer::<AF>::SILENT; MAX_INITIAL_INPUTS],
            sink_key: None,
            placeholder_ctrl_in: [Buffer::<CF>::default()],
            placeholder_ctrl_out: [Buffer::<CF>::default()],
        }
    }
    pub fn add_node(&mut self, node: AudioNode<AF, CF>) -> NodeKey {
        let node_source_length = node.get_outputs().len();
        let node_key = self.graph.add_node(node);

        self.port_sources
            .insert(node_key, vec![Buffer::<AF>::SILENT; node_source_length]);

        node_key
    }
    pub fn remove_node(&mut self, key: NodeKey) {
        self.graph.remove_node(key);
        self.port_sources.remove(key);
    }
    pub fn add_edge(&mut self, connection: Connection) -> Result<Connection, GraphError> {
        self.graph.add_edge(connection)
    }
    pub fn remove_edge(&mut self, connection: Connection) -> Result<(), GraphError> {
        self.graph.remove_edge(connection)
    }
    pub fn set_sink_key(&mut self, key: NodeKey) -> Result<(), GraphError> {
        match self.graph.exists(key) {
            true => {
                self.sink_key = Some(key);
                Ok(())
            }
            false => Err(GraphError::NodeDoesNotExist),
        }
    }
    // TODO: Graphs as nodes again
    pub fn next_block(&mut self) -> &[Buffer<AF>] {
        let (sorted_order, nodes, incoming) = self.graph.get_nodes_and_runtime_info(); // TODO: I don't like this
        for node_key in sorted_order.iter() {
            let node = nodes
                .get_mut(*node_key)
                .expect("Could not find node at index {node_index:?}");

            let input_size = node.get_inputs().len();

            // Zero input buffer
            for i in 0..input_size {
                for n in 0..AF {
                    self.inputs_scratch_buffer[i][n] = 0.0;
                }
            }

            let incoming = incoming.get(*node_key).expect("Invalid connection!");

            for connection in incoming {
                // Write all incoming data from the connection and port, to the current node, and the sink port
                debug_assert!(connection.sink_key == *node_key);
                self.inputs_scratch_buffer[connection.sink_port_index] =
                    self.port_sources[connection.source_key][connection.source_port_index];
            }

            let output = &mut self.port_sources[*node_key]; // Let the node write to the output as a mut_slice

            node.process(
                &self.context,
                &self.inputs_scratch_buffer[0..input_size],
                output.as_mut_slice(),
                self.placeholder_ctrl_in.as_slice(),
                self.placeholder_ctrl_out.as_mut_slice(),
            );
        }

        let sink_key = self.sink_key.expect("Sink node must be provided");
        self.port_sources
            .get(sink_key)
            .expect("Invalid output port!")
            .as_slice()
    }
}

pub fn build_runtime<const AF: usize, const CF: usize, const CHANNEL_SIZE: usize>(
    initial_capacity: usize,
    sample_rate: f32,
    control_rate: f32,
) -> Runtime<AF, CF, CHANNEL_SIZE> {
    let graph = AudioGraph::with_capacity(initial_capacity);
    let context = AudioContext::new(sample_rate, control_rate);

    Runtime::new(context, graph)
}
