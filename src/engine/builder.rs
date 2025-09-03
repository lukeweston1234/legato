use crate::{engine::{graph::NodeKey, node::Node, runtime::Runtime}, nodes::{osc::{OscMC, OscMono, OscStereo}, stereo::Stereo, svf::{SvfMono, SvfStereo}}};

// TODO: Port over proc macro from other repo
pub enum Nodes {
    OscMono,
    OscStereo,
    Stereo,
    SvfMono,
    SvfStereo
}

pub trait RuntimeBuilder {
    fn add_node_api(&mut self, node: Nodes) -> NodeKey;
}

impl<const N: usize, const C: usize> RuntimeBuilder for Runtime<N, C> {
    fn add_node_api(&mut self, node: Nodes) -> NodeKey {
        let item: Box<dyn Node<N> + Send + 'static> = match node {
            Nodes::OscMono => Box::new(OscMono::default()),
            Nodes::OscStereo => Box::new(OscStereo::default()),
            Nodes::Stereo => Box::new(Stereo::default()),
            Nodes::SvfMono => Box::new(SvfMono::default()),
            Nodes::SvfStereo => Box::new(SvfStereo::default()),
        };
        self.add_node(item)
    }
}