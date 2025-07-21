use crate::mini_graph::bang::Bang;
use crate::mini_graph::node::Node;
use crate::mini_graph::buffer::Frame;

enum Stage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

pub struct ADSR<const N: usize, const C: usize> {
    // envelope parameters, in seconds:
    attack_time:  f32,
    decay_time:   f32,
    sustain_time: f32,     // if you want a hold duration
    release_time: f32,

    // envelope levels:
    sustain_level: f32,    // 0.0 … 1.0

    // dynamic state:
    stage: Stage,
    time_in_stage:    f32,
    release_start_level: f32,

    sample_rate: f32,
}

impl<const N: usize, const C: usize> ADSR<N, C> {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            attack_time:   0.1,   // e.g. 10 ms
            decay_time:    0.2,    // e.g. 200 ms
            sustain_time:  0.0,    // 0 for infinite sustain until note-off
            release_time:  0.2,    // e.g. 500 ms

            sustain_level: 0.1,    // 70% amplitude

            stage: Stage::Idle,
            time_in_stage: 0.0,
            release_start_level: 0.0,

            sample_rate: sample_rate as f32,
        }
    }

    fn note_on(&mut self) {
        self.stage = Stage::Attack;
        self.time_in_stage = 0.0;
    }

    fn note_off(&mut self) {
        // capture whatever level we’re at, then go to release
        let current = self.current_level();
        self.release_start_level = current;
        self.stage = Stage::Release;
        self.time_in_stage = 0.0;
    }

    /// compute current envelope level *before* advancing time
    fn current_level(&self) -> f32 {
        match self.stage {
            Stage::Idle    => 0.0,
            Stage::Attack  => (self.time_in_stage / self.attack_time).min(1.0),
            Stage::Decay   => {
                let t = (self.time_in_stage / self.decay_time).min(1.0);
                1.0 + t * (self.sustain_level - 1.0) // lerp(1.0, sustain_level, t)
            }
            Stage::Sustain => self.sustain_level,
            Stage::Release => {
                let t = (self.time_in_stage / self.release_time).min(1.0);
                self.release_start_level * (1.0 - t) // lerp(release_start, 0.0, t)
            }
        }
    }
}

impl<const N: usize, const C: usize> Node<N, C> for ADSR<N, C> {
    fn process(&mut self, inputs: &[Frame<N, C>], output: &mut Frame<N, C>) {
        let dt = 1.0 / self.sample_rate;
        let input = inputs[0];

        for n in 0..N {
            // 1) get current envelope level
            let level = self.current_level();

            // 2) write out
            for c in 0..C {
                output[c][n] = input[c][n] * level;
            }

            // 3) advance timer and handle stage transitions
            self.time_in_stage += dt;
            match self.stage {
                Stage::Attack if self.time_in_stage >= self.attack_time => {
                    self.stage = Stage::Decay;
                    self.time_in_stage = 0.0;
                }
                Stage::Decay if self.time_in_stage >= self.decay_time => {
                    if self.sustain_time > 0.0 {
                        self.stage = Stage::Sustain;
                        self.time_in_stage = 0.0;
                    } else {
                        // if no sustain-time, stay at sustain level until note-off
                        self.stage = Stage::Sustain;
                    }
                }
                Stage::Sustain if self.sustain_time > 0.0
                    && self.time_in_stage >= self.sustain_time =>
                {
                    // auto-release after a fixed hold
                    self.note_off();
                }
                Stage::Release if self.time_in_stage >= self.release_time => {
                    self.stage = Stage::Idle;
                    self.time_in_stage = 0.0;
                }
                _ => {}
            }
        }
    }

    fn handle_bang(&mut self, inputs: &[Bang], _: &mut Bang) {
        for &bang in inputs {
            if let Bang::Bang = bang {
                // you probably want distinct on/off bangs; adjust as needed:
                if matches!(self.stage, Stage::Idle) {
                    self.note_on();
                } else {
                    self.note_off();
                }
            }
        }
    }
}
