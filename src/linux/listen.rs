extern crate libc;
extern crate x11;
use crate::linux::common::{convert, FALSE};
use crate::rdev::{Callback, Event, ListenError};
use std::convert::TryInto;
use std::ffi::CStr;
use std::os::raw::{c_int, c_uchar, c_uint};
use std::ptr::null;
use x11::xlib;
use x11::xrecord;

fn default_callback(event: Event) {
    println!("Default : Event {:?}", event);
}

static mut GLOBAL_CALLBACK: Callback = default_callback;

pub fn listen(callback: Callback) -> Result<(), ListenError> {
    unsafe {
        GLOBAL_CALLBACK = callback;
        // Open displays
        let dpy_control = xlib::XOpenDisplay(null());
        if dpy_control.is_null() {
            return Err(ListenError::MissingDisplayError);
        }
        let extension_name = CStr::from_bytes_with_nul(b"RECORD\0")
            .map_err(|_| ListenError::XRecordExtensionError)?;
        let extension = xlib::XInitExtension(dpy_control, extension_name.as_ptr());
        if extension.is_null() {
            return Err(ListenError::XRecordExtensionError);
        }

        // Prepare record range
        let mut record_range: xrecord::XRecordRange = *xrecord::XRecordAllocRange();
        record_range.device_events.first = xlib::KeyPress as c_uchar;
        record_range.device_events.last = xlib::MotionNotify as c_uchar;

        // Create context
        let context = xrecord::XRecordCreateContext(
            dpy_control,
            0,
            &mut xrecord::XRecordAllClients,
            1,
            &mut &mut record_range as *mut &mut xrecord::XRecordRange
                as *mut *mut xrecord::XRecordRange,
            1,
        );

        if context == 0 {
            return Err(ListenError::RecordContextError);
        }

        xlib::XSync(dpy_control, FALSE);
        // Run
        let result =
            xrecord::XRecordEnableContext(dpy_control, context, Some(record_callback), &mut 0);
        if result == 0 {
            return Err(ListenError::RecordContextEnablingError);
        }
    }
    Ok(())
}
// pub fn listen(callback: Callback) -> Result<(), ListenError> {
//     unsafe {
//         GLOBAL_CALLBACK = callback;
//         // Open displays
//         let dpy_control = xlib::XOpenDisplay(null());
//         if dpy_control.is_null() {
//             return Err(ListenError::MissingDisplayError);
//         }
//         let window = xlib::XDefaultRootWindow(dpy_control);
//
//         let mut event = xlib::XEvent {
//             key: xlib::XKeyEvent {
//                 display: dpy_control,
//                 root: 0,
//                 window,
//                 subwindow: 0,
//                 x: 0,
//                 y: 0,
//                 x_root: 0,
//                 y_root: 0,
//                 state: 0,
//                 keycode: 0,
//                 same_screen: 0,
//                 send_event: 0,
//                 serial: 0,
//                 type_: xlib::KeyPress,
//                 time: xlib::CurrentTime,
//             },
//         };
//         let mut mask = [0u8; 4];
//         let mut event_mask = xinput2::XIEventMask {
//             deviceid: xinput2::XIAllDevices,
//             mask_len: xinput2::XI_ButtonRelease,
//             mask: mask.as_mut_ptr(),
//         };
//         xinput2::XISetMask(&mut mask, xinput2::XI_KeyPress);
//         xinput2::XISetMask(&mut mask, xinput2::XI_KeyPress);
//         xinput2::XISetMask(&mut mask, xinput2::XI_ButtonPress);
//         xinput2::XISetMask(&mut mask, xinput2::XI_ButtonRelease);
//         xinput2::XISelectEvents(dpy_control, window, &mut event_mask, 3);
//         println!("HEre");
//         loop {
//             xlib::XNextEvent(dpy_control, &mut event);
//             println!("HEre {:?}", event);
//             if xlib::XFilterEvent(&mut event, window) != 0 {
//                 continue;
//             }
//             match event.type_ {
//                 xlib::MappingNotify => {
//                     xlib::XRefreshKeyboardMapping(&mut event.mapping);
//                 }
//
//                 xlib::KeyPress => {
//                     println!("pressed KEY: {:?}", event);
//                 }
//                 xlib::KeyRelease => {
//                     println!("released KEY: {:?}", event);
//                 }
//                 _ => (),
//             }
//         }
//     }
// }

// No idea how to do that properly relevant doc lives here:
// https://www.x.org/releases/X11R7.7/doc/libXtst/recordlib.html#Datum_Flags
// https://docs.rs/xproto/1.1.5/xproto/struct._xEvent__bindgen_ty_1.html
#[repr(C)]
struct XRecordDatum {
    type_: u8,
    code: u8,
    _rest: u64,
    _1: bool,
    _2: bool,
    _3: bool,
    root_x: i16,
    root_y: i16,
    event_x: i16,
    event_y: i16,
    state: u16,
}

unsafe extern "C" fn record_callback(_null: *mut i8, raw_data: *mut xrecord::XRecordInterceptData) {
    let data = raw_data.as_ref().unwrap();
    if data.category != xrecord::XRecordFromServer {
        return;
    }

    debug_assert!(data.data_len * 4 >= std::mem::size_of::<XRecordDatum>().try_into().unwrap());
    // Cast binary data
    #[allow(clippy::cast_ptr_alignment)]
    let xdatum = (data.data as *const XRecordDatum).as_ref().unwrap();

    let code: c_uint = xdatum.code.into();
    let type_: c_int = xdatum.type_.into();

    let x = xdatum.root_x as f64;
    let y = xdatum.root_y as f64;

    if let Some(event) = convert(code, type_, x, y) {
        GLOBAL_CALLBACK(event);
    }
    xrecord::XRecordFreeData(raw_data);
}
