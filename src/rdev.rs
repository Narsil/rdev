#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use std::{fmt, fmt::Display};

// /// Callback type to send to listen function.
// pub type Callback = dyn FnMut(Event) -> ();

/// Callback type to send to grab function.
pub type GrabCallback = fn(event: Event) -> Option<Event>;

/// Errors that occur when trying to capture OS events.
/// Be careful on Mac, not setting accessibility does not cause an error
/// it justs ignores events.
#[derive(Debug)]
#[non_exhaustive]
pub enum ListenError {
    /// MacOS
    EventTapError,
    /// MacOS
    LoopSourceError,
    /// Linux
    MissingDisplayError,
    /// Linux
    KeyboardError,
    /// Linux
    RecordContextEnablingError,
    /// Linux
    RecordContextError,
    /// Linux
    XRecordExtensionError,
    /// Windows
    KeyHookError(u32),
    /// Windows
    MouseHookError(u32),
}

/// Errors that occur when trying to grab OS events.
/// Be careful on Mac, not setting accessibility does not cause an error
/// it justs ignores events.
#[derive(Debug)]
#[non_exhaustive]
pub enum GrabError {
    /// MacOS
    EventTapError,
    /// MacOS
    LoopSourceError,
    /// Linux
    MissingDisplayError,
    /// Linux
    KeyboardError,
    /// Windows
    KeyHookError(u32),
    /// Windows
    MouseHookError(u32),
    /// All
    SimulateError,
    IoError(std::io::Error),
}

/// Errors that occur when trying to get display size.
#[non_exhaustive]
#[derive(Debug)]
pub enum DisplayError {
    NoDisplay,
    ConversionError,
}

impl From<SimulateError> for GrabError {
    fn from(_: SimulateError) -> GrabError {
        GrabError::SimulateError
    }
}

impl From<std::io::Error> for GrabError {
    fn from(err: std::io::Error) -> GrabError {
        GrabError::IoError(err)
    }
}

/// Marking an error when we tried to simulate and event
#[derive(Debug)]
pub struct SimulateError;

impl Display for SimulateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not simulate event")
    }
}

impl std::error::Error for SimulateError {}

/// Key names based on physical location on the device
/// Merge Option(MacOS) and Alt(Windows, Linux) into Alt
/// Merge Windows (Windows), Meta(Linux), Command(MacOS) into Meta
/// Characters based on Qwerty layout, don't use this for characters as it WILL
/// depend on the layout. Use Event.name instead. Key modifiers gives those keys
/// a different value too.
/// Careful, on Windows KpReturn does not exist, it' s strictly equivalent to Return, also Keypad keys
/// get modified if NumLock is Off and ARE pagedown and so on.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum Key {
    /// Alt key on Linux and Windows (option key on macOS)
    Alt,
    AltGr,
    Backspace,
    CapsLock,
    ControlLeft,
    ControlRight,
    Delete,
    DownArrow,
    End,
    Escape,
    F1,
    F10,
    F11,
    F12,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    Home,
    LeftArrow,
    /// also known as "windows", "super", and "command"
    MetaLeft,
    /// also known as "windows", "super", and "command"
    MetaRight,
    PageDown,
    PageUp,
    Return,
    RightArrow,
    ShiftLeft,
    ShiftRight,
    Space,
    Tab,
    UpArrow,
    PrintScreen,
    ScrollLock,
    Pause,
    NumLock,
    BackQuote,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Num0,
    Minus,
    Equal,
    KeyQ,
    KeyW,
    KeyE,
    KeyR,
    KeyT,
    KeyY,
    KeyU,
    KeyI,
    KeyO,
    KeyP,
    LeftBracket,
    RightBracket,
    KeyA,
    KeyS,
    KeyD,
    KeyF,
    KeyG,
    KeyH,
    KeyJ,
    KeyK,
    KeyL,
    SemiColon,
    Quote,
    BackSlash,
    IntlBackslash,
    KeyZ,
    KeyX,
    KeyC,
    KeyV,
    KeyB,
    KeyN,
    KeyM,
    Comma,
    Dot,
    Slash,
    Insert,
    KpReturn,
    KpMinus,
    KpPlus,
    KpMultiply,
    KpDivide,
    Kp0,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    KpDelete,
    Function,
    Unknown(u32),
}

/// Adds the ability to retrieve a key from a string
/// return None if str != key
impl Key {
    pub fn from_str(str: &str) -> Option<Key> {
        let str = &*str.to_lowercase();
        match str {
            "alt" => Some(Key::Alt),
            "altgr" => Some(Key::AltGr),
            "backspace" => Some(Key::Backspace),
            "capslock" => Some(Key::CapsLock),
            "controlleft" => Some(Key::ControlLeft),
            "controlright" => Some(Key::ControlRight),
            "delete" => Some(Key::Delete),
            "downarrow" => Some(Key::DownArrow),
            "end" => Some(Key::End),
            "escape" => Some(Key::Escape),
            "f1" => Some(Key::F1),
            "f10" => Some(Key::F10),
            "f11" => Some(Key::F11),
            "f12" => Some(Key::F12),
            "f2" => Some(Key::F2),
            "f3" => Some(Key::F3),
            "f4" => Some(Key::F4),
            "f5" => Some(Key::F5),
            "f6" => Some(Key::F6),
            "f7" => Some(Key::F7),
            "f8" => Some(Key::F8),
            "f9" => Some(Key::F9),
            "home" => Some(Key::Home),
            "leftarrow" => Some(Key::LeftArrow),
            /// also known as "windows", "super", and "command"
            "metaleft" => Some(Key::MetaLeft),
            /// also known as "windows", "super", and "command"
            "metaright" => Some(Key::MetaRight),
            "pagedown" => Some(Key::PageDown),
            "pageup" => Some(Key::PageUp),
            "return" => Some(Key::Return),
            "rightarrow" => Some(Key::RightArrow),
            "shiftleft" => Some(Key::ShiftLeft),
            "shiftright" => Some(Key::ShiftRight),
            "space" => Some(Key::Space),
            "tab" => Some(Key::Tab),
            "uparrow" => Some(Key::UpArrow),
            "printscreen" => Some(Key::PrintScreen),
            "scrollLock" => Some(Key::ScrollLock),
            "pause" => Some(Key::Pause),
            "numLock" => Some(Key::NumLock),
            "backquote" => Some(Key::BackQuote),
            "num1" => Some(Key::Num1),
            "num2" => Some(Key::Num2),
            "num3" => Some(Key::Num3),
            "num4" => Some(Key::Num4),
            "num5" => Some(Key::Num5),
            "num6" => Some(Key::Num6),
            "num7" => Some(Key::Num7),
            "num8" => Some(Key::Num8),
            "num9" => Some(Key::Num9),
            "num0" => Some(Key::Num0),
            "minus" => Some(Key::Minus),
            "equal" => Some(Key::Equal),
            "q" => Some(Key::KeyQ),
            "w" => Some(Key::KeyW),
            "e" => Some(Key::KeyE),
            "r" => Some(Key::KeyR),
            "t" => Some(Key::KeyT),
            "y" => Some(Key::KeyY),
            "u" => Some(Key::KeyU),
            "i" => Some(Key::KeyI),
            "o" => Some(Key::KeyO),
            "p" => Some(Key::KeyP),
            "leftbracket" => Some(Key::LeftBracket),
            "rightbracket" => Some(Key::RightBracket),
            "a" => Some(Key::KeyA),
            "s" => Some(Key::KeyS),
            "d" => Some(Key::KeyD),
            "f" => Some(Key::KeyF),
            "g" => Some(Key::KeyG),
            "h" => Some(Key::KeyH),
            "j" => Some(Key::KeyJ),
            "k" => Some(Key::KeyK),
            "l" => Some(Key::KeyL),
            "semicolon" => Some(Key::SemiColon),
            "quote" => Some(Key::Quote),
            "backslash" => Some(Key::BackSlash),
            "intlbackslash" => Some(Key::IntlBackslash),
            "z" => Some(Key::KeyZ),
            "x" => Some(Key::KeyX),
            "c" => Some(Key::KeyC),
            "v" => Some(Key::KeyV),
            "b" => Some(Key::KeyB),
            "n" => Some(Key::KeyN),
            "m" => Some(Key::KeyM),
            "comma" => Some(Key::Comma),
            "dot" => Some(Key::Dot),
            "slash" => Some(Key::Slash),
            "insert" => Some(Key::Insert),
            "kpreturn" => Some(Key::KpReturn),
            "kpminus" => Some(Key::KpMinus),
            "kpplus" => Some(Key::KpPlus),
            "kpmultiply" => Some(Key::KpMultiply),
            "kpdivide" => Some(Key::KpDivide),
            "kp0" => Some(Key::Kp0),
            "kp1" => Some(Key::Kp1),
            "kp2" => Some(Key::Kp2),
            "kp3" => Some(Key::Kp3),
            "kp4" => Some(Key::Kp4),
            "kp5" => Some(Key::Kp5),
            "kp6" => Some(Key::Kp6),
            "kp7" => Some(Key::Kp7),
            "kp8" => Some(Key::Kp8),
            "kp9" => Some(Key::Kp9),
            "kpdelete" => Some(Key::KpDelete),
            "function" => Some(Key::Function),
            _ => None,
        }
    }
}

/// Standard mouse buttons
/// Some mice have more than 3 buttons. These are not defined, and different
/// OSs will give different `Button::Unknown` values.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum Button {
    Left,
    Right,
    Middle,
    Unknown(u8),
}

/// In order to manage different OSs, the current EventType choices are a mix and
/// match to account for all possible events.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum EventType {
    /// The keys correspond to a standard qwerty layout, they don't correspond
    /// To the actual letter a user would use, that requires some layout logic to be added.
    KeyPress(Key),
    KeyRelease(Key),
    /// Mouse Button
    ButtonPress(Button),
    ButtonRelease(Button),
    /// Values in pixels. `EventType::MouseMove{x: 0, y: 0}` corresponds to the
    /// top left corner, with x increasing downward and y increasing rightward
    MouseMove {
        x: f64,
        y: f64,
    },
    /// `delta_y` represents vertical scroll and `delta_x` represents horizontal scroll.
    /// Positive values correspond to scrolling up or right and negative values
    /// correspond to scrolling down or left
    Wheel {
        delta_x: i64,
        delta_y: i64,
    },
}

/// When events arrive from the OS they get some additional information added from
/// EventType, which is the time when this event was received, and the name Option
/// which contains what characters should be emmitted from that event. This relies
/// on the OS layout and keyboard state machinery.
/// Caveat: Dead keys don't function on Linux(X11) yet. You will receive None for
/// a dead key, and the raw letter instead of accentuated letter.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct Event {
    pub time: SystemTime,
    pub name: Option<String>,
    pub event_type: EventType,
}

/// We can define a dummy Keyboard, that we will use to detect
/// what kind of EventType trigger some String. We get the currently used
/// layout for now !
/// Caveat : This is layout dependent. If your app needs to support
/// layout switching don't use this !
/// Caveat: On Linux, the dead keys mechanism is not implemented.
/// Caveat: Only shift and dead keys are implemented, Alt+unicode code on windows
/// won't work.
///
/// ```no_run
/// use rdev::{Keyboard, EventType, Key, KeyboardState};
///
/// let mut keyboard = Keyboard::new().unwrap();
/// let string = keyboard.add(&EventType::KeyPress(Key::KeyS));
/// // string == Some("s")
/// ```
pub trait KeyboardState {
    /// Changes the keyboard state as if this event happened. we don't
    /// really hit the OS here, which might come handy to test what should happen
    /// if we were to hit said key.
    fn add(&mut self, event_type: &EventType) -> Option<String>;

    /// Resets the keyboard state as if we never touched it (no shift, caps_lock and so on)
    fn reset(&mut self);
}
