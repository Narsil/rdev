use crate::linux::keyboard::Keyboard;
use crate::linux::keycodes::key_from_code;
use crate::rdev::{Button, Event, EventType};
use std::convert::TryInto;
use std::os::raw::{c_int, c_uchar, c_uint};
use std::ptr::null;
use std::time::SystemTime;
use x11::xlib;

pub const TRUE: c_int = 1;
pub const FALSE: c_int = 0;

pub fn convert_event(code: c_uchar, type_: c_int, x: f64, y: f64) -> Option<EventType> {
    match type_ {
        xlib::KeyPress => {
            let key = key_from_code(code.into());
            Some(EventType::KeyPress(key))
        }
        xlib::KeyRelease => {
            let key = key_from_code(code.into());
            Some(EventType::KeyRelease(key))
        }
        // Xlib does not implement wheel events left and right afaik.
        // But MacOS does, so we need to acknowledge the larger event space.
        xlib::ButtonPress => match code {
            1 => Some(EventType::ButtonPress(Button::Left)),
            2 => Some(EventType::ButtonPress(Button::Middle)),
            3 => Some(EventType::ButtonPress(Button::Right)),
            4 => Some(EventType::Wheel {
                delta_y: 1,
                delta_x: 0,
            }),
            5 => Some(EventType::Wheel {
                delta_y: -1,
                delta_x: 0,
            }),
            #[allow(clippy::identity_conversion)]
            code => Some(EventType::ButtonPress(Button::Unknown(code.into()))),
        },
        xlib::ButtonRelease => match code {
            1 => Some(EventType::ButtonRelease(Button::Left)),
            2 => Some(EventType::ButtonRelease(Button::Middle)),
            3 => Some(EventType::ButtonRelease(Button::Right)),
            4 | 5 => None,
            #[allow(clippy::identity_conversion)]
            _ => Some(EventType::ButtonRelease(Button::Unknown(code.into()))),
        },
        xlib::MotionNotify => Some(EventType::MouseMove { x, y }),
        _ => None,
    }
}

pub fn convert(code: c_uint, state: c_uint, type_: c_int, x: f64, y: f64) -> Option<Event> {
    unsafe {
        let event_type = convert_event(code as c_uchar, type_, x, y)?;
        let name = match event_type {
            EventType::KeyPress(_) => {
                Keyboard::new().map(|mut kboard| kboard.name_from_code(code, state))
            }
            _ => None,
        }
        .flatten();
        Some(Event {
            event_type,
            time: SystemTime::now(),
            name,
        })
    }
}

pub struct Display {
    display: *mut xlib::Display,
}

impl Display {
    pub fn new() -> Option<Display> {
        unsafe {
            let display = xlib::XOpenDisplay(null());
            if display.is_null() {
                return None;
            }
            Some(Display { display })
        }
    }

    pub fn get_size(&self) -> Option<(u64, u64)> {
        unsafe {
            let screen_ptr = xlib::XDefaultScreenOfDisplay(self.display);
            if screen_ptr.is_null() {
                return None;
            }
            let screen = *screen_ptr;
            Some((
                screen.width.try_into().ok()?,
                screen.height.try_into().ok()?,
            ))
        }
    }
}
impl Drop for Display {
    fn drop(&mut self) {
        unsafe {
            xlib::XCloseDisplay(self.display);
        }
    }
}
