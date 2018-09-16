extern crate rand;

use self::rand::prelude::*;
use std::f32::consts::PI;

#[allow(dead_code)]
pub enum Types {
    Sine,
    Square,
    Sawtooth,
    Triangle,
    Random,
    /// The custom type takes a Box of a closure that takes one parameter: the phase
    /// of the oscillator, between 0 and 2*PI
    /// Types::Custom(Box::new(|phase: f32| 0.5 * phase.sin())),
    Custom(Box<Fn(f32) -> f32 + Sync + Send>),
}

pub struct Oscillator {
    shape: Types,
    frequency: f32,
    phase: f32,
}

impl Oscillator {
    pub fn new(frequency: f32, shape: Types) -> Self {
        Self {
            shape,
            frequency,
            phase: 0.0,
        }
    }

    /// Returns the phase (between 0 and 2PI) of the oscillator
    fn update(&mut self, sample_rate: f32) -> f32 {
        self.phase += (2.0 * PI * self.frequency) / sample_rate;

        if self.phase >= 2.0 * PI {
            self.phase -= 2.0 * PI;
        }
        self.phase
    }

    /// Returns the value of the sample (between -1 and 1)
    pub fn get_value(&mut self, sample_rate: f32) -> f32 {
        self.update(sample_rate);
        let value = match self.shape {
            Types::Sine => 1.0 * self.phase.sin(),
            Types::Square => {
                if self.phase < PI {
                    1.0
                } else {
                    -1.0
                }
            }
            Types::Sawtooth => 1.0 - self.phase / PI,
            Types::Triangle => {
                if self.phase < PI {
                    -1.0 + 2.0 * self.phase / PI
                } else {
                    3.0 - 2.0 * self.phase / PI
                }
            }
            Types::Random => rand::thread_rng().gen_range(-1.0, 1.0),
            Types::Custom(ref generator) => generator(self.phase),
        };

        // Clamp the value
        value.max(-1.0).min(1.0)
    }

    pub fn set_frequency(&mut self, target_frequency: f32) {
        self.frequency = target_frequency;
    }

    pub fn set_exp_frequency(&mut self, target_frequency: f32, exp: f32) {
        let adapted_exp = exp / 1000.0;
        self.frequency = self.frequency * (1.0 - adapted_exp) + target_frequency * adapted_exp;
    }
}
