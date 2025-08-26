use slotmap::SecondaryMap;

use crate::engine::{audio_context::AudioContext, buffer::{Buffer, Frame}, graph::{AudioGraph, AudioNode, Connection, GraphError, NodeKey}};

// Arbitrary max init. inputs
const MAX_INITIAL_INPUTS: usize = 8;

pub struct Runtime<const BUFFER_SIZE: usize, const CHANNELS: usize> {
    context: AudioContext,
    graph: AudioGraph<BUFFER_SIZE>,
    execution_order: Vec<NodeKey>,
    port_sources: SecondaryMap<NodeKey, Vec<Buffer<BUFFER_SIZE>>>, // A of all nodes/ports, and their emitted sources
    inputs_scratch_buffer: Vec<Buffer<BUFFER_SIZE>>,
    sink_key: Option<NodeKey>
}
impl<const N: usize, const C: usize> Runtime<N, C> {
    pub fn new(context: AudioContext, mut graph: AudioGraph<N>) -> Self {
        let execution_order = graph.invalidate_topo_sort().expect("Invalid graph passed to runtime!");
        let port_sources = SecondaryMap::with_capacity(graph.len());

        Self {
            context,
            graph,
            execution_order,
            port_sources,
            inputs_scratch_buffer: Vec::with_capacity(MAX_INITIAL_INPUTS),
            sink_key: None
        }
    }
    pub fn add_node(&mut self, node: AudioNode<N>) -> NodeKey {
        let node_source_length = node.get_output_ports().len();
        let node_key = self.graph.add_node(node);
        self.port_sources.insert(node_key, Vec::with_capacity(node_source_length));

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
    // TODO: Graphs as nodes again
    pub fn next_block(&mut self) -> &[Buffer<N>] {
        for node_key in self.execution_order.iter() {
            // Clear the nodes, and double check that the size is reserved
            self.inputs_scratch_buffer.clear();
            self.inputs_scratch_buffer.reserve(MAX_INITIAL_INPUTS);

            let incoming = self.graph.incoming_connections(*node_key).expect("Invalid connection!");

            for connection in incoming {
                // Write all incoming data from the connection and port, to the current node, and the sink port
                debug_assert!(connection.sink_key == *node_key);
                self.inputs_scratch_buffer[connection.sink_port_index] = self.port_sources[connection.source_key][connection.source_port_index];
            }
            
            let node = self.graph.get_node_mut(node_key).expect("Could not find node at index {node_index:?}");

            let output = &mut self.port_sources[*node_key]; // Let the node write to the output as a mut_slice

            node.process(&self.context, self.inputs_scratch_buffer.as_slice(), output.as_mut_slice());
        }

        let sink_key = self.sink_key.expect("Sink node must be provided");
        self.port_sources.get(sink_key).expect("Invalid output port!").as_slice()
    }
}

pub fn build_runtime<const BUFFER_SIZE: usize, const CHANNEL_SIZE: usize>(initial_capacity: usize, sample_rate: u32) -> Runtime<BUFFER_SIZE, CHANNEL_SIZE> {
    let graph = AudioGraph::with_capacity(initial_capacity);
    let context = AudioContext::new(sample_rate);

    Runtime::new(context, graph)
}