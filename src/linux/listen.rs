extern crate libc;
extern crate x11;
extern crate xproto;
use crate::linux::keyboard_state::KeyboardState;
use crate::linux::keycodes::key_from_code;
use crate::rdev::{Button, Callback, Event, EventType, ListenError};
use std::ffi::CString;
use std::os::raw::{c_int, c_uchar};
use std::ptr::null;
use std::time::SystemTime;
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
        let dpy_data = xlib::XOpenDisplay(null());
        if dpy_control.is_null() || dpy_data.is_null() {
            return Err(ListenError::MissingDisplayError);
        }
        // Enable synchronization
        xlib::XSynchronize(dpy_control, 1);

        let extension_name = CString::new("RECORD").unwrap();

        let extension = xlib::XInitExtension(dpy_control, extension_name.as_ptr());
        if extension.is_null() {
            return Err(ListenError::XRecordExtensionError);
        }

        // Get version
        let mut version_major: c_int = 0;
        let mut version_minor: c_int = 0;
        xrecord::XRecordQueryVersion(dpy_control, &mut version_major, &mut version_minor);

        // Prepare record range
        let mut record_range: xrecord::XRecordRange = *xrecord::XRecordAllocRange();
        record_range.device_events.first = xlib::KeyPress as u8;
        record_range.device_events.last = xlib::MotionNotify as u8;

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
        // Run
        let result =
            xrecord::XRecordEnableContext(dpy_data, context, Some(record_callback), &mut 0);
        if result == 0 {
            return Err(ListenError::RecordContextEnablingError);
        }
    }
    Ok(())
}

// No idea how to do that properly relevant doc lives here:
// https://www.x.org/releases/X11R7.7/doc/libXtst/recordlib.html#Datum_Flags
#[repr(C)]
union XRecordDatum {
    type_: c_uchar,
    event: xproto::xEvent,
    // error: xError,
    // req: xResourceReq,
    // replay: xGenericReply,
    // setup: xConnSetupPrefix,
}

// #[repr(C)]
// struct XRecordDatum {
//     xtype: u8,
//     code: u8,
//     a: u16,
//     b: u32,
//     c: u32,
//     d: u32,
//     e: u32,
//     x: u16,
//     y: u16,
//     h: u32,
// }
//

unsafe extern "C" fn record_callback(_: *mut i8, raw_data: *mut xrecord::XRecordInterceptData) {
    let data = &*raw_data;

    // Skip server events
    if data.category != xrecord::XRecordFromServer {
        return;
    }

    // Cast binary data
    #[allow(clippy::cast_ptr_alignment)]
    let xdatum = &mut *(data.data as *mut XRecordDatum);

    // println!("xdatum.type_ {:?}", xdatum.type_);
    // println!("xdatum.event {:?}", xdatum.event);

    let code = xdatum.event.u.u.as_ref().detail;
    let option_type = match xdatum.type_ as i32 {
        xlib::KeyPress => {
            let key = key_from_code(code as u32);
            Some(EventType::KeyPress(key))
        }
        xlib::KeyRelease => {
            let key = key_from_code(code as u32);
            Some(EventType::KeyRelease(key))
        }
        // Xlib does not implement wheel events left and right afaik.
        // But MacOS does, so we need to acknowledge the larger event space.
        xlib::ButtonPress => {
            if code == 4 {
                Some(EventType::Wheel {
                    delta_y: -1,
                    delta_x: 0,
                })
            } else if code == 5 {
                Some(EventType::Wheel {
                    delta_y: 1,
                    delta_x: 0,
                })
            } else {
                match code {
                    1 => Some(EventType::ButtonPress(Button::Left)),
                    2 => Some(EventType::ButtonPress(Button::Middle)),
                    3 => Some(EventType::ButtonPress(Button::Right)),
                    code => Some(EventType::ButtonPress(Button::Unknown(code as u8))),
                }
            }
        }
        xlib::ButtonRelease => {
            if code == 4 || code == 5 {
                None
            } else {
                match code {
                    1 => Some(EventType::ButtonRelease(Button::Left)),
                    2 => Some(EventType::ButtonRelease(Button::Middle)),
                    3 => Some(EventType::ButtonRelease(Button::Right)),
                    _ => Some(EventType::ButtonRelease(Button::Unknown(code as u8))),
                }
            }
        }
        xlib::MotionNotify => Some(EventType::MouseMove {
            x: xdatum.event.u.keyButtonPointer.as_ref().rootX as f64,
            y: xdatum.event.u.keyButtonPointer.as_ref().rootY as f64,
        }),
        _ => None,
    };

    if let Some(event_type) = option_type {
        let name = match event_type {
            EventType::KeyPress(_) => {
                if let Ok(mut keyboard) = KeyboardState::new() {
                    keyboard.name_from_code(&mut xdatum.event)
                } else {
                    None
                }
            }
            _ => None,
        };
        let time = SystemTime::now();
        let event = Event {
            event_type,
            time,
            name,
        };
        GLOBAL_CALLBACK(event);
    }

    xrecord::XRecordFreeData(raw_data);
}
