use std::{
    collections::HashMap,
    ops::{Add, Mul},
};

use legato_core::engine::{builder::AddNode, node::FrameSize};
use typenum::{Prod, U2};

use crate::ast::{Object, Value};

/// ValidationError covers logical issues
/// when lowering from the AST to the IR.
///
/// Typically, these might be bad parameters,
/// bad values, nodes that don't exist, etc.
pub enum ValidationError {
    NodeNotFound(String),
    InvalidParameters(String),
}

/// Convenience struct to help lower from Objects
/// to parameters that can create nodes and pipes.
pub struct Params<'a>(pub &'a Object);

impl<'a> Params<'a> {
    pub fn get_f32(&self, key: &str) -> Option<f32> {
        match self.0.get(key) {
            Some(Value::F32(x)) => Some(*x),
            Some(Value::I32(x)) => Some(*x as f32),
            Some(Value::U32(x)) => Some(*x as f32),
            _ => None,
        }
    }

    pub fn get_str(&self, key: &str) -> Option<&str> {
        match self.0.get(key) {
            Some(Value::Str(s)) => Some(s),
            Some(Value::Ident(i)) => Some(i),
            _ => None,
        }
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        match self.0.get(key) {
            Some(Value::Bool(b)) => Some(*b),
            _ => None,
        }
    }

    pub fn get_array(&self, key: &str) -> Option<&Vec<Value>> {
        match self.0.get(key) {
            Some(Value::Array(v)) => Some(v),
            _ => None,
        }
    }
    pub fn validate() {
        todo!()
    }
}

/// A node registry trait that let's users extend the graph logic
/// to make their own node namespaces. For example, you could make a
/// reverb namespace that has a bunch of primitives you might need,
/// or you could make a physics namespace with physics logic.
trait NodeRegistry<AF, CF>
where
    AF: FrameSize + Mul<U2>,
    Prod<AF, U2>: FrameSize,
    CF: FrameSize,
{
    fn lower_to_ir(
        name: String,
        params: Option<&Params>,
    ) -> Result<AddNode<AF, CF>, ValidationError>;
}

/// The default container of node registries.
///
/// Users can make their own registries with their
/// own pairs. This means that when you're using
/// the graph, you can choose which namespaces
/// and nodes are in there, extend it on your own, etc.
///
/// To do this yourself, implement the NodeRegistry trait.
/// You can then extend the Legato registry container,
/// or make your own at a later time.
struct LegatoRegistryContainer<AF, CF>
where
    AF: FrameSize + Mul<U2>,
    Prod<AF, U2>: FrameSize,
    CF: FrameSize,
{
    namespaces: HashMap<String, Box<dyn NodeRegistry>>,
}

impl LegatoRegistryContainer<AF, CF>
where
    AF: FrameSize + Mul<U2>,
    Prod<AF, U2>: FrameSize,
    CF: FrameSize,
{
    pub fn new() -> Self {
        let mut namespaces = HashMap::new();
        namespaces.insert(String::from("audio"), Box::new(AudioRegistry));
        Self { namespaces }
    }
}

/// One of the default registries, audio deals
/// with common audio effects. This may be renamed
/// in the future.
struct AudioRegistry<AF, CF>
where
    AF: FrameSize + Mul<U2>,
    Prod<AF, U2>: FrameSize,
    CF: FrameSize, {}

impl<AF, CF> NodeRegistry for AudioRegistry<AF, CF>
where
    AF: FrameSize + Mul<U2>,
    Prod<AF, U2>: FrameSize,
    CF: FrameSize,
{
    fn lower_to_ir(
        &self,
        name: String,
        params: Option<&Params>,
    ) -> Result<AddNode<AF, CF, ValidationError>> {
        match name {
            "sine_mono" => (),
            "sine_stereo" => (),
            "stereo" => (),
            _ => ValidationError::NodeNotFound(format!("Could not find node with name {}", name)),
        }
    }
}
