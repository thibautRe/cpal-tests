pub trait Instrument {
    fn set_sample_rate(&mut self, sample_rate: f32);
    fn get_sample_rate(&self) -> f32;
    fn get_next_value(&mut self) -> f32;
}
