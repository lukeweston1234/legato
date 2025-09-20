use std::{cell::UnsafeCell, sync::Arc};

use generic_array::{sequence::GenericSequence, ArrayLength, GenericArray};
use typenum::U0;

use crate::{
    engine::{
        audio_context::AudioContext,
        buffer::Frame,
        node::Node,
        port::{
            AudioInputPort, AudioOutputPort, ControlInputPort, ControlOutputPort, Mono,
            PortedErased, Ports, Stereo, UpsampleAlg,
        },
    },
    nodes::utils::{generate_audio_inputs, generate_audio_outputs},
};

pub fn lerp(v0: f32, v1: f32, t: f32) -> f32 {
    (1.0 - t) * v0 + t * v1
}

#[derive(Clone)]
pub struct DelayLine<const N: usize, C>
where
    C: ArrayLength,
{
    buffers: GenericArray<Vec<f32>, C>,
    capacity: usize,
    write_pos: usize,
}

impl<const N: usize, C> DelayLine<N, C>
where
    C: ArrayLength,
{
    pub fn new(capacity: usize) -> Self {
        let buffers = GenericArray::generate(|_| vec![0.0; capacity]);
        Self {
            buffers,
            capacity: capacity,
            write_pos: 0,
        }
    }
    #[inline(always)]
    pub fn get_write_pos(&self) -> &usize {
        &self.write_pos
    }
    #[inline(always)]
    pub fn write_block(&mut self, block: &Frame<N>) {
        // We're assuming single threaded, with the graph firing in order, so no aliasing writes
        // Our first writing block is whatever capacity is leftover from the writing position
        // Our maximum write size is the block N
        let first_write_size = (self.capacity - self.write_pos).min(N);
        // Our second write size is whatever leftover from N we still have
        let second_write_size = N - first_write_size;

        for c in 0..C::USIZE {
            let buf = &mut self.buffers[c];
            buf[self.write_pos..self.write_pos + first_write_size]
                .copy_from_slice(&block[c][0..first_write_size]);
            // TODO: Maybe some sort of mask?
            if second_write_size > 0 {
                buf[0..second_write_size].copy_from_slice(
                    &block[c][first_write_size..first_write_size + second_write_size],
                );
            }
        }
        self.write_pos = (self.write_pos + N) % self.capacity;
    }
    // Note: both of these functions use f32 sample indexes, as we allow for interpolated values
    #[inline(always)]
    pub fn get_delay_linear_interp(&self, channel: usize, offset: f32) -> f32 {
        // Get the remainder of the difference of the write position and fractional sample index we need
        let read_pos = (self.write_pos as f32 - offset).rem_euclid(self.capacity as f32);

        let pos_floor = read_pos.floor() as usize;
        let next_sample = (pos_floor + 1) % self.capacity; // TODO: can we have some sort of mask if we make the delay a power of 2?

        let buffer = &self.buffers[channel];

        lerp(
            buffer[pos_floor],
            buffer[next_sample],
            read_pos - pos_floor as f32,
        )
    }
}

pub struct DelayWrite<const AF: usize, Ai>
where
    Ai: ArrayLength,
{
    delay_line: Arc<UnsafeCell<DelayLine<AF, Ai>>>,
    ports: Ports<Ai, U0, U0, U0>,
}
impl<const AF: usize, Ai> DelayWrite<AF, Ai>
where
    Ai: ArrayLength,
{
    pub fn new(delay_line: Arc<UnsafeCell<DelayLine<AF, Ai>>>) -> Self {
        Self {
            delay_line,
            ports: Ports {
                audio_inputs: Some(generate_audio_inputs()),
                audio_outputs: None,
                control_inputs: None,
                control_outputs: None,
            },
        }
    }
}

impl<const AF: usize, const CF: usize, Ai> Node<AF, CF> for DelayWrite<AF, Ai>
where
    Ai: ArrayLength,
{
    fn process(
        &mut self,
        _: &AudioContext,
        ai: &Frame<AF>,
        _: &mut Frame<AF>,
        _: &Frame<CF>,
        _: &mut Frame<CF>,
    ) {
        // Single threaded, no aliasing read/writes in the graph. Reference counted so no leaks. Hopefully safe.
        unsafe {
            (&mut *self.delay_line.get()).write_block(ai);
        }
    }
}

impl<const AF: usize, Ai> PortedErased for DelayWrite<AF, Ai>
where
    Ai: ArrayLength,
{
    fn get_audio_inputs(&self) -> Option<&[crate::engine::port::AudioInputPort]> {
        self.ports.get_audio_inputs()
    }
    fn get_audio_outputs(&self) -> Option<&[crate::engine::port::AudioOutputPort]> {
        self.ports.get_audio_outputs()
    }
    fn get_control_inputs(&self) -> Option<&[crate::engine::port::ControlInputPort]> {
        self.ports.get_control_inputs()
    }
    fn get_control_outputs(&self) -> Option<&[crate::engine::port::ControlOutputPort]> {
        self.ports.get_control_outputs()
    }
}

pub struct DelayRead<const AF: usize, Ao>
where
    Ao: ArrayLength,
{
    delay_line: Arc<UnsafeCell<DelayLine<AF, Ao>>>,
    delay_times: GenericArray<f32, Ao>, // Different times for each channel if desired
    ports: Ports<U0, Ao, U0, U0>,
}
impl<const AF: usize, Ao> DelayRead<AF, Ao>
where
    Ao: ArrayLength,
{
    pub fn new(delay_line: Arc<UnsafeCell<DelayLine<AF, Ao>>>) -> Self {
        Self {
            delay_line,
            delay_times: GenericArray::generate(|_| 900.0), // Default 900ms for now
            ports: Ports {
                audio_inputs: None,
                audio_outputs: Some(generate_audio_outputs()),
                control_inputs: None, // TODO: modulate delay times per channel
                control_outputs: None,
            },
        }
    }
}

impl<const AF: usize, const CF: usize, Ao> Node<AF, CF> for DelayRead<AF, Ao>
where
    Ao: ArrayLength,
{
    fn process(
        &mut self,
        ctx: &AudioContext,
        _: &Frame<AF>,
        ao: &mut Frame<AF>,
        _: &Frame<CF>,
        _: &mut Frame<CF>,
    ) {
        debug_assert_eq!(Ao::USIZE, ao.len());
        unsafe {
            let delay_line = &(*self.delay_line.get());
            for n in 0..AF {
                for c in 0..Ao::USIZE {
                    let offset =
                        ((self.delay_times[c] / 1000.0) * ctx.get_sample_rate()) + (AF - n) as f32;
                    // Read delay line based on per channel delay time. Must cast to sample index.
                    ao[c][n] = delay_line.get_delay_linear_interp(c, offset)
                }
            }
        }
    }
}

impl<const AF: usize, Ao> PortedErased for DelayRead<AF, Ao>
where
    Ao: ArrayLength,
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

unsafe impl<const AF: usize, C> Send for DelayRead<AF, C> where C: ArrayLength {}
unsafe impl<const AF: usize, C> Sync for DelayRead<AF, C> where C: ArrayLength {}

unsafe impl<const AF: usize, C> Send for DelayWrite<AF, C> where C: ArrayLength {}
unsafe impl<const AF: usize, C> Sync for DelayWrite<AF, C> where C: ArrayLength {}

pub type DelayReadMono<const AF: usize> = DelayRead<AF, Mono>;
pub type DelayReadStereo<const AF: usize> = DelayRead<AF, Stereo>;

pub type DelayWriteMono<const AF: usize> = DelayWrite<AF, Mono>;
pub type DelayWriteStereo<const AF: usize> = DelayWrite<AF, Stereo>;
