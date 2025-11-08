use generic_array::ArrayLength;

use crate::engine::{audio_context::AudioContext, buffer::Frame, port::PortedErased};

pub trait Node<AF, CF>: PortedErased
where
    AF: ArrayLength,
    CF: ArrayLength,
{
    fn process(
        &mut self,
        ctx: &mut AudioContext<AF>,
        ai: &Frame<AF>,
        ao: &mut Frame<AF>,
        ci: &Frame<CF>,
        co: &mut Frame<CF>,
    );
    fn tick_ctrl(&mut self) {}
}
