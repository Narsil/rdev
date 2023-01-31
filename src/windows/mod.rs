mod common;
mod display;
#[cfg(feature = "unstable_grab")]
mod grab;
mod keyboard;
mod keycodes;
mod listen;
mod simulate;

pub use crate::windows::display::display_size;
#[cfg(feature = "unstable_grab")]
pub use crate::windows::grab::grab;
pub use crate::windows::keyboard::Keyboard;
pub use crate::windows::listen::listen;
pub use crate::windows::simulate::simulate;

// types not defined by windows-sys
pub type DWORD = u32;
pub type WORD = u16;
pub type LONG = i32;
