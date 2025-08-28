use crate::engine::{buffer::Buffer, node::Node, port::{Port, PortBehavior, PortedErased}};
use crate::engine::audio_context::AudioContext;

pub struct Stereo {
    ports: StereoPorts
}

impl Default for Stereo {
    fn default() -> Self {
        Self {
            ports: StereoPorts::new()
        }
    }
}


impl<const N: usize> Node<N> for Stereo {
    fn process(&mut self, ctx: &AudioContext, inputs: &[Buffer<N>], outputs: &mut [Buffer<N>]) {
        debug_assert_eq!(inputs.len(), 1);
        debug_assert_eq!(outputs.len(), 2);

        for n in 0..N {
            for c in 0..1 {
                outputs[c][n] = inputs[0][n];
            }
        }
    }
}

struct StereoPorts {
    inputs: [Port;1],
    outputs: [Port; 2]
}
impl StereoPorts {
    pub fn new() -> Self {
        Self {
            inputs: [
                Port {name: "AUDIO", index: 0, behavior: PortBehavior::Default }
            ],
            outputs: [
                Port {name: "L", index: 0, behavior: PortBehavior::Default },
                Port {name: "R", index: 0, behavior: PortBehavior::Default }
            ]
        }
    }
}

impl PortedErased for Stereo {
    fn get_inputs(&self) -> &[Port] {
        &self.ports.inputs    
    }
    fn get_outputs(&self) -> &[Port] {
        &self.ports.outputs
    }
}