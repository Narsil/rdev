#[cfg(feature = "x11")]
mod x11;

#[cfg(feature = "x11")]
pub use x11::*;

#[cfg(all(feature = "wayland", not(feature = "x11")))]
mod wayland;

#[cfg(all(feature = "wayland", not(feature = "x11")))]
pub use wayland::*;

#[cfg(not(any(feature = "wayland", feature = "x11")))]
compile_error!("Need to activate either wayland or x11 feature on linux");
