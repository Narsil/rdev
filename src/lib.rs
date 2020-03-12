//! Simple library to listen and send events to keyboard and mouse on MacOS, Windows and Linux
//! (x11).
//!
//! You can also check out [Enigo](https://github.com/Enigo-rs/Enigo) which is another
//! crate which helped me write this one.
//!
//! This crate is so far a pet project for me to understand the rust ecosystem.
//!
//! ## Simple Usage
//!
//! Listening to global events
//!
//! ```no_run
//! use rdev::{listen, Event};
//!
//! fn main() {
//!     listen(callback);
//! }
//!
//! fn callback(event: Event) {
//!     println!("My callback {:?}", event);
//!     match event.name {
//!         Some(string) => println!("User wrote {:?}", string),
//!         None => (),
//!     }
//! }
//! ```
//!
//! Sending some events
//!
//! ```no_run
//! use rdev::{simulate, Button, EventType, Key, SimulateError};
//! use std::{thread, time};
//!
//! fn send(event_type: &EventType) {
//!     let delay = time::Duration::from_millis(20);
//!     match simulate(event_type) {
//!         Ok(()) => (),
//!         Err(SimulateError) => {
//!             println!("We could not send {:?}", event_type);
//!         }
//!     }
//!     // Let ths OS catchup (at least MacOS)
//!     thread::sleep(delay);
//! }
//!
//! fn main() {
//!     send(&EventType::KeyPress(Key::KeyS));
//!     send(&EventType::KeyRelease(Key::KeyS));
//!
//!     send(&EventType::MouseMove { x: 0.0, y: 0.0 });
//!     send(&EventType::MouseMove { x: 400.0, y: 400.0 });
//!     send(&EventType::ButtonPress(Button::Left));
//!     send(&EventType::ButtonRelease(Button::Right));
//!     send(&EventType::Wheel {
//!         delta_x: 0,
//!         delta_y: 1,
//!     });
//! }
//! ```
mod rdev;
pub use crate::rdev::{Button, Callback, Event, EventType, Key, SimulateError};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use crate::macos::{display_size as _display_size, listen as _listen, simulate as _simulate};

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
use crate::linux::{display_size as _display_size, listen as _listen, simulate as _simulate};

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
use crate::windows::{display_size as _display_size, listen as _listen, simulate as _simulate};

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
/// fn main(){
///     // This will block.
///     listen(callback);
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

/// Returns the size in pixels of the main screen.
/// This is useful to use with x, y from MouseMove Event.
///
/// ```no_run
/// use rdev::{display_size};
///
/// fn main() {
///     let (w, h )= display_size();
///     println!("My screen size : {:?}x{:?}", w, h);
/// }
/// ```
pub fn display_size() -> (u64, u64) {
    _display_size()
}
