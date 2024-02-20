extern crate libc;
extern crate x11;

mod common;
mod display;
#[cfg(feature = "unstable_grab")]
mod grab;
mod keyboard;
mod keycodes;
mod listen;
mod simulate;

pub use crate::x11::display::display_size;
#[cfg(feature = "unstable_grab")]
pub use crate::x11::grab::grab;
pub use crate::x11::keyboard::Keyboard;
pub use crate::x11::listen::listen;
pub use crate::x11::simulate::simulate;
