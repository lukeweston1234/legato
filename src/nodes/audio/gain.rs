use crate::mini_graph::node::AudioNode;
use crate::mini_graph::buffer::{Frame};

pub struct Gain<const FRAME_SIZE: usize> {
    gain: f32 // Arc<AtomicF32> might be more helpful. If you need an atomic f32 there is an easy trick
}
impl<const N: usize> Gain<N> {
    pub fn new(gain: f32) -> Self {
        Self {
            gain
        }
    }
}
impl <const N: usize, const C: usize> AudioNode<N, C> for Gain<N> {
    #[inline(always)]
    fn process(&mut self, inputs: &[Frame<N, C>], output: &mut Frame<N, C>){
        // This node only takes an input of one stereo buffer.
        let input = inputs[0];
        for n in 0..N { // For ever sample in our frame size
            for c in 0..C { // For ever channel in our frame
                output[c][n] = (input[c][n] * self.gain).clamp(-1.0 , 1.0);
            }
        }
    }
}