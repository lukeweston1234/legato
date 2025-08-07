use crate::mini_graph::node::AudioNode;
use crate::mini_graph::buffer::{Frame};

pub struct Gate {
    is_open: bool
}
impl Gate {
    pub fn new() -> Self {
        Self {
            is_open: false
        }
    }
}
impl<const C: usize, const N: usize> AudioNode<C, N> for Gate {
    #[inline(always)]
    fn process(&mut self, inputs: &[Frame<C, N>], output: &mut Frame<C, N>) {
        if let Some(input) = inputs.get(0) {
            if self.is_open {
                *output = *input;
            }
            else {
                for buf in output.iter_mut() {
                    buf.fill(0.0);
                }
            }
        }
    }
}
