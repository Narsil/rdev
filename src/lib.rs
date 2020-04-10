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
//! ### Listening to global events
//!
//! ```no_run
//! use rdev::{listen, Event};
//!
//! // This will block.
//! if let Err(error) = listen(callback) {
//!     println!("Error: {:?}", error)
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
//! ### Sending some events
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
//! send(&EventType::KeyPress(Key::KeyS));
//! send(&EventType::KeyRelease(Key::KeyS));
//!
//! send(&EventType::MouseMove { x: 0.0, y: 0.0 });
//! send(&EventType::MouseMove { x: 400.0, y: 400.0 });
//! send(&EventType::ButtonPress(Button::Left));
//! send(&EventType::ButtonRelease(Button::Right));
//! send(&EventType::Wheel {
//!     delta_x: 0,
//!     delta_y: 1,
//! });
//! ```
//! ### Getting the main screen size
//!
//! ```no_run
//! use rdev::{display_size};
//!
//! let (w, h) = display_size().unwrap();
//! assert!(w > 0);
//! assert!(h > 0);
//! ```
//!
//! ### Keyboard state
//!
//! We can define a dummy Keyboard, that we will use to detect
//! what kind of EventType trigger some String. We get the currently used
//! layout for now !
//! Caveat : This is layout dependent. If your app needs to support
//! layout switching don't use this !
//! Caveat: On Linux, the dead keys mechanism is not implemented.
//! Caveat: Only shift and dead keys are implemented, Alt+unicode code on windows
//! won't work.
//!
//! ```no_run
//! use rdev::{Keyboard, EventType, Key, KeyboardState};
//!
//! let mut keyboard = Keyboard::new().unwrap();
//! let string = keyboard.add(&EventType::KeyPress(Key::KeyS));
//! // string == Some("s")
//! ```
//!
//! ### Grabbing global events. (Requires `unstable_grab` feature)
//!
//! In the callback, returning None ignores the event
//! and returning the event let's it pass. There is no modification of the event
//! possible here.
//! Caveat: On MacOS, you require the grab
//! loop needs to be the primary app (no fork before) and need to have accessibility
//! settings enabled.
//! **Not implemented on Linux, you will always receive an error.**
//!
//! ### Serialization
//!
//! Serialization and deserialization. (Requires `serialize` feature).
mod rdev;
pub use crate::rdev::{
    Button, Callback, DisplayError, Event, EventType, GrabCallback, GrabError, Key, KeyboardState,
    ListenError, SimulateError,
};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use crate::macos::Keyboard;
#[cfg(target_os = "macos")]
use crate::macos::{display_size as _display_size, listen as _listen, simulate as _simulate};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use crate::linux::Keyboard;
#[cfg(target_os = "linux")]
use crate::linux::{display_size as _display_size, listen as _listen, simulate as _simulate};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use crate::windows::Keyboard;
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
///     if let Err(error) = listen(callback) {
///         println!("Error: {:?}", error)
///     }
/// }
/// ```
pub fn listen(callback: Callback) -> Result<(), ListenError> {
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
/// let (w, h) = display_size().unwrap();
/// println!("My screen size : {:?}x{:?}", w, h);
/// ```
pub fn display_size() -> Result<(u64, u64), DisplayError> {
    _display_size()
}

#[cfg(feature = "unstable_grab")]
#[cfg(target_os = "linux")]
pub use crate::linux::grab as _grab;
#[cfg(feature = "unstable_grab")]
#[cfg(target_os = "macos")]
pub use crate::macos::grab as _grab;
#[cfg(feature = "unstable_grab")]
#[cfg(target_os = "windows")]
pub use crate::windows::grab as _grab;
#[cfg(any(feature = "unstable_grab"))]
/// Grabbing global events. In the callback, returning None ignores the event
/// and returning the event let's it pass. There is no modification of the event
/// possible here.
/// Caveat: On MacOS, you require the grab
/// loop needs to be the primary app (no fork before) and need to have accessibility
/// settings enabled.
/// On Linux, this is not implemented, you will always receive an error.
///
/// ```no_run
/// use rdev::{grab, Event, EventType, Key};
///
/// fn callback(event: Event) -> Option<Event> {
///     println!("My callback {:?}", event);
///     match event.event_type{
///         EventType::KeyPress(Key::Tab) => None,
///         _ => Some(event),
///     }
/// }
/// fn main(){
///     // This will block.
///     if let Err(error) = grab(callback) {
///         println!("Error: {:?}", error)
///     }
/// }
/// ```
#[cfg(any(feature = "unstable_grab"))]
pub fn grab(callback: GrabCallback) -> Result<(), GrabError> {
    _grab(callback)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyboard_state() {
        // S
        let mut keyboard = Keyboard::new().unwrap();
        let char_s = keyboard.add(&EventType::KeyPress(Key::KeyS)).unwrap();
        assert_eq!(
            char_s,
            "s".to_string(),
            "This test should pass only on Qwerty layout !"
        );
        let n = keyboard.add(&EventType::KeyRelease(Key::KeyS));
        assert_eq!(n, None);

        // Shift + S
        keyboard.add(&EventType::KeyPress(Key::ShiftLeft));
        let char_s = keyboard.add(&EventType::KeyPress(Key::KeyS)).unwrap();
        assert_eq!(char_s, "S".to_string());
        let n = keyboard.add(&EventType::KeyRelease(Key::KeyS));
        assert_eq!(n, None);
        keyboard.add(&EventType::KeyRelease(Key::ShiftLeft));

        // Reset
        keyboard.add(&EventType::KeyPress(Key::ShiftLeft));
        keyboard.reset();
        let char_s = keyboard.add(&EventType::KeyPress(Key::KeyS)).unwrap();
        assert_eq!(char_s, "s".to_string());
        let n = keyboard.add(&EventType::KeyRelease(Key::KeyS));
        assert_eq!(n, None);
        keyboard.add(&EventType::KeyRelease(Key::ShiftLeft));

        // UsIntl layout required
        // let n = keyboard.add(&EventType::KeyPress(Key::Quote));
        // assert_eq!(n, Some("".to_string()));
        // let m = keyboard.add(&EventType::KeyRelease(Key::Quote));
        // assert_eq!(m, None);
        // let e = keyboard.add(&EventType::KeyPress(Key::KeyE)).unwrap();
        // assert_eq!(e, "é".to_string());
        // keyboard.add(&EventType::KeyRelease(Key::KeyE));
    }
}
