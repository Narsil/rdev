mod rdev;
pub use crate::rdev::{Event, EventType};

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "linux")]
pub use crate::linux::listen;
