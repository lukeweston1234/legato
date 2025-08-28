use std::ops::Add;

use generic_array::ArrayLength;
use typenum::{Sum, Unsigned};

use crate::engine::{audio_context::AudioContext, buffer::Frame, port::PortedErased,};

pub trait Node<const N: usize>: PortedErased {
    fn process(&mut self, ctx: &AudioContext, inputs: &Frame<N>, output: &mut Frame<N>){}
}

