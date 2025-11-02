use generic_array::{ArrayLength, GenericArray};
use typenum::U0;

use crate::engine::{audio_context::AudioContext, buffer::Frame, node::Node, port::*};

pub struct FIR<C>
where
    C: ArrayLength,
{
    kernel: Vec<f32>,
    remainder: GenericArray<Vec<f32>, C>,
    ports: Ports<C, C, U0, U0>,
}

impl<C, const AF: usize, const CF: usize> Node<AF, CF> for FIR<C>
where
    C: ArrayLength,
{
    fn process(
        &mut self,
        ctx: &mut AudioContext<AF>,
        ai: &Frame<AF>,
        ao: &mut Frame<AF>,
        ci: &Frame<CF>,
        co: &mut Frame<CF>,
    ) {
    }
}

impl<C> PortedErased for FIR<C>
where
    C: ArrayLength,
{
    fn get_audio_inputs(&self) -> Option<&[AudioInputPort]> {
        self.ports.get_audio_inputs()
    }
    fn get_audio_outputs(&self) -> Option<&[AudioOutputPort]> {
        self.ports.get_audio_outputs()
    }
    fn get_control_inputs(&self) -> Option<&[ControlInputPort]> {
        self.ports.get_control_inputs()
    }
    fn get_control_outputs(&self) -> Option<&[ControlOutputPort]> {
        self.ports.get_control_outputs()
    }
}
