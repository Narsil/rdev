use crate::{
    rdev::{EventType, KeyboardState},
    SimulateError,
};

#[derive(Debug)]
pub struct Keyboard {}

impl Keyboard {
    pub fn new() -> Result<Self, SimulateError> {
        Ok(Self {})
    }
}

impl KeyboardState for Keyboard {
    fn add(&mut self, _event_type: &EventType) -> Option<String> {
        None
    }
    fn reset(&mut self) {}
}
