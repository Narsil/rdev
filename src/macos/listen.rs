use crate::macos::common::*;
use crate::macos::keyboard_state::KeyboardState;
use crate::rdev::{Button, Callback, Event, EventType, ListenError};
use cocoa::base::nil;
use cocoa::foundation::NSAutoreleasePool;
use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGEventType, EventField};
use std::convert::TryInto;
use std::os::raw::c_void;
use std::time::SystemTime;

use crate::macos::keycodes::key_from_code;

fn default_callback(event: Event) {
    println!("Default {:?}", event)
}
static mut GLOBAL_CALLBACK: Callback = default_callback;

unsafe extern "C" fn raw_callback(
    _proxy: CGEventTapProxy,
    _type: CGEventType,
    cg_event: CGEventRef,
    _user_info: *mut c_void,
) -> CGEventRef {
    // println!("Event ref {:?}", cg_event_ptr);
    // let cg_event: CGEvent = transmute_copy::<*mut c_void, CGEvent>(&cg_event_ptr);
    if let Some(event) = convert(_type, &cg_event, &mut KEYBOARD_STATE) {
        GLOBAL_CALLBACK(event);
    }
    // println!("Event ref END {:?}", cg_event_ptr);
    // cg_event_ptr
    cg_event
}

#[link(name = "Cocoa", kind = "framework")]
pub fn listen(callback: Callback) -> Result<(), ListenError> {
    unsafe {
        GLOBAL_CALLBACK = callback;
        let _pool = NSAutoreleasePool::new(nil);
        let tap = CGEventTapCreate(
            CGEventTapLocation::HID, // HID, Session, AnnotatedSession,
            kCGHeadInsertEventTap,
            CGEventTapOption::ListenOnly,
            kCGEventMaskForAllEvents,
            raw_callback,
            nil,
        );
        if tap.is_null() {
            return Err(ListenError::EventTapError);
        }
        let _loop = CFMachPortCreateRunLoopSource(nil, tap, 0);
        if _loop.is_null() {
            return Err(ListenError::LoopSourceError);
        }

        let current_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(current_loop, _loop, kCFRunLoopCommonModes);

        CGEventTapEnable(tap, true);
        CFRunLoopRun();
    }
    Ok(())
}
