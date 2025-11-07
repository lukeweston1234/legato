use crate::{
    engine::buffer::{Buffer, Frame},
    nodes::utils::ring::RingBuffer,
};
use generic_array::{sequence::GenericSequence, ArrayLength, GenericArray};

// TODO: Polyphase and half band filters, SIMD

pub struct RateAdapter2X {}

trait Resample<const N: usize, const M: usize> {
    fn resample(&mut self, ai: &Frame<N>, ao: &mut Frame<M>);
}

struct Upsample2x<C>
where
    C: ArrayLength,
{
    coeffs: Vec<f32>,
    state: GenericArray<RingBuffer, C>,
}

impl<const N: usize, const M: usize, C> Resample<N, M> for Upsample2x<C>
where
    C: ArrayLength,
{
    fn resample(&mut self, ai: &Frame<N>, ao: &mut Frame<M>) {
        debug_assert!(N * 2 == M); // Ensure that we have the correct
        debug_assert!(ai.len() == ao.len() * 2);

        // Zero insert to expand buffer
        for c in 0..C::USIZE {
            let input = ai[c];
            let out = &mut ao[c];
            for n in 0..N {
                out[2 * n] = input[n];
                out[(2 * n) + 1] = 0.0;
            }
        }

        // Naive FIR filter to remove spectral image
        for c in 0..C::USIZE {
            let buffer = &mut self.state[c];

            let input = ai[c];
            let out = &mut ao[c];
            // I don't think the auto-vectorization gods can save me here
            for (n, &x) in input.iter().enumerate() {
                buffer.push(x);
                let mut y = 0.0;
                for (k, &h) in self.coeffs.iter().enumerate() {
                    y += h * buffer.get(k);
                }
                out[n] = y;
            }
        }
    }
}

struct Downsample2X<const N: usize, C>
where
    C: ArrayLength,
{
    coeffs: Vec<f32>,
    state: GenericArray<RingBuffer, C>,
    filtered: GenericArray<Buffer<N>, C>,
}

impl<const N: usize, C> Downsample2X<N, C>
where
    C: ArrayLength,
{
    pub fn new(coeffs: Vec<f32>) -> Self {
        let kernel_len = C::USIZE;
        Self {
            coeffs,
            state: GenericArray::generate(|_| RingBuffer::with_capacity(kernel_len)),
            filtered: GenericArray::generate(|_| Buffer::SILENT),
        }
    }
}

impl<const N: usize, const M: usize, C> Resample<N, M> for Downsample2X<N, C>
where
    C: ArrayLength,
{
    fn resample(&mut self, ai: &Frame<N>, ao: &mut Frame<M>) {
        debug_assert!(N / 2 == M); // Ensure that we have the correct
        debug_assert!(ai.len() == ao.len());

        // Naive FIR filter to remove frequencies above fs/4
        for c in 0..C::USIZE {
            let buffer = &mut self.state[c];

            let input = ai[c];
            let out = &mut self.filtered[c];
            // I don't think the auto-vectorization gods can save me here
            for (n, &x) in input.iter().enumerate() {
                buffer.push(x);
                let mut y = 0.0;
                for (k, &h) in self.coeffs.iter().enumerate() {
                    y += h * buffer.get(k);
                }
                out[n] = y;
            }
        }

        // Decimate by 2
        for c in 0..C::USIZE {
            let input = self.filtered[c];
            let out = &mut ao[c];
            for m in 0..M {
                out[m] = input[m * 2]
            }
        }
    }
}
