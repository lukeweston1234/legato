use crate::mini_graph::node::AudioNode;
use crate::mini_graph::buffer::{Frame};

pub struct HardClipper<const FRAME_SIZE: usize> {
    limit: f32,
}
impl<const N: usize> HardClipper<N> {
    pub fn new(limit: f32) -> Self {
        Self {
            limit
        }
    }
}
impl<const N: usize, const C: usize> AudioNode<N,C> for HardClipper<N> {
    fn process(&mut self, inputs: &[Frame<N, C>], output: &mut Frame<N, C>){
        if (self.limit > 1.0) {
            panic!("Invalid limit!!")
        }
        let input = inputs[0];
        for n in 0..N {
            for c in 0..C { 
                output[c][n] = input[c][n].clamp(-1.0 * self.limit , self.limit);
            }
        }
    }
}