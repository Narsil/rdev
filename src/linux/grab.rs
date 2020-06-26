use crate::rdev::{Button, Event, EventType, GrabCallback, GrabError, Key};
use gelo::evdev_rs::{
    enums::{EventCode, EV_KEY, EV_REL},
    InputEvent, TimeVal,
};
use gelo::{filter_map_events, GrabStatus};

use std::time::SystemTime;

macro_rules! convert_keys {
    ($($ev_key:ident, $rdev_key:ident),*) => {
        //TODO: make const when rust lang issue #49146 is fixed
        #[allow(unreachable_patterns)]
        fn evdev_key_to_rdev_key(key: &EV_KEY) -> Option<Key> {
            match key {
                $(
                    EV_KEY::$ev_key => Some(Key::$rdev_key),
                )*
                _ => None,
            }
        }

        //TODO: make const when rust lang issue #49146 is fixed
        fn rdev_key_to_evdev_key(key: &Key) -> Option<EV_KEY> {
            match key {
                $(
                    Key::$rdev_key => Some(EV_KEY::$ev_key),
                )*
                _ => None
            }
        }
    };
}

macro_rules! convert_buttons {
    ($($ev_key:ident, $rdev_key:ident),*) => {
        //TODO: make const when rust lang issue #49146 is fixed
        fn evdev_key_to_rdev_button(key: &EV_KEY) -> Option<Button> {
            match key {
                $(
                    EV_KEY::$ev_key => Some(Button::$rdev_key),
                )*
                _ => None,
            }
        }

        //TODO: make const when rust lang issue #49146 is fixed
        #[allow(unreachable_patterns)]
        fn rdev_button_to_evdev_key(event: &Button) -> Option<EV_KEY> {
            match event {
                $(
                    Button::$rdev_key => Some(EV_KEY::$ev_key),
                )*
                _ => None
            }
        }
    };
}

#[rustfmt::skip]
convert_buttons!(
    BTN_LEFT, Left,
    BTN_RIGHT, Right,
    BTN_MIDDLE, Middle
);

//TODO: IntlBackslash, kpDelete
#[rustfmt::skip]
convert_keys!(
    KEY_ESC, Escape,
    KEY_1, Num1,
    KEY_2, Num2,
    KEY_3, Num3,
    KEY_4, Num4,
    KEY_5, Num5,
    KEY_6, Num6,
    KEY_7, Num7,
    KEY_8, Num8,
    KEY_9, Num9,
    KEY_0, Num0,
    KEY_MINUS, Minus,
    KEY_EQUAL, Equal,
    KEY_BACKSPACE, Backspace,
    KEY_TAB, Tab,
    KEY_Q, KeyQ,
    KEY_W, KeyW,
    KEY_E, KeyE,
    KEY_R, KeyR,
    KEY_T, KeyT,
    KEY_Y, KeyY,
    KEY_U, KeyU,
    KEY_I, KeyI,
    KEY_O, KeyO,
    KEY_P, KeyP,
    KEY_LEFTBRACE, LeftBracket,
    KEY_RIGHTBRACE, RightBracket,
    KEY_ENTER, Return,
    KEY_LEFTCTRL, ControlLeft,
    KEY_A, KeyA,
    KEY_S, KeyS,
    KEY_D, KeyD,
    KEY_F, KeyF,
    KEY_G, KeyG,
    KEY_H, KeyH,
    KEY_J, KeyJ,
    KEY_K, KeyK,
    KEY_L, KeyL,
    KEY_SEMICOLON, SemiColon,
    KEY_APOSTROPHE, Quote,
    KEY_GRAVE, BackQuote,
    KEY_LEFTSHIFT, ShiftLeft,
    KEY_BACKSLASH, BackSlash,
    KEY_Z, KeyZ,
    KEY_X, KeyX,
    KEY_C, KeyC,
    KEY_V, KeyV,
    KEY_B, KeyB,
    KEY_N, KeyN,
    KEY_M, KeyM,
    KEY_COMMA, Comma,
    KEY_DOT, Dot,
    KEY_SLASH, Slash,
    KEY_RIGHTSHIFT, ShiftRight,
    KEY_KPASTERISK , KpMultiply,
    KEY_LEFTALT, Alt,
    KEY_SPACE, Space,
    KEY_CAPSLOCK, CapsLock,
    KEY_F1, F1,
    KEY_F2, F2,
    KEY_F3, F3,
    KEY_F4, F4,
    KEY_F5, F5,
    KEY_F6, F6,
    KEY_F7, F7,
    KEY_F8, F8,
    KEY_F9, F9,
    KEY_F10, F10,
    KEY_NUMLOCK, NumLock,
    KEY_SCROLLLOCK, ScrollLock,
    KEY_KP7, Kp7,
    KEY_KP8, Kp8,
    KEY_KP9, Kp9,
    KEY_KPMINUS, KpMinus,
    KEY_KP4, Kp4,
    KEY_KP5, Kp5,
    KEY_KP6, Kp6,
    KEY_KPPLUS, KpPlus,
    KEY_KP1, Kp1,
    KEY_KP2, Kp2,
    KEY_KP3, Kp3,
    KEY_KP0, Kp0,
    KEY_F11, F11,
    KEY_F12, F12,
    KEY_KPENTER, KpReturn,
    KEY_RIGHTCTRL, ControlRight,
    KEY_KPSLASH, KpDivide,
    KEY_RIGHTALT, AltGr,
    KEY_HOME , Home,
    KEY_UP, UpArrow,
    KEY_PAGEUP, PageUp,
    KEY_LEFT, LeftArrow,
    KEY_RIGHT, RightArrow,
    KEY_END, End,
    KEY_DOWN, DownArrow,
    KEY_PAGEDOWN, PageDown,
    KEY_INSERT, Insert,
    KEY_DELETE, Delete,
    KEY_PAUSE, Pause,
    KEY_LEFTMETA, MetaLeft,
    KEY_RIGHTMETA, MetaRight,
    KEY_PRINT, PrintScreen,
    // KpDelete behaves like normal Delete most of the time
    KEY_DELETE, KpDelete,
    // Linux doesn't have an IntlBackslash key
    KEY_BACKSLASH, IntlBackslash
);

fn evdev_event_to_rdev_event(event: &InputEvent) -> Option<EventType> {
    match &event.event_code {
        EventCode::EV_KEY(key) => {
            if let Some(button) = evdev_key_to_rdev_button(&key) {
                // first check if pressed key is a mouse button
                match event.value {
                    0 => Some(EventType::ButtonRelease(button)),
                    _ => Some(EventType::ButtonPress(button)),
                }
            } else if let Some(key) = evdev_key_to_rdev_key(&key) {
                // check if pressed key is a keyboard key
                match event.value {
                    0 => Some(EventType::KeyRelease(key)),
                    _ => Some(EventType::KeyPress(key)),
                }
            } else {
                // if neither mouse button nor keyboard key, return none
                None
            }
        }
        EventCode::EV_REL(mouse) => match mouse {
            EV_REL::REL_X => Some(EventType::MouseMove {
                x: event.value as f64,
                y: 0.0,
            }),
            EV_REL::REL_Y => Some(EventType::MouseMove {
                x: 0.0,
                y: event.value as f64,
            }),
            EV_REL::REL_HWHEEL => Some(EventType::Wheel {
                delta_x: event.value.into(),
                delta_y: 0,
            }),
            EV_REL::REL_WHEEL => Some(EventType::Wheel {
                delta_x: 0,
                delta_y: event.value.into(),
            }),
            // Other EV_REL events cannot be represented by rdev
            _ => return None,
        },
        // Other event_codes cannot be represented by rdev,
        // and some never will e.g. EV_SYN
        _ => return None,
    }
}

fn rdev_event_to_evdev_event(event: &EventType, time: &TimeVal) -> Option<InputEvent> {
    match event {
        EventType::KeyPress(key) => {
            let key = rdev_key_to_evdev_key(&key)?;
            Some(InputEvent::new(&time, &EventCode::EV_KEY(key), 1))
        }
        EventType::KeyRelease(key) => {
            let key = rdev_key_to_evdev_key(&key)?;
            Some(InputEvent::new(&time, &EventCode::EV_KEY(key), 0))
        }
        EventType::ButtonPress(button) => {
            let button = rdev_button_to_evdev_key(&button)?;
            Some(InputEvent::new(&time, &EventCode::EV_KEY(button), 1))
        }
        EventType::ButtonRelease(button) => {
            let button = rdev_button_to_evdev_key(&button)?;
            Some(InputEvent::new(&time, &EventCode::EV_KEY(button), 0))
        }
        EventType::MouseMove { x, y } => {
            let (x, y) = (*x as i32, *y as i32);
            //TODO allow both x and y movements simultaneously
            if x != 0 {
                Some(InputEvent::new(&time, &EventCode::EV_REL(EV_REL::REL_X), x))
            } else {
                Some(InputEvent::new(&time, &EventCode::EV_REL(EV_REL::REL_Y), y))
            }
        }
        EventType::Wheel { delta_x, delta_y } => {
            let (x, y) = (*delta_x as i32, *delta_y as i32);
            //TODO allow both x and y movements simultaneously
            if x != 0 {
                Some(InputEvent::new(
                    &time,
                    &EventCode::EV_REL(EV_REL::REL_HWHEEL),
                    x,
                ))
            } else {
                Some(InputEvent::new(
                    &time,
                    &EventCode::EV_REL(EV_REL::REL_WHEEL),
                    y,
                ))
            }
        }
    }
}

pub fn grab(callback: GrabCallback) -> Result<(), GrabError> {
    filter_map_events(|event| {
        let event_type = match evdev_event_to_rdev_event(&event) {
            Some(rdev_event) => rdev_event,
            // If we can't convert event, simulate it
            None => return (Some(event), GrabStatus::Continue)
        };
        let rdev_event = Event {
            time: SystemTime::now(),
            /* TODO: generate a name here */
            name: None,
            event_type,
        };
        let new_event = match callback(rdev_event) {
            Some(rdev_event) => rdev_event,
            // callback returns None, swallow the event
            None => return (None, GrabStatus::Continue)
        };
        match rdev_event_to_evdev_event(&new_event.event_type, &event.time) {
            Some(evdev_event) => (Some(evdev_event), GrabStatus::Continue),
            // If we can't convert the event back, send the original event
            None => (Some(event), GrabStatus::Continue)
        }
    }).map_err(|_| GrabError::SimulateError)?;
    Ok(())
}
