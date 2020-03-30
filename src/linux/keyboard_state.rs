extern crate x11;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_uchar, c_uint, c_void};
use std::ptr::{null, null_mut, NonNull};
use x11::xlib;

// Inspired from https://github.com/wavexx/screenkey
// But without remitting events to custom windows, instead we recreate  XKeyEvent
// from xEvent data received via xrecord.
// Other source of inspiration https://gist.github.com/baines/5a49f1334281b2685af5dcae81a6fa8a
// Needed xproto crate as x11 does not implement _xevent.
#[derive(Debug)]
pub struct KeyboardState {
    pub xic: Box<xlib::XIC>,
    pub display: Box<*mut xlib::Display>,
    keysym: Box<u64>,
    status: Box<i32>,
}
impl Drop for KeyboardState {
    fn drop(&mut self) {
        unsafe {
            xlib::XCloseDisplay(*self.display);
        }
    }
}

impl KeyboardState {
    pub fn new() -> Option<KeyboardState> {
        unsafe {
            let dpy = xlib::XOpenDisplay(null());
            NonNull::new(dpy)?; //return None if dpy is null
            let xim = xlib::XOpenIM(dpy, null_mut(), null_mut(), null_mut());
            NonNull::new(xim)?;
            let style = xlib::XIMPreeditNothing | xlib::XIMStatusNothing;
            let input_style = CString::new(xlib::XNInputStyle).expect("CString::new failed");
            let window_client = CString::new(xlib::XNClientWindow).expect("CString::new failed");

            let mut win_attr = xlib::XSetWindowAttributes {
                background_pixel: 0,
                background_pixmap: 0,
                border_pixel: 0,
                border_pixmap: 0,
                bit_gravity: 0,
                win_gravity: 0,
                backing_store: 0,
                backing_planes: 0,
                backing_pixel: 0,
                event_mask: 0,
                save_under: 0,
                do_not_propagate_mask: 0,
                override_redirect: 0,
                colormap: 0,
                cursor: 0,
            };

            let window = xlib::XCreateWindow(
                dpy,
                xlib::XDefaultRootWindow(dpy),
                0,
                0,
                1,
                1,
                0,
                xlib::CopyFromParent,
                xlib::InputOnly as c_uint,
                null_mut(),
                0,
                &mut win_attr,
            );

            let xic = xlib::XCreateIC(
                xim,
                window_client.as_ptr(),
                window,
                input_style.as_ptr(),
                style,
                null::<c_void>(),
            );
            NonNull::new(xic)?;
            Some(KeyboardState {
                xic: Box::new(xic),
                display: Box::new(dpy),
                keysym: Box::new(0),
                status: Box::new(0),
            })
        }
    }

    pub unsafe fn name_from_code(&mut self, xevent: &xproto::_xEvent) -> Option<String> {
        if self.display.is_null() || self.xic.is_null() {
            println!("We don't seem to have a display or a xic");
            return None;
        }
        const BUF_LEN: usize = 4;
        let mut buf = [0 as c_uchar; BUF_LEN];
        let mut xkey = xlib::XKeyEvent {
            display: *self.display,
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
            *self.xic,
            &mut xkey,
            buf.as_mut_ptr() as *mut c_char,
            BUF_LEN as c_int,
            &mut *self.keysym,
            &mut *self.status,
        );

        let len = buf.iter().position(|ch| ch == &0).unwrap_or(BUF_LEN);
        String::from_utf8(buf[..len].to_vec()).ok()
    }
}
