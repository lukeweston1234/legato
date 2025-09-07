use crate::engine::runtime::Runtime;
use cpal::{FromSample, SizedSample};

#[inline(always)]
pub fn write_data_cpal<const AF: usize, const CF: usize, const CHANNEL_COUNT: usize, T>(
    output: &mut [T],
    runtime: &mut Runtime<AF, CF, CHANNEL_COUNT>,
) where
    T: SizedSample + FromSample<f64>,
{
    let next_block = runtime.next_block();

    for (frame_index, frame) in output.chunks_mut(CHANNEL_COUNT).enumerate() {
        for (channel, sample) in frame.iter_mut().enumerate() {
            let pipeline_next_frame = &next_block[channel];
            *sample = T::from_sample(pipeline_next_frame[frame_index] as f64);
        }
    }
}
