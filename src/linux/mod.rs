#[cfg(feature = "x11")]
mod x11;

#[cfg(feature = "x11")]
pub use x11::*;

#[cfg(feature = "wayland")]
mod wayland;

#[cfg(feature = "wayland")]
pub use wayland::*;

#[cfg(not(any(feature = "wayland", feature = "x11")))]
compile_error!("Need to activate either wayland or x11 feature on linux");
