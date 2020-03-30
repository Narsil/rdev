extern crate libc;
extern crate x11;

mod common;
mod display;
mod grab;
mod keyboard_state;
mod keycodes;
mod listen;
mod simulate;

pub use crate::linux::display::display_size;
pub use crate::linux::grab::grab;
pub use crate::linux::listen::listen;
pub use crate::linux::simulate::simulate;
