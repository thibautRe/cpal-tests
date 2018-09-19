use std::f32::consts::PI;

struct BiquadFilterCoefficients {
    a0: f32,
    a1: f32,
    a2: f32,
    b0: f32,
    b1: f32,
    b2: f32,
}

#[derive(Copy, Clone)]
pub enum BiquadFilterTypes {
    LowPass,
}

#[allow(non_snake_case)]
pub struct BiquadFilter {
    sample_rate: f32,
    coefficients: BiquadFilterCoefficients,
    frequency: f32,
    Q: f32,
    filter_type: BiquadFilterTypes,
}

impl BiquadFilter {
    // Creates a new filter based on a type, a frequency and a quality factor
    #[allow(non_snake_case)]
    pub fn new(sample_rate: f32, frequency: f32, Q: f32, filter_type: BiquadFilterTypes) -> Self {
        Self {
            sample_rate,
            frequency,
            Q,
            filter_type,
            coefficients: Self::calculate_coefficients(sample_rate, frequency, Q, filter_type),
        }
    }

    /// Returns the next filtered value based on 5 inputs: current, previous and last previous
    /// entry value, and previous and last previous output value of the filter.
    /// https://dxr.mozilla.org/mozilla-central/source/dom/media/webaudio/blink/Biquad.h#100
    pub fn get_next_value(&self, x: f32, x1: f32, x2: f32, y1: f32, y2: f32) -> f32 {
        let c = &self.coefficients;
        c.b0 / c.a0 * x + c.b1 / c.a0 * x1 + c.b2 / c.a0 * x2 - c.a1 / c.a0 * y1 - c.a2 / c.a0 * y2
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
        self.update_coefficients();
    }

    fn update_coefficients(&mut self) {
        self.coefficients =
            Self::calculate_coefficients(self.sample_rate, self.frequency, self.Q, self.filter_type)
    }

    /// Calculate coefficients
    /// Based on http://www.musicdsp.org/files/Audio-EQ-Cookbook.txt
    #[allow(non_snake_case)]
    fn calculate_coefficients(
        sample_rate: f32,
        frequency: f32,
        Q: f32,
        filter_type: BiquadFilterTypes,
    ) -> BiquadFilterCoefficients {
        let w0 = 2.0 * PI * frequency / sample_rate;
        let alpha = w0.sin() / (2.0 * Q);
        match filter_type {
            BiquadFilterTypes::LowPass => BiquadFilterCoefficients {
                b0: (1.0 - w0.cos()) / 2.0,
                b1: 1.0 - w0.cos(),
                b2: (1.0 - w0.cos()) / 2.0,
                a0: 1.0 + alpha,
                a1: -2.0 * w0.cos(),
                a2: 1.0 - alpha,
            },
        }
    }
}
