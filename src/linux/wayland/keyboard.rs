use crate::rdev::{EventType, KeyboardState};

#[derive(Debug)]
pub struct Keyboard {}

impl KeyboardState for Keyboard {
    fn add(&mut self, _event_type: &EventType) -> Option<String> {
        None
    }
    fn reset(&mut self) {}
}
