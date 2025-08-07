use crate::mini_graph::node::AudioNode;
use crate::mini_graph::buffer::{Frame};

#[derive(Default)]
pub struct Mixer<const N: usize> {}

impl<const N: usize, const C: usize> AudioNode<N, C> for Mixer<N> {
    #[inline(always)]
    fn process(&mut self, inputs: &[Frame<N, C>], output: &mut Frame<N, C>){
        for n in 0..N {
            for c in 0..C {
                let mut sum = 0.0;
                for input in inputs {
                    sum += input[c][n]
                }
                output[c][n] = (sum / inputs.len() as f32).clamp(-1.0, 1.0);
            }
        }
    }
}