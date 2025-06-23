use crate::rdev::Key;
use input_linux::Key as UKey;
use std::os::raw::c_uint;

macro_rules! decl_keycodes {
    ($($key:ident, $code:literal),*) => {
        pub const fn code_from_key(key: Key) -> Option<c_uint> {
            match key {
                $(
                    Key::$key => Some($code),
                )*
                Key::Unknown(code) => Some(code),
                _ => None,
            }
        }

        pub const fn key_from_code(code: c_uint) -> Key {
            match code {
                $(
                    $code => Key::$key,
                )*
                _ => Key::Unknown(code)
            }
        }
    };
}

macro_rules! decl_keycodes_uinput {
    ($($key:path, $ukey:path),*) => {
        pub const fn ukey_from_key(key: Key) -> Option<UKey> {
            match key {
                $(
                    $key => Some($ukey),
                )*
                Key::Unknown(code) => Some(unsafe { std::mem::transmute::<u16, UKey>(code as u16) }),
                _ => None,
            }
        }

        #[cfg(test)]
        pub const fn key_from_ukey(code: UKey) -> Key {
            match code {
                $(
                    $ukey => $key,
                )*
                ukey => Key::Unknown(unsafe { std::mem::transmute::<UKey, u16>(ukey) } as u32),
            }
        }
    };
}

#[rustfmt::skip]
decl_keycodes!(
    Alt, 56,
    AltGr, 126,
    Backspace, 14,
    CapsLock, 58,
    ControlLeft, 29,
    ControlRight, 97,
    Delete, 110,
    DownArrow, 108,
    End, 107,
    Escape, 1,
    F1, 59,
    F2, 60,
    F3, 61,
    F4, 62,
    F5, 63,
    F6, 64,
    F7, 65,
    F8, 66,
    F9, 67,
    F10, 68,
    F11, 87,
    F12, 88,
    Home, 102,
    LeftArrow, 105,
    MetaLeft, 125,
    PageDown, 109,
    PageUp, 104,
    Return, 28,
    RightArrow, 106,
    ShiftLeft, 42,
    ShiftRight, 54,
    Space, 57,
    Tab, 15,
    UpArrow, 103,
    PrintScreen, 99,
    ScrollLock, 70,
    Pause, 119,
    NumLock, 69,
    BackQuote, 41,
    Num1, 2,
    Num2, 3,
    Num3, 4,
    Num4, 5,
    Num5, 6,
    Num6, 7,
    Num7, 8,
    Num8, 9,
    Num9, 10,
    Num0, 11,
    Minus, 12,
    Equal, 13,
    KeyQ, 16,
    KeyW, 17,
    KeyE, 18,
    KeyR, 19,
    KeyT, 20,
    KeyY, 21,
    KeyU, 22,
    KeyI, 23,
    KeyO, 24,
    KeyP, 25,
    LeftBracket, 26,
    RightBracket, 27,
    KeyA, 30,
    KeyS, 31,
    KeyD, 32,
    KeyF, 33,
    KeyG, 34,
    KeyH, 35,
    KeyJ, 36,
    KeyK, 37,
    KeyL, 38,
    SemiColon, 39,
    Quote, 40,
    BackSlash, 43,
    IntlBackslash, 86,
    KeyZ, 44,
    KeyX, 45,
    KeyC, 46,
    KeyV, 47,
    KeyB, 48,
    KeyN, 49,
    KeyM, 50,
    Comma, 51,
    Dot, 52,
    Slash, 53,
    Insert, 111,
    KpReturn, 96,
    KpMinus, 74,
    KpPlus, 78,
    KpMultiply, 55,
    KpDivide, 98,
    Kp0, 82,
    Kp1, 79,
    Kp2, 80,
    Kp3, 81,
    Kp4, 75,
    Kp5, 76,
    Kp6, 77,
    Kp7, 71,
    Kp8, 72,
    Kp9, 73,
    KpDelete, 83,
    VolumeMute, 113,
    VolumeDown, 114,
    VolumeUp, 115,
    NextTrack, 163,
    PlayPause, 164,
    PreviousTrack, 165,
    PlayCd, 200,
    Function, 464
);

#[rustfmt::skip]
decl_keycodes_uinput!(
    Key::Escape , UKey::Esc,
    Key::Num1 , UKey::Num1,
    Key::Num2 , UKey::Num2,
    Key::Num3 , UKey::Num3,
    Key::Num4 , UKey::Num4,
    Key::Num5 , UKey::Num5,
    Key::Num6 , UKey::Num6,
    Key::Num7 , UKey::Num7,
    Key::Num8 , UKey::Num8,
    Key::Num9 , UKey::Num9,
    Key::Num0 , UKey::Num0,
    Key::Minus , UKey::Minus,
    Key::Equal , UKey::Equal,
    Key::Backspace , UKey::Backspace,
    Key::Tab , UKey::Tab,
    Key::KeyQ , UKey::Q,
    Key::KeyW , UKey::W,
    Key::KeyE , UKey::E,
    Key::KeyR , UKey::R,
    Key::KeyT , UKey::T,
    Key::KeyY , UKey::Y,
    Key::KeyU , UKey::U,
    Key::KeyI , UKey::I,
    Key::KeyO , UKey::O,
    Key::KeyP , UKey::P,
    Key::LeftBracket , UKey::LeftBrace,
    Key::RightBracket , UKey::RightBrace,
    Key::Return , UKey::Enter,
    Key::ControlLeft , UKey::LeftCtrl,
    Key::KeyA , UKey::A,
    Key::KeyS , UKey::S,
    Key::KeyD , UKey::D,
    Key::KeyF , UKey::F,
    Key::KeyG , UKey::G,
    Key::KeyH , UKey::H,
    Key::KeyJ , UKey::J,
    Key::KeyK , UKey::K,
    Key::KeyL , UKey::L,
    Key::SemiColon , UKey::Semicolon,
    Key::Quote , UKey::Apostrophe,
    Key::BackQuote , UKey::Grave,
    Key::ShiftLeft , UKey::LeftShift,
    Key::BackSlash , UKey::Backslash,
    Key::KeyZ , UKey::Z,
    Key::KeyX , UKey::X,
    Key::KeyC , UKey::C,
    Key::KeyV , UKey::V,
    Key::KeyB , UKey::B,
    Key::KeyN , UKey::N,
    Key::KeyM , UKey::M,
    Key::Comma , UKey::Comma,
    Key::Dot , UKey::Dot,
    Key::Slash , UKey::Slash,
    Key::ShiftRight , UKey::RightShift,
    Key::KpMultiply , UKey::KpAsterisk,
    Key::Alt , UKey::LeftAlt,
    Key::Space , UKey::Space,
    Key::CapsLock , UKey::CapsLock,
    Key::F1 , UKey::F1,
    Key::F2 , UKey::F2,
    Key::F3 , UKey::F3,
    Key::F4 , UKey::F4,
    Key::F5 , UKey::F5,
    Key::F6 , UKey::F6,
    Key::F7 , UKey::F7,
    Key::F8 , UKey::F8,
    Key::F9 , UKey::F9,
    Key::F10 , UKey::F10,
    Key::NumLock , UKey::NumLock,
    Key::ScrollLock , UKey::ScrollLock,
    Key::Kp7 , UKey::Kp7,
    Key::Kp8 , UKey::Kp8,
    Key::Kp9 , UKey::Kp9,
    Key::KpMinus , UKey::KpMinus,
    Key::Kp4 , UKey::Kp4,
    Key::Kp5 , UKey::Kp5,
    Key::Kp6 , UKey::Kp6,
    Key::KpPlus , UKey::KpPlus,
    Key::Kp1 , UKey::Kp1,
    Key::Kp2 , UKey::Kp2,
    Key::Kp3 , UKey::Kp3,
    Key::Kp0 , UKey::Kp0,
    // Key::KpDelete , UKey::Delete,
    Key::F11 , UKey::F11,
    Key::F12 , UKey::F12,
    Key::KpReturn , UKey::KpEnter,
    Key::ControlRight , UKey::RightCtrl,
    Key::KpDivide , UKey::KpSlash,
    Key::AltGr , UKey::RightAlt,
    Key::Home , UKey::Home,
    Key::UpArrow , UKey::Up,
    Key::PageUp , UKey::PageUp,
    Key::LeftArrow , UKey::Left,
    Key::RightArrow , UKey::Right,
    Key::End , UKey::End,
    Key::DownArrow , UKey::Down,
    Key::PageDown , UKey::PageDown,
    Key::Insert , UKey::Insert,
    Key::Delete , UKey::Delete,
    Key::Pause , UKey::Pause,
    Key::MetaLeft , UKey::LeftMeta,
    Key::PrintScreen , UKey::Print,
    Key::PlayPause, UKey::PlayPause,
    Key::PlayCd, UKey::PlayCD,
    Key::VolumeMute, UKey::Mute,
    Key::VolumeDown, UKey::VolumeDown,
    Key::VolumeUp, UKey::VolumeUp,
    Key::NextTrack, UKey::NextSong,
    Key::PreviousTrack, UKey::PreviousSong
    //Key::IntlBackslash , UKey::Backslash
);

#[cfg(test)]
mod test {
    use super::{UKey, code_from_key, key_from_code, key_from_ukey, ukey_from_key};
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

    #[test]
    fn test_reversible_uinput() {
        for code in 0..u16::MAX {
            let ukey = unsafe { std::mem::transmute::<u16, UKey>(code) };
            let key = key_from_ukey(ukey);
            if let Some(ukey2) = ukey_from_key(key) {
                assert_eq!(ukey, ukey2, "Key conversion mismatch for code: {}", code);
            }
        }
    }
}
