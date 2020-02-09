mod rdev;
pub use crate::rdev::{Event, EventError, EventType, SimulateError};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use crate::macos::{listen, simulate};

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use crate::linux::{listen, simulate};
