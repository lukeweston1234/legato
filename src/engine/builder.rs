use std::sync::Arc;

use arc_swap::ArcSwapOption;
use generic_array::GenericArray;

use crate::{
    engine::{graph::NodeKey, node::Node, runtime::Runtime},
    nodes::audio::{
        osc::{OscMono, OscStereo},
        sampler::{SamplerMono, SamplerStereo},
        stereo::Stereo,
    },
};

use typenum::{U1, U2};

// TODO: Port over proc macro from other repo
pub enum Nodes {
    OscMono,
    OscStereo,
    Stereo,
    SamplerMono,
    SamplerStereo,
    // SvfMono,
    // SvfStereo
}

pub enum NodeProps {
    SamplerMono {
        sample: Arc<ArcSwapOption<GenericArray<Vec<f32>, U1>>>,
    },
    SamplerStereo {
        sample: Arc<ArcSwapOption<GenericArray<Vec<f32>, U2>>>,
    },
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum BuilderError {
    InvalidProps,
}

pub trait RuntimeBuilder {
    fn add_node_api(
        &mut self,
        node: Nodes,
        props: Option<NodeProps>,
    ) -> Result<NodeKey, BuilderError>;
}

impl<const AF: usize, const CF: usize, const C: usize> RuntimeBuilder for Runtime<AF, CF, C> {
    fn add_node_api(
        &mut self,
        node: Nodes,
        props: Option<NodeProps>,
    ) -> Result<NodeKey, BuilderError> {
        let node_created: Result<Box<dyn Node<AF, CF> + Send + 'static>, BuilderError> = match node
        {
            Nodes::OscMono => Ok(Box::new(OscMono::default())),
            Nodes::OscStereo => Ok(Box::new(OscStereo::default())),
            Nodes::Stereo => Ok(Box::new(Stereo::default())),
            Nodes::SamplerMono => {
                if let Some(item) = props {
                    match item {
                        NodeProps::SamplerMono { sample } => Ok(Box::new(SamplerMono::new(sample))),
                        _ => Err(BuilderError::InvalidProps),
                    }
                } else {
                    Err(BuilderError::InvalidProps)
                }
            }
            Nodes::SamplerStereo => {
                if let Some(item) = props {
                    match item {
                        NodeProps::SamplerStereo { sample } => {
                            Ok(Box::new(SamplerStereo::new(sample)))
                        }
                        _ => Err(BuilderError::InvalidProps),
                    }
                } else {
                    Err(BuilderError::InvalidProps)
                }
            }
        };
        match node_created {
            Ok(node) => Ok(self.add_node(node)),
            Err(err) => Err(err),
        }
    }
}
