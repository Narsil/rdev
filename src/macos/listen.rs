#![allow(improper_ctypes_definitions)]
use crate::macos::common::*;
use crate::rdev::{Event, ListenError};
use core::ptr::NonNull;
use objc2_core_foundation::{CFMachPort, CFRunLoop, kCFRunLoopCommonModes};
use objc2_core_graphics::{
    CGEvent, CGEventTapCallBack, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventTapProxy, CGEventType, kCGEventMaskForAllEvents,
};
use objc2_foundation::NSAutoreleasePool;
use std::ffi::c_void;
use std::ptr::null_mut;

static mut GLOBAL_CALLBACK: Option<Box<dyn FnMut(Event)>> = None;

#[link(name = "Cocoa", kind = "framework")]
unsafe extern "C" {}

unsafe extern "C-unwind" fn raw_callback(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    cg_event: NonNull<CGEvent>,
    _user_info: *mut c_void,
) -> *mut CGEvent {
    let opt = KEYBOARD_STATE.lock();
    if let Ok(mut keyboard) = opt {
        unsafe {
            if let Some(event) = convert(event_type, cg_event, &mut keyboard) {
                // Reborrowing the global callback pointer.
                // Totally UB. but not sure there's a great alternative.
                let ptr = &raw mut GLOBAL_CALLBACK;
                if let Some(callback) = &mut *ptr {
                    callback(event);
                }
            }
        }
    }
    cg_event.as_ptr()
}

pub fn listen<T>(callback: T) -> Result<(), ListenError>
where
    T: FnMut(Event) + 'static,
{
    unsafe {
        GLOBAL_CALLBACK = Some(Box::new(callback));
        let _pool = NSAutoreleasePool::new();
        let callback: CGEventTapCallBack = Some(raw_callback);
        let tap = CGEvent::tap_create(
            CGEventTapLocation::HIDEventTap, // HID, Session, AnnotatedSession,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            kCGEventMaskForAllEvents.into(),
            callback,
            null_mut(),
        )
        .ok_or(ListenError::EventTapError)?;
        let loop_ = CFMachPort::new_run_loop_source(None, Some(&tap), 0)
            .ok_or(ListenError::LoopSourceError)?;

        let current_loop = CFRunLoop::current().unwrap();
        current_loop.add_source(Some(&loop_), kCFRunLoopCommonModes);

        CGEvent::tap_enable(&tap, true);
        CFRunLoop::run();
    }
    Ok(())
}
