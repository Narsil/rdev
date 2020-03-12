use x11::xlib;

fn display_size() -> (u64, u64) {
    let dpy = xlib::XOpenDisplay(null());
    if dpy.is_null() {
        panic!("We can't connect to X server");
    }
    let screen = xlib::DefaultScreenOfDisplay(dpy);
    if screen.is_null() {
        panic!("We can't connect to screen of X server");
    }
    (screen.width, screen.height)
}
