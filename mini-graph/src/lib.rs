// pub mod mini_graph;
// pub mod nodes;
// pub mod utils;

use core::fmt;
use core::ops::{Deref, DerefMut};

use mini_graph_macros::Port;

pub type Frame<const BUFFER_SIZE: usize, const CHANNEL_COUNT: usize> = [Buffer<BUFFER_SIZE>; CHANNEL_COUNT];
#[derive(Clone, Copy)]
pub struct Buffer<const BUFFER_SIZE: usize> {
    data: [f32; BUFFER_SIZE],
}

impl<const N: usize> Buffer<N> {
    pub const SILENT: Self = Buffer { data: [0.0; N] };
}

impl<const N: usize> Default for Buffer<N> {
    fn default() -> Self {
        Self::SILENT
    }
}

impl<const N: usize> From<[f32; N]> for Buffer<N> {
    fn from(data: [f32; N]) -> Self {
        Buffer { data }
    }
}

impl<const N: usize> fmt::Debug for Buffer<N> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.data[..], f)
    }
}

impl<const N: usize> PartialEq for Buffer<N> {
    fn eq(&self, other: &Self) -> bool {
        self[..] == other[..]
    }
}

impl<const N: usize> Deref for Buffer<N> {
    type Target = [f32];
    fn deref(&self) -> &Self::Target {
        &self.data[..]
    }
}

impl<const N: usize> DerefMut for Buffer<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data[..]
    }
}

#[derive(Debug, PartialEq)]
pub enum PortError {
    InvalidPort
}

struct AudioContext {
    sample_rate: f32,
    frame_size: usize,
    channels: usize
}

pub trait Port {
    fn into_index(&self) -> usize;
    fn from_index(index: usize) -> Result<Self, PortError> where Self: Sized;
}

trait Node<const N: usize, const C: usize> {
    type Inputs: Port;

    fn process(&mut self, ctx: &AudioContext, inputs: &[Frame<N, C>], output: &mut Frame<N, C>){}
}

#[derive(Port, PartialEq, Debug)]
enum OscillatorInputs {
    Freq,
    FM,
}

struct Oscillator {
    phase: f32,
}
impl<const N: usize, const C: usize> Node<N, C> for Oscillator {
    type Inputs = OscillatorInputs;

    fn process(&mut self, ctx: &AudioContext, inputs: &[Frame<N, C>], output: &mut Frame<N, C>) {
        let freq = inputs.get(OscillatorInputs::Freq.into_index());
        let fm = inputs.get(OscillatorInputs::FM.into_index());


    }
}




