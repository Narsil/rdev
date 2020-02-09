use std::time::SystemTime;

#[derive(Debug)]
pub enum EventError {
    InvalidCoordinates,
    InvalidName,
    SimulateError,
}
#[derive(Debug)]
pub struct SimulateError;

#[derive(Debug)]
pub enum EventType {
    KeyPress { code: u8 },
    KeyRelease { code: u8 },
    ButtonPress { code: u8 },
    ButtonRelease { code: u8 },
    MouseMove { x: f64, y: f64 },
    Wheel { delta_x: i64, delta_y: i64 },
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
