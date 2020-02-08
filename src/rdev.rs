use serde::ser::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, PartialEq)]
pub enum EventError {
    InvalidCoordinates,
    InvalidName,
}

#[derive(Debug)]
pub enum EventType {
    KeyPress { code: u8 },
    KeyRelease { code: u8 },
    ButtonPress { code: u8 },
    ButtonRelease { code: u8 },
    MouseMove { x: u64, y: u64 },
    Wheel { delta_x: u64, delta_y: u64 },
}

#[derive(Debug)]
pub struct Event {
    time: SystemTime,
    name: Option<String>,
    event_type: EventType,
}

impl Event {
    pub fn new(
        event_type: EventType,
        time: SystemTime,
        name: Option<String>,
    ) -> Result<Event, EventError> {
        Ok(Event {
            event_type,
            time,
            name,
        })
    }
}
