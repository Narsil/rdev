extern crate libc;
extern crate x11;

use crate::rdev::{Event, EventType, SimulateError, Callback};
use std::ffi::CString;
use std::os::raw::c_int;
use std::ptr::{null, null_mut};
use std::time::SystemTime;
use x11::xlib;
use x11::xrecord;
use x11::xtest;

static mut EVENT_COUNT: u32 = 0;

fn default_callback(event: Event) {
    println!("Default : Event {:?}", event);
}

static mut GLOBAL_CALLBACK: Callback = default_callback;

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
            x: xdatum.x as f64,
            y: xdatum.y as f64,
        }),
        _ => None,
    };

    if let Some(event_type) = option_type {
        let time = SystemTime::now();
        let event = Event {
            event_type,
            time,
            name: None,
        };
        GLOBAL_CALLBACK(event);
    }

    xrecord::XRecordFreeData(raw_data);
}

static TRUE: i32 = 1;
static FALSE: i32 = 0;

unsafe fn send_native(event_type: &EventType, display: *mut xlib::Display) -> Result<(), ()> {
    match event_type {
        EventType::KeyPress { code } => {
            xtest::XTestFakeKeyEvent(display, *code as u32, TRUE, 0);
            Ok(())
        }
        EventType::KeyRelease { code } => {
            xtest::XTestFakeKeyEvent(display, *code as u32, FALSE, 0);
            Ok(())
        }
        EventType::ButtonPress { code } => {
            xtest::XTestFakeButtonEvent(display, *code as u32, TRUE, 0);
            Ok(())
        }
        EventType::ButtonRelease { code } => {
            xtest::XTestFakeButtonEvent(display, *code as u32, FALSE, 0);
            Ok(())
        }
        EventType::MouseMove { x, y } => {
            xtest::XTestFakeMotionEvent(display, 0, *x as i32, *y as i32, 0);
            // unsafe {
            //     xlib::XWarpPointer(display, 0, root, 0, 0, 0, 0, *x as i32, *y as i32);
            // }
            Ok(())
        }
        EventType::Wheel {
            delta_x: _,
            delta_y,
        } => {
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
