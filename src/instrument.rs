extern crate serde;
extern crate serde_json;

use serde_json::Value;
use std::collections::HashMap;
use std::string::String;

#[derive(Serialize, Deserialize, Debug)]
pub struct InstrumentSetParameter {
    pub parameter: String,
    pub value: serde_json::Value,
}

pub type InstrumentState = Value;

pub trait Instrument {
    fn get_next_value(&mut self, sample_rate: f32) -> f32;
    fn get_state(&self) -> InstrumentState;
    fn set_parameter(&mut self, set_parameter: InstrumentSetParameter);
}
