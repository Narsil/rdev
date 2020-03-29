use std::convert::TryInto;
use std::ptr::null;
use x11::xlib;

pub fn display_size() -> (u64, u64) {
    unsafe {
        let dpy = xlib::XOpenDisplay(null());
        if dpy.is_null() {
            panic!("We can't connect to X server");
        }
        let screen = xlib::XDefaultScreenOfDisplay(dpy);
        if screen.is_null() {
            panic!("We can't connect to screen of X server");
        }
        let res = (
            (*screen).width.try_into().unwrap(),
            (*screen).height.try_into().unwrap(),
        );

        xlib::XCloseDisplay(dpy);

        res
    }
}
