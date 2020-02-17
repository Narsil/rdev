use std::time::SystemTime;

/// Callback type to send to listen function.
pub type Callback = fn(event: Event);

/// Marking an error when we tried to simulate and event
#[derive(Debug)]
pub struct SimulateError;

/// In order to manage different OS, the current EventType choices is a mix&match
/// to account for all possible events.
#[derive(Debug)]
pub enum EventType {
    /// This code is the raw keycode of the OS they tend to vary depending on keyboard
    /// layout. They might not event correspond to a key on dead keys layout.
    KeyPress {
        code: u8,
    },
    KeyRelease {
        code: u8,
    },
    /// Note: On MacOS, only LeftButton and RightButton Press and Release are defined
    /// They are currently mapped to ButtonPress{code: 1} and ButtonPress{code: 3} to
    /// be closer to X11 behaviour
    ButtonPress {
        code: u8,
    },
    ButtonRelease {
        code: u8,
    },
    /// Values in pixels
    MouseMove {
        x: f64,
        y: f64,
    },
    /// Note: On Linux, there is no actual delta the actual values are ignored for delta_x
    /// and we only look at the sign of delta_y to simulate wheelup or wheeldown.
    Wheel {
        delta_x: i64,
        delta_y: i64,
    },
}

/// When events arrive from the system we can add some information
/// time is when the event was received, name *will* be at some point changed
/// to be mapped to the function of the key (Alt, s, Return and so on).
#[derive(Debug)]
pub struct Event {
    pub time: SystemTime,
    pub name: Option<String>,
    pub event_type: EventType,
}
