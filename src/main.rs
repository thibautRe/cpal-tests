extern crate cpal;
extern crate tokio;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

mod filter;
mod instrument;
mod oscillator;

use std::collections::VecDeque;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread;

use instrument::Instrument;
use oscillator::Oscillator;

use tokio::io;
use tokio::net::TcpListener;
use tokio::prelude::*;

struct TripleOsc {
    osc1: Oscillator,
    osc2: Oscillator,
    osc3: Oscillator,
    sample_rate: f32,
}

#[derive(Serialize, Deserialize)]
struct TestStruct {
    value: f32,
}

impl Instrument for TripleOsc {
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    fn get_sample_rate(&self) -> f32 {
        self.sample_rate
    }

    fn get_next_value(&mut self) -> f32 {
        // Multiply all fields
        self.osc1.get_value(self.sample_rate)
            * self.osc2.get_value(self.sample_rate)
            * self.osc3.get_value(self.sample_rate)
    }
}

fn main() {
    let device = cpal::default_output_device().expect("Failed to get default output device");
    let format = device
        .default_output_format()
        .expect("Failed to get default output format");
    let event_loop = cpal::EventLoop::new();
    let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
    event_loop.play_stream(stream_id);
    let sample_rate = format.sample_rate.0 as f32;

    // Initialize ring buffers with empty values
    let mut before_filter = VecDeque::<f32>::new();
    before_filter.push_back(0.0);
    before_filter.push_back(0.0);
    let mut after_filter = VecDeque::<f32>::new();
    after_filter.push_back(0.0);
    after_filter.push_back(0.0);

    let mut osc_fm = Oscillator::new(4.0, oscillator::Types::Sawtooth);
    let mut osc1 = Oscillator::new(440.0, oscillator::Types::Square);
    // let mut osc2 = Oscillator::new(100.0, oscillator::Types::Triangle);

    // let mut triple_osc = TripleOsc {
    //     osc1: Oscillator::new(440.0, oscillator::Types::Sine),
    //     osc2: Oscillator::new(110.0, oscillator::Types::Square),
    //     osc3: Oscillator::new(55.0, oscillator::Types::Sawtooth),
    //     sample_rate,
    // };

    let filter_freq_osc = Arc::new(Mutex::new(Oscillator::new(
        2.0,
        oscillator::Types::Sawtooth,
    )));
    let mut test_filter =
        filter::BiquadFilter::new(sample_rate, 200.0, 2.7, filter::BiquadFilterTypes::LowPass);

    let filter_freq_osc = Arc::clone(&filter_freq_osc);

    // Generate the next value
    let mut next_value = || {
        osc1.set_exp_frequency((1.0 + osc_fm.get_value(sample_rate)) * 100.0 + 100.0, 10.0);
        let value = osc1.get_value(sample_rate);
        test_filter
            .set_frequency((filter_freq_osc.lock().unwrap().get_value(sample_rate) + 1.0) * 500.0);

        let filtered = test_filter.get_next_value(
            value,
            before_filter[1],
            before_filter[0],
            after_filter[1],
            after_filter[0],
        );

        before_filter.push_back(value);
        before_filter.pop_front();
        after_filter.push_back(filtered);
        after_filter.pop_front();

        filtered
    };

    let play = || {
        event_loop.run(move |_, data| match data {
            cpal::StreamData::Output {
                buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer),
            } => {
                for sample in buffer.chunks_mut(format.channels as usize) {
                    let value = ((next_value() * 0.5 + 0.5) * f32::from(std::u16::MAX)) as u16;
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            }
            cpal::StreamData::Output {
                buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer),
            } => {
                for sample in buffer.chunks_mut(format.channels as usize) {
                    let value = (next_value() * f32::from(std::i16::MAX)) as i16;
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            }
            cpal::StreamData::Output {
                buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
            } => {
                // around 480
                for sample in buffer.chunks_mut(format.channels as usize) {
                    let value = next_value();
                    // 2 (stereo?)
                    for out in sample.iter_mut() {
                        *out = value;
                    }
                }
            }
            _ => (),
        })
    };

    // let mut record = || {
    //     let sample_rate_wav = 48000;
    //     let time_to_record = 2;
    //     let spec = hound::WavSpec {
    //         channels: 1,
    //         sample_rate: sample_rate_wav,
    //         bits_per_sample: 16,
    //         sample_format: hound::SampleFormat::Int,
    //     };
    //     let mut writer = hound::WavWriter::create("sine.wav", spec).unwrap();

    //     let total_samples = time_to_record * sample_rate_wav;
    //     for _ in 0..total_samples {
    //         let sample = next_value();
    //         let amplitude = f32::from(std::i16::MAX);
    //         writer.write_sample((sample * amplitude) as i16).unwrap();
    //     }
    // };

    // record();

    let addr = "127.0.0.1:6142".parse().unwrap();
    let listener = TcpListener::bind(&addr).unwrap();

    let filter_freq_osc = Arc::clone(&filter_freq_osc);

    let server = listener
        .incoming()
        .map_err(|e| println!("failed to accept socket; error = {:?}", e))
        .for_each(move |socket| {
            println!("New socket: {}", socket.peer_addr().unwrap());
            let (reader, writer) = socket.split();
            let lines = io::lines(BufReader::new(reader));

            let responses = lines.map(|line| {
                let test_struct: TestStruct = serde_json::from_str(&line).unwrap();
                test_struct
            });

            let filter_freq_osc = Arc::clone(&filter_freq_osc);

            let writes = responses.fold(writer, move |writer, line| {
                let mut filter_freq_osc = filter_freq_osc.lock().unwrap();
                filter_freq_osc.set_frequency(line.value);
                let response = String::from("Respones").into_bytes();
                io::write_all(writer, response).map(|(w, _)| w)
            });

            tokio::spawn(writes.then(|_| Ok(())))
        });

    thread::spawn(move || {
        tokio::run(server);
        println!("Done!");
    });

    play();
}
