use core::fmt;
use core::ops::{Deref, DerefMut};
use generic_array::{ArrayLength, GenericArray};

#[derive(Clone)]
pub struct Buffer<N: ArrayLength> {
    pub data: GenericArray<f32, N>,
}

impl<N: ArrayLength> Buffer<N> {
    pub fn silent() -> Self {
        Self {
            data: GenericArray::default(),
        }
    }
}

impl<N: ArrayLength> Default for Buffer<N> {
    fn default() -> Self {
        Self::silent()
    }
}

impl<N: ArrayLength> From<GenericArray<f32, N>> for Buffer<N> {
    fn from(data: GenericArray<f32, N>) -> Self {
        Self { data }
    }
}

impl<N: ArrayLength> fmt::Debug for Buffer<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.data.as_slice(), f)
    }
}

impl<N: ArrayLength> PartialEq for Buffer<N> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<N: ArrayLength> Deref for Buffer<N> {
    type Target = [f32];
    fn deref(&self) -> &Self::Target {
        self.data.as_slice()
    }
}

impl<N: ArrayLength> DerefMut for Buffer<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data.as_mut_slice()
    }
}

pub type Frame<N> = [Buffer<N>];
