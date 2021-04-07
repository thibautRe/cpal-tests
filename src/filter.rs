use std::collections::VecDeque;
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
    HighPass,
}

/// Bi-quadratic filter, used for lowpass, highpass and other types
/// of "simple" filters
///
/// ```
/// BiquadFilter::new(
///     sample_rate,
///     2000.0,
///     2.7,
///     BiquadFilterTypes::LowPass,
/// ))
#[allow(non_snake_case)]
pub struct BiquadFilter {
    sample_rate: f32,
    coefficients: BiquadFilterCoefficients,
    frequency: f32,
    Q: f32,
    filter_type: BiquadFilterTypes,

    /// Ring buffer for input values
    input: VecDeque<f32>,
    /// Ring buffer for output values
    output: VecDeque<f32>,
}

impl BiquadFilter {
    /// Creates a new filter based on a type, a frequency and a quality factor
    #[allow(non_snake_case)]
    pub fn new(sample_rate: f32, frequency: f32, Q: f32, filter_type: BiquadFilterTypes) -> Self {
        let mut input = VecDeque::<f32>::with_capacity(2);
        input.push_back(0.0);
        input.push_back(0.0);
        let mut output = VecDeque::<f32>::with_capacity(2);
        output.push_back(0.0);
        output.push_back(0.0);

        Self {
            sample_rate,
            frequency,
            Q,
            filter_type,
            coefficients: Self::calculate_coefficients(sample_rate, frequency, Q, filter_type),
            input,
            output,
        }
    }

    /// Returns the next filtered value based on 5 inputs: current, previous and last previous
    /// entry value, and previous and last previous output value of the filter.
    /// https://dxr.mozilla.org/mozilla-central/source/dom/media/webaudio/blink/Biquad.h#100
    pub fn get_next_value(&mut self, input: f32) -> f32 {
        let c = &self.coefficients;
        // Calculate the next output based on previous outputs and inputs of the filter
        let next_output =
            c.b0 / c.a0 * input + c.b1 / c.a0 * self.input[1] + c.b2 / c.a0 * self.input[0]
                - c.a1 / c.a0 * self.output[1]
                - c.a2 / c.a0 * self.output[0];

        // Save output and input in the queues
        self.input.pop_front();
        self.input.push_back(input);
        self.output.pop_front();
        self.output.push_back(next_output);
        next_output
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
        self.update_coefficients();
    }

    pub fn get_frequency(&self) -> f32 {
        self.frequency
    }

    #[allow(non_snake_case)]
    pub fn set_Q(&mut self, Q: f32) {
        self.Q = Q;
        self.update_coefficients();
    }

    #[allow(non_snake_case)]
    pub fn get_Q(&self) -> f32 {
        self.Q
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
        let w0cos = w0.cos();
        match filter_type {
            BiquadFilterTypes::LowPass => BiquadFilterCoefficients {
                b0: (1.0 - w0cos) / 2.0,
                b1: 1.0 - w0cos,
                b2: (1.0 - w0cos) / 2.0,
                a0: 1.0 + alpha,
                a1: -2.0 * w0cos,
                a2: 1.0 - alpha,
            },
            BiquadFilterTypes::HighPass => BiquadFilterCoefficients {
                b0: (1.0 + w0cos) / 2.0,
                b1: -(1.0 + w0cos),
                b2: (1.0 + w0cos) / 2.0,
                a0: 1.0 + alpha,
                a1: -2.0 * w0cos,
                a2: 1.0 - alpha,
            },
        }
    }
}
