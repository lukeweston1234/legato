use std::{marker::PhantomData, ops::Add};
use generic_array::{ArrayLength, GenericArray};
use typenum::{Add1, Sum, Unsigned};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PortBehavior {
    Default, // Input: Take the first sample, Output: Fill the frame
    Sum,
    SumNormalized,
    Mute,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Port {
    pub name: &'static str,
    pub index: usize,
    pub behavior: PortBehavior,
}

impl Default for Port {
    fn default() -> Self {
        Self {
            name: "",
            index: 0,
            behavior: PortBehavior::Mute
        }
    }
}

pub trait Ported<Ai, Ci, O> 
where 
    Ai: Unsigned + Add<Ci>, 
    Ci: Unsigned, 
    O: Unsigned + ArrayLength,
    Sum<Ai, Ci>: Unsigned + ArrayLength
{
    fn get_input_ports(&self) ->  &'static GenericArray<Port, Sum<Ai, Ci>>;
    fn get_output_ports(&self) -> &'static GenericArray<Port, O>;
    fn num_inputs(&self) -> usize { <Sum<Ai, Ci>>::USIZE } 
    fn num_outputs(&self) -> usize { O::USIZE } 
}