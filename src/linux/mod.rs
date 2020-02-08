extern crate libc;
extern crate x11;

use crate::rdev::{Event, EventType};
use std::ffi::CString;
use std::os::raw::c_int;
use std::ptr::{null, null_mut};
use std::time::SystemTime;
use x11::xlib;
use x11::xrecord;

static mut EVENT_COUNT: u32 = 0;

type Callback = fn(event: Event);

fn default_callback(event: Event) {
    println!("Default : Event {:?}", event);
}

static mut GLOBAL_CALLBACK: Callback = default_callback;

// pub struct EventIterator {
//     display: *mut xlib::Display,
// }
//
// impl Iterator for EventIterator {
//     type Item = Event;
//
//     fn next(&mut self) -> Option<Event> {
//         let code = 1;
//         let event_type = EventType::KeyPress { code };
//         let time = SystemTime::now();
//         Some(Event::new(event_type, time, None).unwrap())
//     }
// }

pub fn listen(callback: Callback) {
    unsafe {
        GLOBAL_CALLBACK = callback;
        // Open displays
        let dpy_control = xlib::XOpenDisplay(null());
        let dpy_data = xlib::XOpenDisplay(null());
        if dpy_control == null_mut() || dpy_data == null_mut() {
            panic!("can't open display");
        }
        // Enable synchronization
        xlib::XSynchronize(dpy_control, 1);

        let extension_name = CString::new("RECORD").unwrap();

        let extension = xlib::XInitExtension(dpy_control, extension_name.as_ptr());
        if extension.is_null() {
            panic!("Error init X Record Extension");
        }

        // Get version
        let mut version_major: c_int = 0;
        let mut version_minor: c_int = 0;
        xrecord::XRecordQueryVersion(dpy_control, &mut version_major, &mut version_minor);
        println!(
            "RECORD extension version {}.{}",
            version_major, version_minor
        );

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
            std::mem::transmute(&mut &mut record_range),
            1,
        );

        if context == 0 {
            panic!("Fail create Record context\n");
        }
        // Run
        let result =
            xrecord::XRecordEnableContext(dpy_data, context, Some(record_callback), &mut 0);
        if result == 0 {
            panic!("Cound not enable the Record context!\n");
        }
    }
}

// No idea how to do that properly relevant doc lives here:
// https://www.x.org/releases/X11R7.7/doc/libXtst/recordlib.html#Datum_Flags
#[repr(C)]
struct XRecordDatum {
    xtype: u8,
    code: u8,
    a: u16,
    b: u32,
    c: u32,
    d: u32,
    e: u32,
    x: u16,
    y: u16,
    h: u32,
}

unsafe extern "C" fn record_callback(_: *mut i8, raw_data: *mut xrecord::XRecordInterceptData) {
    EVENT_COUNT += 1;
    let data = &*raw_data;

    // Skip server events
    if data.category != xrecord::XRecordFromServer {
        return;
    }

    // Cast binary data
    let xdatum = &*(data.data as *mut XRecordDatum);

    let option_type = match xdatum.xtype as i32 {
        xlib::KeyPress => Some(EventType::KeyPress { code: xdatum.code }),
        xlib::KeyRelease => Some(EventType::KeyRelease { code: xdatum.code }),
        // Xlib does not implement wheel events left and right afaik.
        // But MacOS does, so we need to acknowledge the larger event space.
        xlib::ButtonPress => {
            if xdatum.code == 4 {
                Some(EventType::Wheel {
                    delta_y: -1,
                    delta_x: 0,
                })
            } else if xdatum.code == 5 {
                Some(EventType::Wheel {
                    delta_y: 1,
                    delta_x: 0,
                })
            } else {
                Some(EventType::ButtonPress { code: xdatum.code })
            }
        }
        xlib::ButtonRelease => {
            if xdatum.code == 4 {
                None
            } else if xdatum.code == 5 {
                None
            } else {
                Some(EventType::ButtonRelease { code: xdatum.code })
            }
        }
        xlib::MotionNotify => Some(EventType::MouseMove {
            x: xdatum.x as u64,
            y: xdatum.y as u64,
        }),
        _ => None,
    };

    if let Some(event_type) = option_type {
        let time = SystemTime::now();
        let event = Event::new(event_type, time, None).unwrap();
        GLOBAL_CALLBACK(event);
    }

    xrecord::XRecordFreeData(raw_data);
}
