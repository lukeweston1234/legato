use crate::engine::{audio_context::AudioContext, buffer::Frame, port::PortedErased};

pub trait Node<const AF: usize, const CF: usize>: PortedErased {
    fn process(
        &mut self,
        ctx: &AudioContext,
        ai: &Frame<AF>,
        ao: &mut Frame<AF>,
        ci: &Frame<CF>,
        co: &mut Frame<CF>,
    );
    fn tick_ctrl(&mut self) {}
}
