use crate::rdev::Key;
use std::os::raw::c_uint;

macro_rules! decl_keycodes {
    ($($name:ident, $key:ident, $code:literal),*) => {
        $(
            const $name: c_uint = $code;
        )*

        pub fn code_from_key(key: Key) -> Option<c_uint> {
            match key {
                $(
                    Key::$key => Some($name),
                )*
                Key::Unknown(code) => Some(code),
                _ => None,
            }
        }

        pub fn key_from_code(code: c_uint) -> Key {
            match code {
                $(
                    $code => Key::$key,
                )*
                _ => Key::Unknown(code)
            }
        }
    };
}

decl_keycodes!(
    ALT, Alt, 64,
    ALT_GR, AltGr, 108,
    BACKSPACE, Backspace, 22,
    CAPS_LOCK, CapsLock, 66,
    CONTROL_LEFT, ControlLeft, 37,
    CONTROL_RIGHT, ControlRight, 105,
    DELETE, Delete, 119,
    DOWN_ARROW, DownArrow, 116,
    END, End, 115,
    ESCAPE, Escape, 9,
    F1, F1, 67,
    F10, F10, 76,
    F11, F11, 95,
    F12, F12, 96,
    F2, F2, 68,
    F3, F3, 69,
    F4, F4, 70,
    F5, F5, 71,
    F6, F6, 72,
    F7, F7, 73,
    F8, F8, 74,
    F9, F9, 75,
    HOME, Home, 110,
    LEFT_ARROW, LeftArrow, 113,
    META_LEFT, MetaLeft, 133,
    PAGE_DOWN, PageDown, 117,
    PAGE_UP, PageUp, 112,
    RETURN, Return, 36,
    RIGHT_ARROW, RightArrow, 114,
    SHIFT_LEFT, ShiftLeft, 50,
    SHIFT_RIGHT, ShiftRight, 62,
    SPACE, Space, 65,
    TAB, Tab, 23,
    UP_ARROW, UpArrow, 111,
    PRINT_SCREEN, PrintScreen, 107,
    SCROLL_LOCK, ScrollLock, 78,
    PAUSE, Pause, 127,
    NUM_LOCK, NumLock, 77,
    BACK_QUOTE, BackQuote, 49,
    NUM1, Num1, 10,
    NUM2, Num2, 11,
    NUM3, Num3, 12,
    NUM4, Num4, 13,
    NUM5, Num5, 14,
    NUM6, Num6, 15,
    NUM7, Num7, 16,
    NUM8, Num8, 17,
    NUM9, Num9, 18,
    NUM0, Num0, 19,
    MINUS, Minus, 20,
    EQUAL, Equal, 21,
    KEY_Q, KeyQ, 24,
    KEY_W, KeyW, 25,
    KEY_E, KeyE, 26,
    KEY_R, KeyR, 27,
    KEY_T, KeyT, 28,
    KEY_Y, KeyY, 29,
    KEY_U, KeyU, 30,
    KEY_I, KeyI, 31,
    KEY_O, KeyO, 32,
    KEY_P, KeyP, 33,
    LEFT_BRACKET, LeftBracket, 34,
    RIGHT_BRACKET, RightBracket, 35,
    KEY_A, KeyA, 38,
    KEY_S, KeyS, 39,
    KEY_D, KeyD, 40,
    KEY_F, KeyF, 41,
    KEY_G, KeyG, 42,
    KEY_H, KeyH, 43,
    KEY_J, KeyJ, 44,
    KEY_K, KeyK, 45,
    KEY_L, KeyL, 46,
    SEMI_COLON, SemiColon, 47,
    QUOTE, Quote, 48,
    BACK_SLASH, BackSlash, 51,
    INTL_BACKSLASH, IntlBackslash, 94,
    KEY_Z, KeyZ, 52,
    KEY_X, KeyX, 53,
    KEY_C, KeyC, 54,
    KEY_V, KeyV, 55,
    KEY_B, KeyB, 56,
    KEY_N, KeyN, 57,
    KEY_M, KeyM, 58,
    COMMA, Comma, 59,
    DOT, Dot, 60,
    SLASH, Slash, 61,
    INSERT, Insert, 118,
    KP_RETURN, KpReturn, 104,
    KP_MINUS, KpMinus, 82,
    KP_PLUS, KpPlus, 86,
    KP_MULTIPLY, KpMultiply, 63,
    KP_DIVIDE, KpDivide, 106,
    KP0, Kp0, 90,
    KP1, Kp1, 87,
    KP2, Kp2, 88,
    KP3, Kp3, 89,
    KP4, Kp4, 83,
    KP5, Kp5, 84,
    KP6, Kp6, 85,
    KP7, Kp7, 79,
    KP8, Kp8, 80,
    KP9, Kp9, 81,
    KP_DELETE, KpDelete, 91
);

#[cfg(test)]
mod test {
    use super::{code_from_key, key_from_code};
    #[test]
    fn test_reversible() {
        for code in 0..65636 {
            let key = key_from_code(code);
            match code_from_key(key) {
                Some(code2) => assert_eq!(code, code2),
                None => panic!("Could not convert back code: {:?}", code),
            }
        }
    }
}
