

pub struct AudioContext {
    sample_rate: f32, // avoiding frequent casting
}
impl AudioContext {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            sample_rate: sample_rate as f32
        }
    }
    #[inline(always)]
    pub fn get_sample_rate(&self) -> f32 {
        self.sample_rate
    }
}
