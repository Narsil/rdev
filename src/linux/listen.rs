extern crate libc;
extern crate x11;
extern crate xproto;
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

unsafe extern "C" fn record_callback(_null: *mut i8, raw_data: *mut xrecord::XRecordInterceptData) {
    let ptr = raw_data.as_ref();
    if let Some(data) = ptr {
        // Skip server events
        if data.category != xrecord::XRecordFromServer {
            return;
        }

        debug_assert!(data.data_len * 4 >= std::mem::size_of::<XRecordDatum>().try_into().unwrap());
        // Cast binary data
        #[allow(clippy::cast_ptr_alignment)]
        let xdatum_ptr = (data.data as *const XRecordDatum).as_ref();

        if let Some(xdatum) = xdatum_ptr {
            let code: c_uint = xdatum.event.u.u.as_ref().detail.into();
            let type_: c_int = xdatum.type_.into();
            let keypointer = xdatum.event.u.keyButtonPointer.as_ref();
            let x = keypointer.rootX as f64;
            let y = keypointer.rootY as f64;
            let state = keypointer.state as c_uint;

            if let Some(event) = convert(code, state, type_, x, y) {
                GLOBAL_CALLBACK(event);
            }
        }
    }
    xrecord::XRecordFreeData(raw_data);
}
