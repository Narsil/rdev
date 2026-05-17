#[cfg(feature = "unstable_grab")]
use crate::rdev::GrabError;

use crate::rdev::{DisplayError, Event, EventType, KeyboardState, ListenError, SimulateError};

#[cfg(feature = "wayland")]
mod wayland;

#[cfg(feature = "x11")]
mod x11;

#[cfg(not(any(feature = "wayland", feature = "x11")))]
compile_error!("Need to activate either wayland or x11 feature on linux");

#[derive(Clone, Copy)]
enum BackendKind {
    #[cfg(feature = "wayland")]
    Wayland,

    #[cfg(feature = "x11")]
    X11,
}

fn backend_kind() -> BackendKind {
    #[cfg(all(feature = "wayland", feature = "x11"))]
    {
        let has_wayland_display = std::env::var_os("WAYLAND_DISPLAY").is_some();

        let is_wayland_session = std::env::var("XDG_SESSION_TYPE")
            .map(|session| session.eq_ignore_ascii_case("wayland"))
            .unwrap_or(false);

        if has_wayland_display || is_wayland_session {
            BackendKind::Wayland
        } else {
            BackendKind::X11
        }
    }

    #[cfg(all(feature = "wayland", not(feature = "x11")))]
    {
        BackendKind::Wayland
    }

    #[cfg(all(feature = "x11", not(feature = "wayland")))]
    {
        BackendKind::X11
    }
}

macro_rules! dispatch_backend {
    ($function:ident ( $($arg:expr),* $(,)? )) => {
        match backend_kind() {
            #[cfg(feature = "wayland")]
            BackendKind::Wayland => wayland::$function($($arg),*),

            #[cfg(feature = "x11")]
            BackendKind::X11 => x11::$function($($arg),*),
        }
    };
}

macro_rules! dispatch_keyboard {
    ($self:expr, $method:ident ( $($arg:expr),* $(,)? )) => {
        match $self {
            #[cfg(feature = "wayland")]
            Keyboard::Wayland(keyboard) => keyboard.$method($($arg),*),

            #[cfg(feature = "x11")]
            Keyboard::X11(keyboard) => keyboard.$method($($arg),*),
        }
    };
}

pub fn display_size() -> Result<(u64, u64), DisplayError> {
    dispatch_backend!(display_size())
}

pub fn listen<T>(callback: T) -> Result<(), ListenError>
where
    T: FnMut(Event) + 'static,
{
    dispatch_backend!(listen(callback))
}

pub fn simulate(event_type: &EventType) -> Result<(), SimulateError> {
    dispatch_backend!(simulate(event_type))
}

#[cfg(feature = "unstable_grab")]
pub fn grab<T>(callback: T) -> Result<(), GrabError>
where
    T: FnMut(Event) -> Option<Event> + 'static,
{
    dispatch_backend!(grab(callback))
}

pub enum Keyboard {
    #[cfg(feature = "wayland")]
    Wayland(wayland::Keyboard),

    #[cfg(feature = "x11")]
    X11(x11::Keyboard),
}

impl Keyboard {
    pub fn new() -> Option<Self> {
        match backend_kind() {
            #[cfg(feature = "wayland")]
            BackendKind::Wayland => wayland::Keyboard::new().ok().map(Self::Wayland),

            #[cfg(feature = "x11")]
            BackendKind::X11 => x11::Keyboard::new().map(Self::X11),
        }
    }
}

impl KeyboardState for Keyboard {
    fn add(&mut self, event_type: &EventType) -> Option<String> {
        dispatch_keyboard!(self, add(event_type))
    }

    fn reset(&mut self) {
        dispatch_keyboard!(self, reset())
    }
}
