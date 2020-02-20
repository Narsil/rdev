mod rdev;
pub use crate::rdev::{Event, EventType, Key, SimulateError};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use crate::macos::{listen, simulate};

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use crate::linux::{listen, simulate};

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use crate::windows::{listen, simulate};
