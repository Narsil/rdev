extern crate x11;
use std::ffi::CString;
use std::os::raw::{c_char, c_uchar, c_void};
use std::ptr::{null, null_mut};
use x11::xlib;

// Inspired from https://github.com/wavexx/screenkey
// But without remitting events to custom windows, instead we recreate  XKeyEvent
// from xEvent data received via xrecord.
// Other source of inspiration https://gist.github.com/baines/5a49f1334281b2685af5dcae81a6fa8a
// Needed xproto crate as x11 does not implement _xevent.

pub struct KeyboardState {
    pub xic: xlib::XIC,
    pub display: *mut xlib::Display,
    keysym: u64,
    status: i32,
}

impl KeyboardState {
    pub fn new() -> Result<KeyboardState, ()> {
        unsafe {
            let dpy = xlib::XOpenDisplay(null());
            if dpy.is_null() {
                return Err(());
            }
            let xim = xlib::XOpenIM(dpy, null_mut(), null_mut(), null_mut());
            if xim.is_null() {
                return Err(());
            }
            let style = xlib::XIMPreeditNothing | xlib::XIMStatusNothing;
            let input_style = CString::new(xlib::XNInputStyle).expect("CString::new failed");

            let xic = xlib::XCreateIC(xim, input_style.as_ptr(), style, null::<c_void>());
            if xic.is_null() {
                return Err(());
            }

            Ok(KeyboardState {
                xic,
                display: dpy,
                keysym: 0,
                status: 0,
            })
        }
    }
    pub unsafe fn name_from_code(&mut self, xevent: &mut xproto::_xEvent) -> Option<String> {
        let mut buf: [c_uchar; 4] = [0; 4];
        let length = buf.len() as i32;
        let mut xkey = xlib::XKeyEvent {
            display: self.display,
            root: 0,
            window: 0,
            subwindow: 0,
            x: 0,
            y: 0,
            x_root: 0,
            y_root: 0,
            state: xevent.u.keyButtonPointer.as_ref().state as u32,
            keycode: xevent.u.u.as_ref().detail as u32,
            same_screen: 0,
            send_event: 0,
            serial: 0,
            type_: xlib::KeyPress,
            time: xlib::CurrentTime,
        };
        let _ret = xlib::Xutf8LookupString(
            self.xic,
            &mut xkey,
            &mut buf as *mut _ as *mut c_char,
            length,
            &mut self.keysym as *mut u64,
            &mut self.status as *mut i32,
        );

        let mut len = 0;
        for c in buf.iter() {
            if *c == 0 {
                break;
            }
            len += 1;
        }

        match String::from_utf8(buf[0..len].to_vec()) {
            Ok(string) => Some(string),
            Err(_) => None,
        }
    }
}
