pub struct AudioContext {
    sample_rate: f32, // avoiding frequent casting
    control_rate: f32,
}

impl AudioContext {
    pub fn new(sample_rate: f32, control_rate: f32) -> Self {
        Self {
            sample_rate: sample_rate,
            control_rate: control_rate,
        }
    }
    #[inline(always)]
    pub fn get_sample_rate(&self) -> f32 {
        self.sample_rate
    }
    #[inline(always)]
    pub fn get_control_rate(&self) -> f32 {
        self.control_rate
    }
}
