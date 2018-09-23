extern crate serde;
extern crate serde_json;

use serde_json::Value;
use std::collections::HashMap;
use std::string::String;
#[derive(Serialize, Deserialize, Debug)]
pub enum InstrumentParameter {
    Q,
    Frequency,
}

pub type InstrumentState = Value;

pub trait Instrument {
    fn get_next_value(&mut self, sample_rate: f32) -> f32;
    fn get_state(&self) -> InstrumentState;
}

/// A set of instruments, identified by a key
pub type Instruments = HashMap<String, Instrument>;
