#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, time::SystemTime};
use std::{fmt, fmt::Display};

/// Callback type to send to listen function.
pub type Callback = fn(event: Event);

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

/// Errors if a `Key` is attempted to be converted to a `char`,
/// but no appropriate `char` could be found
#[derive(Debug)]
pub struct ConvertToCharError {
    pub code: Option<u32>,
}

impl Display for ConvertToCharError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Could not convert Key to char")
    }
}

impl std::error::Error for ConvertToCharError {}

/// Key names based on physical location on the device
/// Merge Option(MacOS) and Alt(Windows, Linux) into Alt
/// Merge Windows (Windows), Meta(Linux), Command(MacOS) into Meta
/// Characters based on Qwerty layout, don't use this for characters as it WILL
/// depend on the layout. Use Event.name instead. Key modifiers gives those keys
/// a different value too.
/// Careful, on Windows KpReturn does not exist, it' s strictly equivalent to Return, also Keypad keys
/// get modified if NumLock is Off and ARE pagedown and so on.
#[derive(Debug, Copy, Clone, PartialEq)]
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

impl TryFrom<Key> for char {
    type Error = ConvertToCharError;

    fn try_from(value: Key) -> Result<Self, Self::Error> {
        match value {
            Key::Space => Ok(' '),
            Key::Num1 => Ok('1'),
            Key::Num2 => Ok('2'),
            Key::Num3 => Ok('3'),
            Key::Num4 => Ok('4'),
            Key::Num5 => Ok('5'),
            Key::Num6 => Ok('6'),
            Key::Num7 => Ok('7'),
            Key::Num8 => Ok('8'),
            Key::Num9 => Ok('9'),
            Key::Num0 => Ok('0'),
            Key::Minus => Ok('-'),
            Key::Equal => Ok('='),
            Key::KeyQ => Ok('q'),
            Key::KeyW => Ok('w'),
            Key::KeyE => Ok('e'),
            Key::KeyR => Ok('r'),
            Key::KeyT => Ok('t'),
            Key::KeyY => Ok('y'),
            Key::KeyU => Ok('u'),
            Key::KeyI => Ok('i'),
            Key::KeyO => Ok('o'),
            Key::KeyP => Ok('p'),
            Key::LeftBracket => Ok('('),
            Key::RightBracket => Ok(')'),
            Key::KeyA => Ok('a'),
            Key::KeyS => Ok('s'),
            Key::KeyD => Ok('d'),
            Key::KeyF => Ok('f'),
            Key::KeyG => Ok('g'),
            Key::KeyH => Ok('h'),
            Key::KeyJ => Ok('j'),
            Key::KeyK => Ok('k'),
            Key::KeyL => Ok('l'),
            Key::SemiColon => Ok(';'),
            Key::Quote => Ok('\''),
            Key::BackSlash => Ok('\\'),
            Key::IntlBackslash => Ok('\\'),
            Key::KeyZ => Ok('z'),
            Key::KeyX => Ok('x'),
            Key::KeyC => Ok('c'),
            Key::KeyV => Ok('v'),
            Key::KeyB => Ok('b'),
            Key::KeyN => Ok('n'),
            Key::KeyM => Ok('m'),
            Key::Comma => Ok(','),
            Key::Dot => Ok('.'),
            Key::Slash => Ok('/'),
            Key::KpMinus => Ok('-'),
            Key::KpPlus => Ok('+'),
            Key::KpMultiply => Ok('*'),
            Key::KpDivide => Ok('/'),
            Key::Kp0 => Ok('0'),
            Key::Kp1 => Ok('1'),
            Key::Kp2 => Ok('2'),
            Key::Kp3 => Ok('3'),
            Key::Kp4 => Ok('4'),
            Key::Kp5 => Ok('5'),
            Key::Kp6 => Ok('6'),
            Key::Kp7 => Ok('7'),
            Key::Kp8 => Ok('8'),
            Key::Kp9 => Ok('9'),
            Key::Unknown(code) => Err(ConvertToCharError { code: Some(code) }),
            _ => Err(ConvertToCharError { code: None }),
        }
    }
}

/// Standard mouse buttons
/// Some mice have more than 3 buttons. These are not defined, and different
/// OSs will give different `Button::Unknown` values.
#[derive(Debug, Copy, Clone, PartialEq)]
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
    /// Note: Linux does not support horizontal scroll. When simulating scroll on Linux,
    /// only the sign of delta_y is considered, and not the magnitude to determine wheelup or wheeldown.
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
