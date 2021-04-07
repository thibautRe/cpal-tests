extern crate serde;
extern crate serde_json;

use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct InstrumentSetParameter {
    pub parameter: String,
    pub value: serde_json::Value,
}

pub type InstrumentState = Value;

// Could potentially be a filter?
pub trait Instrument {
    fn get_state(&self) -> InstrumentState;
    fn set_parameter(&mut self, InstrumentSetParameter);
}

pub trait OutputNode {
    fn get_next_value(&mut self, sample_rate: f32) -> f32;
}

pub trait InputNode {
    //
}

pub struct Instruments<'a> {
    instruments: HashMap<String, Box<Instrument + 'a>>,
}

impl<'a> Instruments<'a> {
    pub fn new() -> Self {
        let empty_hashmap: HashMap<String, Box<Instrument>> = HashMap::new();
        Self {
            instruments: empty_hashmap,
        }
    }

    pub fn add_instrument<T>(&mut self, name: String, instrument: T)
    where
        T: Instrument + 'a,
    {
        let box_instrument = Box::new(instrument);
        // TODO generate a truly unique name (based on name + random ID)
        self.instruments.insert(name, box_instrument);
    }

    pub fn get_instruments(&self) -> HashMap<String, Box<Instrument + 'a>> {
        self.instruments
    }
}
