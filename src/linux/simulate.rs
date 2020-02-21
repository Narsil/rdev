use crate::linux::keycodes::code_from_key;
use crate::rdev::{Button, EventType, SimulateError};
use std::ptr::null;
use x11::xlib;
use x11::xtest;
static TRUE: i32 = 1;
static FALSE: i32 = 0;

unsafe fn send_native(event_type: &EventType, display: *mut xlib::Display) -> Result<(), ()> {
    match event_type {
        EventType::KeyPress(key) => {
            if let Some(code) = code_from_key(key) {
                xtest::XTestFakeKeyEvent(display, code, TRUE, 0);
                return Ok(());
            }
            Err(())
        }
        EventType::KeyRelease(key) => {
            if let Some(code) = code_from_key(key) {
                xtest::XTestFakeKeyEvent(display, code, FALSE, 0);
                return Ok(());
            }
            Err(())
        }
        EventType::ButtonPress(button) => {
            match button {
                Button::Left => xtest::XTestFakeButtonEvent(display, 1, TRUE, 0),
                Button::Middle => xtest::XTestFakeButtonEvent(display, 2, TRUE, 0),
                Button::Right => xtest::XTestFakeButtonEvent(display, 3, TRUE, 0),
                Button::Unknown(code) => {
                    xtest::XTestFakeButtonEvent(display, *code as u32, TRUE, 0)
                }
            };
            Ok(())
        }
        EventType::ButtonRelease(button) => {
            match button {
                Button::Left => xtest::XTestFakeButtonEvent(display, 1, FALSE, 0),
                Button::Middle => xtest::XTestFakeButtonEvent(display, 2, FALSE, 0),
                Button::Right => xtest::XTestFakeButtonEvent(display, 3, FALSE, 0),
                Button::Unknown(code) => {
                    xtest::XTestFakeButtonEvent(display, *code as u32, FALSE, 0)
                }
            };
            Ok(())
        }
        EventType::MouseMove { x, y } => {
            xtest::XTestFakeMotionEvent(display, 0, *x as i32, *y as i32, 0);
            // unsafe {
            //     xlib::XWarpPointer(display, 0, root, 0, 0, 0, 0, *x as i32, *y as i32);
            // }
            Ok(())
        }
        EventType::Wheel { delta_y, .. } => {
            let code = if *delta_y > 0 { 4 } else { 5 };
            xtest::XTestFakeButtonEvent(display, code, TRUE, 0);
            xtest::XTestFakeButtonEvent(display, code, FALSE, 0);
            Ok(())
        }
    }
}

pub fn simulate(event_type: &EventType) -> Result<(), SimulateError> {
    unsafe {
        let dpy = xlib::XOpenDisplay(null());
        if dpy.is_null() {
            panic!("can't open display");
        }
        match send_native(event_type, dpy) {
            Ok(_) => {
                xlib::XFlush(dpy);
                xlib::XSync(dpy, 0);

                Ok(())
            }
            Err(_) => Err(SimulateError),
        }
    }
}
