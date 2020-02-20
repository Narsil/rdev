extern crate libc;
extern crate x11;

mod keycodes;
mod listen;
mod simulate;

pub use crate::linux::listen::listen;
pub use crate::linux::simulate::simulate;
