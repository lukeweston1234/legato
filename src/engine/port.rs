use std::{ops::Add};
use generic_array::{ArrayLength, GenericArray};
use typenum::{Sum, Unsigned, U1, U2};


/// This will determine how ports audio will fan in and out, etc.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PortBehavior {
    Default, // Input: Take the first sample, Output: Fill the frame
    Sum,
    SumNormalized,
    Mute,
}

/// A basic port with a name and index. The port behavior will eventually
/// tell the runtime how to handle things like fan-in, fan-out, summing inputs, etc.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Port {
    pub name: &'static str,
    pub index: usize,
    pub behavior: PortBehavior,
}


pub trait Ported<Ai, Ci, O>
where
    Ai: Unsigned + Add<Ci>,
    Ci: Unsigned,
    O: Unsigned + ArrayLength,
    Sum<Ai, Ci>: Unsigned + ArrayLength,
{
    fn get_inputs(&self) -> &GenericArray<Port, Sum<Ai, Ci>>;
    fn get_outputs(&self) -> &GenericArray<Port, O>;
}


/// A trait allowing us to erase the specific input and output 
/// types to store them more easily.
pub trait PortedErased {
    fn get_inputs(&self) -> &[Port];
    fn get_outputs(&self) -> &[Port];
}

/// Utility type for one channel
pub type Mono = U1;
/// Utility type for two channels
pub type Stereo = U2;