// use crate::linux::common::{convert, FALSE, TRUE};
// use crate::linux::simulate::simulate;
use crate::rdev::{GrabCallback, GrabError};
// use std::mem::{transmute, zeroed};
// use std::os::raw::c_int;
// use std::ptr::null;
// use std::thread::sleep;
// use std::time::{Duration, SystemTime};
// use x11::{xinput2, xlib};

// unsafe fn convert_event(ev: &xinput2::XIDeviceEvent) -> Option<Event> {
//     let (code, state, x, y) = (ev.detail, ev.mods.effective, ev.root_x, ev.root_y);
//     convert(code as u32, state as u32, ev.evtype, x as f64, y as f64)
// }
//
pub fn grab(_callback: GrabCallback) -> Result<(), GrabError> {
    Err(GrabError::LinuxNotSupported)
    // unsafe {
    //     let dpy = xlib::XOpenDisplay(null());
    //     if dpy.is_null() {
    //         return Err(GrabError::MissingDisplayError);
    //     }

    //     let root = xlib::XDefaultRootWindow(dpy);

    //     // let mode = xlib::GrabModeAsync;
    //     let mode = xinput2::XIGrabModeAsync;
    //     println!("Before grab");
    //     let mut eventmask: xinput2::XIEventMask = zeroed();
    //     let mut mask: [u8; 1] = [0];
    //     eventmask.mask_len = 1;
    //     xinput2::XISetMask(&mut mask, xinput2::XI_ButtonPress);
    //     xinput2::XISetMask(&mut mask, xinput2::XI_ButtonRelease);
    //     xinput2::XISetMask(&mut mask, xinput2::XI_Motion);
    //     xinput2::XISetMask(&mut mask, xinput2::XI_KeyPress);
    //     xinput2::XISetMask(&mut mask, xinput2::XI_KeyRelease);
    //     // eventmask.deviceid = xinput2::XIAllDevices;
    //     eventmask.deviceid = 3;
    //     eventmask.mask = &mut mask[0];
    //     println!("mask ready");
    //     println!("eventmask {:?} ", eventmask);
    //     let mut grabmodifiers: xinput2::XIGrabModifiers = zeroed();
    //     // grabmodifiers.modifiers = xlib::AnyModifier as i32;
    //     grabmodifiers.modifiers = 0;
    //     println!(
    //         "\n Grab keycode mask : {:p}\n modifiers: {:p}",
    //         &eventmask, &grabmodifiers
    //     );
    //     println!("Display {:p}", dpy);
    //     println!("Root {:p}", &root);
    //     println!("mode {:p}", &mode);
    //     println!("false {:p}", &FALSE);
    //     xinput2::XIGrabKeycode(
    //         dpy,
    //         xinput2::XIAllMasterDevices,
    //         xinput2::XIAnyKeycode,
    //         root,
    //         mode,
    //         mode,
    //         FALSE,
    //         &mut eventmask,
    //         1,
    //         &mut grabmodifiers,
    //     );
    //     println!("After grab");
    //     // println!("Grab first keyboard {}", res);
    //     let mut ev: xlib::XEvent = zeroed();
    //     loop {
    //         println!("Waiting for event");
    //         xlib::XNextEvent(dpy, &mut ev);
    //         let mut cookie = ev.generic_event_cookie;
    //         xlib::XGetEventData(dpy, &mut cookie);
    //         // TODO 131 by correct get query extension
    //         if (cookie.type_ == xlib::GenericEvent && cookie.extension == 131) {
    //             let devev: &mut xinput2::XIDeviceEvent = transmute(cookie.data);
    //             println!("Devev {:?}", devev);
    //             if let Some(in_event) = convert_event(&devev) {
    //                 if let Some(out_event) = callback(in_event) {
    //                     xinput2::XIUngrabKeycode(
    //                         dpy,
    //                         xinput2::XIAllMasterDevices,
    //                         xinput2::XIAnyKeycode,
    //                         root,
    //                         1,
    //                         &mut grabmodifiers,
    //                     );
    //                     sleep(Duration::from_millis(100));
    //                     simulate(&out_event.event_type).unwrap();
    //                     // xinput2::XIAllowEvents(dpy, devev.deviceid, mode, xlib::CurrentTime);
    //                 }
    //             }
    //         }
    //         println!("Waited for event");
    //         xlib::XFreeEventData(dpy, &mut cookie);
    //         // let r = xinput2::XIUngrabDevice(dpy, device, xlib::CurrentTime);
    //         // xinput2::XIGrabDevice(
    //         //     dpy,
    //         //     // xinput2::XIAllDevices,
    //         //     device,
    //         //     root,
    //         //     xlib::CurrentTime,
    //         //     cursor,
    //         //     mode,
    //         //     mode,
    //         //     TRUE,
    //         //     &mut eventmask,
    //         // );
    //     }
    // }
}
