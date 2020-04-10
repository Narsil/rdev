use crate::macos::common::*;
use crate::rdev::{Event, GrabCallback, GrabError};
use cocoa::base::nil;
use cocoa::foundation::NSAutoreleasePool;
use core_graphics::event::{CGEventTapLocation, CGEventType};
use std::os::raw::c_void;

fn default_callback(event: Event) -> Option<Event> {
    println!("Default {:?}", event);
    Some(event)
}
static mut GLOBAL_CALLBACK: GrabCallback = default_callback;

unsafe extern "C" fn raw_callback(
    _proxy: CGEventTapProxy,
    _type: CGEventType,
    cg_event: CGEventRef,
    _user_info: *mut c_void,
) -> CGEventRef {
    // println!("Event ref {:?}", cg_event_ptr);
    // let cg_event: CGEvent = transmute_copy::<*mut c_void, CGEvent>(&cg_event_ptr);
    let opt = KEYBOARD_STATE.lock();
    if let Ok(mut keyboard) = opt {
        if let Some(event) = convert(_type, &cg_event, &mut keyboard) {
            if GLOBAL_CALLBACK(event).is_none() {
                cg_event.set_type(CGEventType::Null);
            }
        }
    }
    cg_event
}

#[link(name = "Cocoa", kind = "framework")]
pub fn grab(callback: GrabCallback) -> Result<(), GrabError> {
    unsafe {
        GLOBAL_CALLBACK = callback;
        let _pool = NSAutoreleasePool::new(nil);
        let tap = CGEventTapCreate(
            CGEventTapLocation::HID, // HID, Session, AnnotatedSession,
            kCGHeadInsertEventTap,
            CGEventTapOption::Default,
            kCGEventMaskForAllEvents,
            raw_callback,
            nil,
        );
        if tap.is_null() {
            return Err(GrabError::EventTapError);
        }
        let _loop = CFMachPortCreateRunLoopSource(nil, tap, 0);
        if _loop.is_null() {
            return Err(GrabError::LoopSourceError);
        }

        let current_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(current_loop, _loop, kCFRunLoopCommonModes);

        CGEventTapEnable(tap, true);
        CFRunLoopRun();
    }
    Ok(())
}
