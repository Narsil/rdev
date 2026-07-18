#![allow(improper_ctypes_definitions)]
use crate::macos::common::*;
use crate::rdev::{Event, ListenError};
use core::ptr::NonNull;
use objc2_core_foundation::{CFMachPort, CFRunLoop, kCFRunLoopCommonModes};
#[cfg(not(feature = "macos_keyboard_only"))]
use objc2_core_graphics::kCGEventMaskForAllEvents;
use objc2_core_graphics::{
    CGEvent, CGEventTapCallBack, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventTapProxy, CGEventType,
};
use objc2_foundation::NSAutoreleasePool;
use std::ffi::c_void;
use std::ptr::null_mut;

static mut GLOBAL_CALLBACK: Option<Box<dyn FnMut(Event)>> = None;

#[cfg(feature = "macos_keyboard_only")]
const LISTEN_EVENT_MASK: u64 = (1 << CGEventType::KeyDown.0)
    | (1 << CGEventType::KeyUp.0)
    | (1 << CGEventType::FlagsChanged.0);

#[cfg(not(feature = "macos_keyboard_only"))]
const LISTEN_EVENT_MASK: u64 = kCGEventMaskForAllEvents as u64;

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
            LISTEN_EVENT_MASK,
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

#[cfg(all(test, feature = "macos_keyboard_only"))]
mod tests {
    use super::LISTEN_EVENT_MASK;
    use objc2_core_graphics::CGEventType;

    fn contains(event_type: CGEventType) -> bool {
        LISTEN_EVENT_MASK & (1 << event_type.0) != 0
    }

    #[test]
    fn keyboard_only_mask_excludes_mouse_events() {
        assert!(contains(CGEventType::KeyDown));
        assert!(contains(CGEventType::KeyUp));
        assert!(contains(CGEventType::FlagsChanged));
        assert!(!contains(CGEventType::MouseMoved));
        assert!(!contains(CGEventType::LeftMouseDown));
    }
}
