extern crate cpal;
extern crate hound;
extern crate tokio;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

mod filter;
mod instrument;
mod oscillator;

use std::collections::{HashMap, VecDeque};
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::thread;

use instrument::{Instrument, InstrumentSetParameter, OutputNode};
use oscillator::Oscillator;

use tokio::io;
use tokio::net::TcpListener;
use tokio::prelude::*;

struct TripleOsc {
    osc1: Oscillator,
    osc2: Oscillator,
    osc3: Oscillator,
}

#[derive(Serialize, Deserialize, Debug)]
enum RootDataTargets {
    OutputBuffer,
}

#[derive(Serialize, Deserialize, Debug)]
struct RootDataMessage {
    target: RootDataTargets,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
enum MessageTypes {
    InstrumentSetParameter(String, instrument::InstrumentSetParameter),
    InstrumentGetState(String),
    RootDataMessage(RootDataMessage),
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    id: f32,
    payload: MessageTypes,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response<T> {
    id: f32,
    payload: Option<T>,
}

impl TripleOsc {
    fn get_oscillator_mut(&mut self, id: i8) -> Option<&mut Oscillator> {
        match id {
            1 => Some(&mut self.osc1),
            2 => Some(&mut self.osc2),
            3 => Some(&mut self.osc3),
            _ => None,
        }
    }
}

impl OutputNode for TripleOsc {
    fn get_next_value(&mut self, sample_rate: f32) -> f32 {
        // Add all fields
        self.osc1.get_value(sample_rate)
            + self.osc2.get_value(sample_rate)
            + self.osc3.get_value(sample_rate)
    }
}

impl Instrument for TripleOsc {
    fn get_state(&self) -> instrument::InstrumentState {
        let oscillators = vec![&self.osc1, &self.osc2, &self.osc3];
        serde_json::to_value(oscillators).unwrap()
    }

    fn set_parameter(&mut self, set_parameter: InstrumentSetParameter) {
        let mut iter = set_parameter.parameter.split_whitespace();

        match iter.next() {
            Some("osc") => {
                if let Some(osc_id) = iter.next() {
                    let id = osc_id.parse::<i8>().unwrap();
                    let mut osc = self.get_oscillator_mut(id).unwrap();
                    match iter.next() {
                        Some("frequency") => {
                            osc.set_frequency(set_parameter.value.as_f64().unwrap() as f32);
                        }
                        Some("shape") => {
                            let shape: oscillator::Types =
                                serde_json::from_value(set_parameter.value).unwrap();
                            osc.set_shape(shape);
                        }
                        Some(unknown) => println!("Unknown type \"{}\"", unknown),
                        None => println!(
                            "You need to provide a parameter to change for oscillator {}",
                            id
                        ),
                    }
                }
            }
            _ => {
                println!("Unknown parameter");
            }
        }
    }
}

fn main() {
    // Cpal logic here, to retrieve the default output format
    // on the default device
    let device = cpal::default_output_device().expect("Failed to get default output device");
    let format = device
        .default_output_format()
        .expect("Failed to get default output format");
    let event_loop = cpal::EventLoop::new();
    let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
    event_loop.play_stream(stream_id);
    let sample_rate = format.sample_rate.0 as f32;

    // Output values logic
    // TODO: this needs some restructuration
    let output_values_size = 100;
    let output_values_freq = 8;
    let mut output_values_clock = 0;
    let mut output_values = VecDeque::<f32>::with_capacity(output_values_size);
    for _ in 0..output_values_size {
        output_values.push_back(0.0);
    }
    let output_values = Arc::new(Mutex::new(output_values));

    // // Build some oscillators
    // let mut osc_fm = Oscillator::new(2.0, oscillator::Types::Sawtooth);
    // let mut osc1 = Oscillator::new(440.0, oscillator::Types::Square);
    // let mut osc2 = Oscillator::new(100.0, oscillator::Types::Triangle);

    let triple_osc = TripleOsc {
        osc1: Oscillator::new(440.0, oscillator::Types::Sine),
        osc2: Oscillator::new(110.0, oscillator::Types::Square),
        osc3: Oscillator::new(55.0, oscillator::Types::Sawtooth),
    };

    // let biquadFilter = filter::BiquadFilter::new(sample_rate, 2000.0, 2.7, filter::BiquadFilterTypes::LowPass);

    let mut instruments = HashMap::new();
    instruments.insert(String::from("triple_osc"), triple_osc);
    let instruments = Arc::new(Mutex::new(instruments));

    // Generate the next value
    let mut next_value = || {
        let mut instruments = instruments.lock().unwrap();
        let mut output = 0.0;
        for (_, val) in instruments.iter_mut() {
            output += val.get_next_value(sample_rate);
        }

        // Save the output in the values ringbuffer
        if output_values_clock == output_values_freq - 1 {
            output_values_clock = 0;
            let mut output_values = output_values.lock().unwrap();
            output_values.pop_front();
            output_values.push_back(output);
        } else {
            output_values_clock += 1;
        }
        output
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
    let output_values = output_values.clone();
    let instruments = instruments.clone();
    let server = listener
        .incoming()
        .map_err(|e| println!("failed to accept socket; error = {:?}", e))
        .for_each(move |socket| {
            println!("New socket: {}", socket.peer_addr().unwrap());
            let (reader, writer) = socket.split();
            let lines = io::lines(BufReader::new(reader));

            let responses = lines.map(|line| {
                let message: Message = serde_json::from_str(&line).unwrap();
                message
            });

            let output_values = output_values.clone();
            let instruments = instruments.clone();
            let writes = responses.fold(writer, move |writer, message| {
                let mut instruments = instruments.lock().unwrap();
                let id = message.id;
                let default_res: Response<f32> = Response { id, payload: None };

                let mut response = match message.payload {
                    // Change an instrument's parameter
                    // MessageTypes::InstrumentSetValue(message) => match message.parameter {
                    //     InstrumentParameter::Frequency => {
                    //         let new_freq = (message.value * 100.0).powi(2);
                    //         test_filter.set_frequency(new_freq);
                    //         serde_json::to_string(&default_res).unwrap()
                    //     }
                    //     InstrumentParameter::Q => {
                    //         test_filter.set_Q(0.1 + message.value * 10.0);
                    //         serde_json::to_string(&default_res).unwrap()
                    //     }
                    // },
                    MessageTypes::InstrumentSetParameter(instrument_id, set_parameter) => {
                        let mut instrument = instruments.get_mut(&instrument_id).unwrap();
                        instrument.set_parameter(set_parameter);
                        serde_json::to_string(&default_res).unwrap()
                    }

                    MessageTypes::InstrumentGetState(instrument_id) => {
                        let instrument = instruments.get(&instrument_id).unwrap();
                        let res = Response {
                            id,
                            payload: Some(instrument.get_state()),
                        };
                        serde_json::to_string(&res).unwrap()
                    }

                    // Access root data
                    MessageTypes::RootDataMessage(message) => match message.target {
                        RootDataTargets::OutputBuffer => {
                            let output_values = &*output_values.lock().unwrap();
                            let res = Response {
                                id,
                                payload: Some(output_values),
                            };
                            serde_json::to_string(&res).unwrap()
                        }
                    },
                };

                response.push_str("\n");
                io::write_all(writer, response.into_bytes()).map(|(w, _)| w)
            });

            tokio::spawn(writes.then(|_| Ok(())))
        });

    thread::spawn(move || {
        tokio::run(server);
        println!("Done!");
    });

    play();
}
