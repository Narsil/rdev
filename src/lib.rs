mod rdev;
pub use crate::rdev::{Button, Callback, Event, EventType, Key, SimulateError};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use crate::macos::{listen as _listen, simulate as _simulate};

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
use crate::linux::{listen as _listen, simulate as _simulate};

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
use crate::windows::{listen as _listen, simulate as _simulate};

/// Listening to global events. Caveat: On MacOS, you require the listen
/// loop needs to be the primary app (no fork before) and need to have accessibility
/// settings enabled.
///
/// ```no_run
/// use rdev::{listen, Event};
///
/// fn callback(event: Event) {
///     println!("My callback {:?}", event);
///     match event.name{
///         Some(string) => println!("User wrote {:?}", string),
///         None => ()
///     }
/// }
/// ```
pub fn listen(callback: Callback) {
    _listen(callback)
}

/// Sending some events
///
/// ```no_run
/// use rdev::{simulate, Button, EventType, Key, SimulateError};
/// use std::{thread, time};
///
/// fn send(event_type: &EventType) {
///     let delay = time::Duration::from_millis(20);
///     match simulate(event_type) {
///         Ok(()) => (),
///         Err(SimulateError) => {
///             println!("We could not send {:?}", event_type);
///         }
///     }
///     // Let ths OS catchup (at least MacOS)
///     thread::sleep(delay);
/// }
///
/// fn my_shortcut() {
///     send(&EventType::KeyPress(Key::KeyS));
///     send(&EventType::KeyRelease(Key::KeyS));
///
///     send(&EventType::MouseMove { x: 0.0, y: 0.0 });
///     send(&EventType::MouseMove { x: 400.0, y: 400.0 });
///     send(&EventType::ButtonPress(Button::Left));
///     send(&EventType::ButtonRelease(Button::Right));
///     send(&EventType::Wheel {
///         delta_x: 0,
///         delta_y: 1,
///     });
/// }
/// ```
pub fn simulate(event_type: &EventType) -> Result<(), SimulateError> {
    _simulate(event_type)
}
