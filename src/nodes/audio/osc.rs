use crate::mini_graph::node::AudioNode;
use crate::mini_graph::buffer::{Frame};
use crate::mini_graph::bang::Bang;

pub enum Wave {
    SinWave,
    SawWave,
    TriangleWave,
    SquareWave,
}

pub struct Oscillator<const BUFFER_SIZE: usize> {
    freq: f32,
    sample_rate: f32,
    phase: f32,
    wave: Wave
}

impl<const N: usize> Oscillator<N> {
    pub fn new(freq: f32, sample_rate: u32, phase: f32, wave: Wave) -> Self {
        Self {
            freq,
            sample_rate: sample_rate as f32,
            phase,
            wave
        }
    }
    pub fn set_wave_form(&mut self, wave: Wave){
        self.wave = wave;
    }
    #[inline(always)]
    fn tick_osc(&mut self) -> f32 {
        let sample = match self.wave {
            Wave::SinWave => sin_amp_from_phase(&self.phase),
            Wave::SawWave => saw_amp_from_phase(&self.phase),
            Wave::SquareWave => square_amp_from_phase(&self.phase),
            Wave::TriangleWave => triangle_amp_from_phase(&self.phase),
        };
        self.phase += self.freq / self.sample_rate as f32;
        self.phase -= (self.phase >= 1.0) as u32 as f32; 
        sample
    }
}
impl<const N: usize, const C: usize> AudioNode<N, C> for Oscillator<N> {
    #[inline(always)]
    fn process(&mut self, _: &[Frame<N, C>], output: &mut Frame<N, C>){
        for i in 0..N {
            let sample = self.tick_osc();
            for buf in output.iter_mut() {
                buf[i] = sample;
            }
        }
    }
    
    fn handle_bang(&mut self, inputs: &[Bang], _: &mut Bang) {
        if let Some(bang) = inputs.get(0) {
            match bang {
                Bang::BangF32(freq) => self.freq = *freq,
                _ => ()
            }
        }
    }
}

#[inline(always)]
fn sin_amp_from_phase(phase: &f32) -> f32 {
    (*phase * 2.0 * std::f32::consts::PI).sin()
}

#[inline(always)]
fn saw_amp_from_phase(phase: &f32) -> f32 {
    *phase * 2.0 - 1.0
}

#[inline(always)]
fn triangle_amp_from_phase(phase: &f32) -> f32 {
    2.0 * ((-1.0 + (*phase * 2.0)).abs() - 0.5)
}

#[inline(always)]
fn square_amp_from_phase(phase: &f32) -> f32 {
    match *phase <= 0.5 {
        true => 1.0,
        false => -1.0,
    }
}