extern crate winapi;

mod keycodes;
mod listen;
mod simulate;

pub use crate::windows::listen::listen;
pub use crate::windows::simulate::simulate;
