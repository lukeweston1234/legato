use crate::engine::{
    audio_context::AudioContext,
    buffer::Frame,
    node::Node,
    port::{AudioInputPort, AudioOutputPort, ControlInputPort, ControlOutputPort, PortedErased},
};

/// Oversampling node, that takes possession of a boxed node,
/// and upsamples by a const ration K.
///
/// This node uses the common algorithm of filling every K samples
/// with a zero, and filtering to approx. the original value.
///
/// This is useful for operations like distortion, or FM sampling
/// that create sidebands or harmonics higher than the normal
/// audio rate Nyquist frequency. This node will apply the same
/// or a more cost effective process to the control rate as well.
pub struct Oversample<const K: usize, const AF: usize, const CF: usize> {
    node: Box<dyn Node<AF, CF>>,
    // Audio in work buffers
    ai_work_buffer: Vec<Vec<f32>>,
    ao_work_buffer: Vec<Vec<f32>>, // we can heap allocate with K * AF, but we can't on the stack I believe
    ci_work_buffer: Vec<Vec<f32>>,
    co_work_buffer: Vec<Vec<f32>>,
}

impl<const K: usize, const AF: usize, const CF: usize> Oversample<K, AF, CF> {
    /// Constructor for oversampler, you must provide a Boxed Node, and an
    /// upsampling factor K. Likely, this will be hidden behind a type that exposes
    /// two and four times sampling.
    ///
    /// I have not figured out a clean way to get this information at comp time,
    /// so, we will instead use heap allocation to store the work buffers here.
    /// This is needed as these have larger sizes that the already allocated
    /// work buffers in the engine.
    pub fn new(node: Box<dyn Node<AF, CF>>) -> Self {
        // preallocated audio work buffers
        let ai_work_buffer = vec![vec![0.0; AF * K]; node.get_audio_inputs().iter().len()];
        let ao_work_buffer = vec![vec![0.0; CF * K]; node.get_audio_outputs().iter().len()];
        // preallocated control work buffers
        let ci_work_buffer = vec![vec![0.0; CF * K]; node.get_control_inputs().iter().len()];
        let co_work_buffer = vec![vec![0.0; CF * K]; node.get_audio_outputs().iter().len()];
        Self {
            node,
            ai_work_buffer,
            ao_work_buffer,
            ci_work_buffer,
            co_work_buffer,
        }
    }
}

impl<const K: usize, const AF: usize, const CF: usize> Node<AF, CF> for Oversample<K, AF, CF> {
    fn process(
        &mut self,
        ctx: &mut AudioContext<AF>,
        ai: &Frame<AF>,
        ao: &mut Frame<AF>,
        ci: &Frame<CF>,
        co: &mut Frame<CF>,
    ) {
        // Upsample audio and control by factor of K
    }
}

fn oversample(k: usize, input: &[&[f32]], output: &mut [&mut [f32]]) {
    for buffer in output.iter_mut() {
        // Zero pad
        for (i, sample) in buffer.iter_mut().enumerate() {
            // Using a mask for SIMD down the line
            let mask = !(i % k == k - 1) as usize as f32;
            *sample *= mask;
        }
    }
}

impl<const K: usize, const AF: usize, const CF: usize> PortedErased for Oversample<K, AF, CF> {
    fn get_audio_inputs(&self) -> Option<&[AudioInputPort]> {
        self.node.get_audio_inputs()
    }
    fn get_audio_outputs(&self) -> Option<&[AudioOutputPort]> {
        self.node.get_audio_outputs()
    }
    fn get_control_inputs(&self) -> Option<&[ControlInputPort]> {
        self.node.get_control_inputs()
    }
    fn get_control_outputs(&self) -> Option<&[ControlOutputPort]> {
        self.node.get_control_outputs()
    }
}
