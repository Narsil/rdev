use crate::linux::common::{FALSE, TRUE};
use crate::rdev::{Event, EventType, GrabCallback, GrabError, Key};
use std::mem::zeroed;
use std::ptr::null;
use std::time::SystemTime;
use x11::{xlib, xtest};

fn convert(_ev: &xlib::XEvent) -> Event {
    // TODO
    Event {
        event_type: EventType::KeyPress(Key::KeyS),
        name: None,
        time: SystemTime::now(),
    }
}

pub fn grab(callback: GrabCallback) -> Result<(), GrabError> {
    unsafe {
        let dpy = xlib::XOpenDisplay(null());
        if dpy.is_null() {
            return Err(GrabError::MissingDisplayError);
        }

        let root = xlib::XDefaultRootWindow(dpy);

        let res = xlib::XGrabKeyboard(
            dpy,
            root,
            TRUE,
            xlib::GrabModeAsync,
            xlib::GrabModeAsync,
            xlib::CurrentTime,
        );
        println!("Grab first keyboard {}", res);
        let mut ev: xlib::XEvent = zeroed();
        loop {
            println!("Listening 1 event");
            xlib::XNextEvent(dpy, &mut ev);
            println!("Got 1 event {:?}", ev);
            let event_type = convert(&ev);
            if let Some(_event) = callback(event_type) {
                let r = xlib::XUngrabKeyboard(dpy, xlib::CurrentTime);
                println!("Ungrab keyboard {}", r);
                if ev.type_ == xlib::KeyPress {
                    // xlib::XSendEvent(dpy, xlib::InputFocus as u64, TRUE, xlib::KeyPressMask, &mut ev);
                    println!("Sending keypress {}", ev.key.keycode);
                    xtest::XTestFakeKeyEvent(dpy, ev.key.keycode, TRUE, 0);
                } else {
                    // xlib::XSendEvent(dpy, xlib::InputFocus as u64, TRUE, xlib::KeyReleaseMask, &mut ev);
                    println!("Sending keyrelease {}", ev.key.keycode);
                    xtest::XTestFakeKeyEvent(dpy, ev.key.keycode, FALSE, 0);
                }
                xlib::XFlush(dpy);
                xlib::XSync(dpy, 0);
                let res = xlib::XGrabKeyboard(
                    dpy,
                    root,
                    TRUE,
                    xlib::GrabModeAsync,
                    xlib::GrabModeAsync,
                    xlib::CurrentTime,
                );
                println!("Regrab keyboard {}", res);
            }
        }
    }
}
