use objc2_core_graphics::CGKeyCode;

use crate::rdev::Key;
use std::convert::TryInto;

/// Option
const ALT: CGKeyCode = 58;
/// Option_Right
const ALT_GR: CGKeyCode = 61;
const BACKSPACE: CGKeyCode = 51;
const CAPS_LOCK: CGKeyCode = 57;
const CONTROL_LEFT: CGKeyCode = 59;
const CONTROL_RIGHT: CGKeyCode = 62;
const DOWN_ARROW: CGKeyCode = 125;
const ESCAPE: CGKeyCode = 53;
const F1: CGKeyCode = 122;
const F10: CGKeyCode = 109;
const F11: CGKeyCode = 103;
const F12: CGKeyCode = 111;
const F2: CGKeyCode = 120;
const F3: CGKeyCode = 99;
const F4: CGKeyCode = 118;
const F5: CGKeyCode = 96;
const F6: CGKeyCode = 97;
const F7: CGKeyCode = 98;
const F8: CGKeyCode = 100;
const F9: CGKeyCode = 101;
const FUNCTION: CGKeyCode = 63;
const LEFT_ARROW: CGKeyCode = 123;
const META_LEFT: CGKeyCode = 55;
const META_RIGHT: CGKeyCode = 54;
const RETURN: CGKeyCode = 36;
const RIGHT_ARROW: CGKeyCode = 124;
const SHIFT_LEFT: CGKeyCode = 56;
const SHIFT_RIGHT: CGKeyCode = 60;
const SPACE: CGKeyCode = 49;
const TAB: CGKeyCode = 48;
const UP_ARROW: CGKeyCode = 126;
const BACK_QUOTE: CGKeyCode = 50;
const NUM1: CGKeyCode = 18;
const NUM2: CGKeyCode = 19;
const NUM3: CGKeyCode = 20;
const NUM4: CGKeyCode = 21;
const NUM5: CGKeyCode = 23;
const NUM6: CGKeyCode = 22;
const NUM7: CGKeyCode = 26;
const NUM8: CGKeyCode = 28;
const NUM9: CGKeyCode = 25;
const NUM0: CGKeyCode = 29;
const MINUS: CGKeyCode = 27;
const EQUAL: CGKeyCode = 24;
const KEY_Q: CGKeyCode = 12;
const KEY_W: CGKeyCode = 13;
const KEY_E: CGKeyCode = 14;
const KEY_R: CGKeyCode = 15;
const KEY_T: CGKeyCode = 17;
const KEY_Y: CGKeyCode = 16;
const KEY_U: CGKeyCode = 32;
const KEY_I: CGKeyCode = 34;
const KEY_O: CGKeyCode = 31;
const KEY_P: CGKeyCode = 35;
const LEFT_BRACKET: CGKeyCode = 33;
const RIGHT_BRACKET: CGKeyCode = 30;
const KEY_A: CGKeyCode = 0;
const KEY_S: CGKeyCode = 1;
const KEY_D: CGKeyCode = 2;
const KEY_F: CGKeyCode = 3;
const KEY_G: CGKeyCode = 5;
const KEY_H: CGKeyCode = 4;
const KEY_J: CGKeyCode = 38;
const KEY_K: CGKeyCode = 40;
const KEY_L: CGKeyCode = 37;
const SEMI_COLON: CGKeyCode = 41;
const QUOTE: CGKeyCode = 39;
const BACK_SLASH: CGKeyCode = 42;
const KEY_Z: CGKeyCode = 6;
const KEY_X: CGKeyCode = 7;
const KEY_C: CGKeyCode = 8;
const KEY_V: CGKeyCode = 9;
const KEY_B: CGKeyCode = 11;
const KEY_N: CGKeyCode = 45;
const KEY_M: CGKeyCode = 46;
const COMMA: CGKeyCode = 43;
const DOT: CGKeyCode = 47;
const SLASH: CGKeyCode = 44;

pub fn code_from_key(key: Key) -> Option<CGKeyCode> {
    match key {
        Key::Alt => Some(ALT),
        Key::AltGr => Some(ALT_GR),
        Key::Backspace => Some(BACKSPACE),
        Key::CapsLock => Some(CAPS_LOCK),
        Key::ControlLeft => Some(CONTROL_LEFT),
        Key::ControlRight => Some(CONTROL_RIGHT),
        Key::DownArrow => Some(DOWN_ARROW),
        Key::Escape => Some(ESCAPE),
        Key::F1 => Some(F1),
        Key::F10 => Some(F10),
        Key::F11 => Some(F11),
        Key::F12 => Some(F12),
        Key::F2 => Some(F2),
        Key::F3 => Some(F3),
        Key::F4 => Some(F4),
        Key::F5 => Some(F5),
        Key::F6 => Some(F6),
        Key::F7 => Some(F7),
        Key::F8 => Some(F8),
        Key::F9 => Some(F9),
        Key::LeftArrow => Some(LEFT_ARROW),
        Key::MetaLeft => Some(META_LEFT),
        Key::MetaRight => Some(META_RIGHT),
        Key::Return => Some(RETURN),
        Key::RightArrow => Some(RIGHT_ARROW),
        Key::ShiftLeft => Some(SHIFT_LEFT),
        Key::ShiftRight => Some(SHIFT_RIGHT),
        Key::Space => Some(SPACE),
        Key::Tab => Some(TAB),
        Key::UpArrow => Some(UP_ARROW),
        Key::BackQuote => Some(BACK_QUOTE),
        Key::Num1 => Some(NUM1),
        Key::Num2 => Some(NUM2),
        Key::Num3 => Some(NUM3),
        Key::Num4 => Some(NUM4),
        Key::Num5 => Some(NUM5),
        Key::Num6 => Some(NUM6),
        Key::Num7 => Some(NUM7),
        Key::Num8 => Some(NUM8),
        Key::Num9 => Some(NUM9),
        Key::Num0 => Some(NUM0),
        Key::Minus => Some(MINUS),
        Key::Equal => Some(EQUAL),
        Key::KeyQ => Some(KEY_Q),
        Key::KeyW => Some(KEY_W),
        Key::KeyE => Some(KEY_E),
        Key::KeyR => Some(KEY_R),
        Key::KeyT => Some(KEY_T),
        Key::KeyY => Some(KEY_Y),
        Key::KeyU => Some(KEY_U),
        Key::KeyI => Some(KEY_I),
        Key::KeyO => Some(KEY_O),
        Key::KeyP => Some(KEY_P),
        Key::LeftBracket => Some(LEFT_BRACKET),
        Key::RightBracket => Some(RIGHT_BRACKET),
        Key::KeyA => Some(KEY_A),
        Key::KeyS => Some(KEY_S),
        Key::KeyD => Some(KEY_D),
        Key::KeyF => Some(KEY_F),
        Key::KeyG => Some(KEY_G),
        Key::KeyH => Some(KEY_H),
        Key::KeyJ => Some(KEY_J),
        Key::KeyK => Some(KEY_K),
        Key::KeyL => Some(KEY_L),
        Key::SemiColon => Some(SEMI_COLON),
        Key::Quote => Some(QUOTE),
        Key::BackSlash => Some(BACK_SLASH),
        Key::KeyZ => Some(KEY_Z),
        Key::KeyX => Some(KEY_X),
        Key::KeyC => Some(KEY_C),
        Key::KeyV => Some(KEY_V),
        Key::KeyB => Some(KEY_B),
        Key::KeyN => Some(KEY_N),
        Key::KeyM => Some(KEY_M),
        Key::Comma => Some(COMMA),
        Key::Dot => Some(DOT),
        Key::Slash => Some(SLASH),
        Key::Function => Some(FUNCTION),
        Key::Unknown(code) => code.try_into().ok(),
        _ => None,
    }
}

pub fn key_from_code(code: CGKeyCode) -> Key {
    match code {
        ALT => Key::Alt,
        ALT_GR => Key::AltGr,
        BACKSPACE => Key::Backspace,
        CAPS_LOCK => Key::CapsLock,
        CONTROL_LEFT => Key::ControlLeft,
        DOWN_ARROW => Key::DownArrow,
        ESCAPE => Key::Escape,
        F1 => Key::F1,
        F10 => Key::F10,
        F11 => Key::F11,
        F12 => Key::F12,
        F2 => Key::F2,
        F3 => Key::F3,
        F4 => Key::F4,
        F5 => Key::F5,
        F6 => Key::F6,
        F7 => Key::F7,
        F8 => Key::F8,
        F9 => Key::F9,
        LEFT_ARROW => Key::LeftArrow,
        META_LEFT => Key::MetaLeft,
        META_RIGHT => Key::MetaRight,
        RETURN => Key::Return,
        RIGHT_ARROW => Key::RightArrow,
        SHIFT_LEFT => Key::ShiftLeft,
        SHIFT_RIGHT => Key::ShiftRight,
        SPACE => Key::Space,
        TAB => Key::Tab,
        UP_ARROW => Key::UpArrow,
        BACK_QUOTE => Key::BackQuote,
        NUM1 => Key::Num1,
        NUM2 => Key::Num2,
        NUM3 => Key::Num3,
        NUM4 => Key::Num4,
        NUM5 => Key::Num5,
        NUM6 => Key::Num6,
        NUM7 => Key::Num7,
        NUM8 => Key::Num8,
        NUM9 => Key::Num9,
        NUM0 => Key::Num0,
        MINUS => Key::Minus,
        EQUAL => Key::Equal,
        KEY_Q => Key::KeyQ,
        KEY_W => Key::KeyW,
        KEY_E => Key::KeyE,
        KEY_R => Key::KeyR,
        KEY_T => Key::KeyT,
        KEY_Y => Key::KeyY,
        KEY_U => Key::KeyU,
        KEY_I => Key::KeyI,
        KEY_O => Key::KeyO,
        KEY_P => Key::KeyP,
        LEFT_BRACKET => Key::LeftBracket,
        RIGHT_BRACKET => Key::RightBracket,
        KEY_A => Key::KeyA,
        KEY_S => Key::KeyS,
        KEY_D => Key::KeyD,
        KEY_F => Key::KeyF,
        KEY_G => Key::KeyG,
        KEY_H => Key::KeyH,
        KEY_J => Key::KeyJ,
        KEY_K => Key::KeyK,
        KEY_L => Key::KeyL,
        SEMI_COLON => Key::SemiColon,
        QUOTE => Key::Quote,
        BACK_SLASH => Key::BackSlash,
        KEY_Z => Key::KeyZ,
        KEY_X => Key::KeyX,
        KEY_C => Key::KeyC,
        KEY_V => Key::KeyV,
        KEY_B => Key::KeyB,
        KEY_N => Key::KeyN,
        KEY_M => Key::KeyM,
        COMMA => Key::Comma,
        DOT => Key::Dot,
        SLASH => Key::Slash,
        FUNCTION => Key::Function,
        code => Key::Unknown(code.into()),
    }
}

//https://opensource.apple.com/source/IOHIDFamily/IOHIDFamily-308/IOHIDSystem/IOKit/hidsystem/ev_keymap.h
// https://github.com/acidanthera/MacKernelSDK/blob/master/Headers/IOKit/hidsystem/ev_keymap.h

/*
 * Special keys currently known to and understood by the system.
 * If new specialty keys are invented, extend this list as appropriate.
 * The presence of these keys in a particular implementation is not
 * guaranteed.
 */

#[allow(unused)]
const NX_NOSPECIALKEY: u32 = 0xFFFF;
const NX_KEYTYPE_SOUND_UP: u32 = 0;
const NX_KEYTYPE_SOUND_DOWN: u32 = 1;
const NX_KEYTYPE_BRIGHTNESS_UP: u32 = 2;
const NX_KEYTYPE_BRIGHTNESS_DOWN: u32 = 3;
const NX_KEYTYPE_CAPS_LOCK: u32 = 4;
#[allow(unused)]
const NX_KEYTYPE_HELP: u32 = 5;
#[allow(unused)]
const NX_POWER_KEY: u32 = 6;
const NX_KEYTYPE_MUTE: u32 = 7;
const NX_UP_ARROW_KEY: u32 = 8;
const NX_DOWN_ARROW_KEY: u32 = 9;
const NX_KEYTYPE_NUM_LOCK: u32 = 10;

#[allow(unused)]
const NX_KEYTYPE_CONTRAST_UP: u32 = 11;
#[allow(unused)]
const NX_KEYTYPE_CONTRAST_DOWN: u32 = 12;
#[allow(unused)]
const NX_KEYTYPE_LAUNCH_PANEL: u32 = 13;
#[allow(unused)]
const NX_KEYTYPE_EJECT: u32 = 14;
#[allow(unused)]
const NX_KEYTYPE_VIDMIRROR: u32 = 15;

const NX_KEYTYPE_PLAY: u32 = 16;
const NX_KEYTYPE_NEXT: u32 = 17;
const NX_KEYTYPE_PREVIOUS: u32 = 18;
#[allow(unused)]
const NX_KEYTYPE_FAST: u32 = 19;
#[allow(unused)]
const NX_KEYTYPE_REWIND: u32 = 20;

#[allow(unused)]
const NX_KEYTYPE_ILLUMINATION_UP: u32 = 21;
#[allow(unused)]
const NX_KEYTYPE_ILLUMINATION_DOWN: u32 = 22;
#[allow(unused)]
const NX_KEYTYPE_ILLUMINATION_TOGGLE: u32 = 23;

#[allow(unused)]
const NX_NUMSPECIALKEYS: u32 = 24; /* Maximum number of special keys */
#[allow(unused)]
const NX_NUM_SCANNED_SPECIALKEYS: u32 = 24; /* First 24 special keys are */
/* actively scanned in kernel */

pub fn key_from_special_key(code: u32) -> Option<Key> {
    match code {
        NX_KEYTYPE_SOUND_UP => Some(Key::VolumeUp),
        NX_KEYTYPE_SOUND_DOWN => Some(Key::VolumeDown),
        NX_KEYTYPE_MUTE => Some(Key::VolumeMute),
        NX_KEYTYPE_BRIGHTNESS_UP => Some(Key::BrightnessUp),
        NX_KEYTYPE_BRIGHTNESS_DOWN => Some(Key::BrightnessDown),
        NX_KEYTYPE_PLAY => Some(Key::PlayPause),
        NX_KEYTYPE_NEXT => Some(Key::NextTrack),
        NX_KEYTYPE_PREVIOUS => Some(Key::PreviousTrack),
        NX_KEYTYPE_CAPS_LOCK => Some(Key::CapsLock),
        NX_UP_ARROW_KEY => Some(Key::UpArrow),
        NX_DOWN_ARROW_KEY => Some(Key::DownArrow),
        NX_KEYTYPE_NUM_LOCK => Some(Key::NumLock),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::{code_from_key, key_from_code};
    #[test]
    fn test_reversible() {
        for code in 0..=65535 {
            let key = key_from_code(code);
            match code_from_key(key) {
                Some(code2) => assert_eq!(code, code2),
                None => panic!("Could not convert back code: {:?}", code),
            }
        }
    }
}
