#![allow(improper_ctypes_definitions)]
use crate::macos::common::*;
use crate::rdev::{Event, ListenError};
use core::ptr::NonNull;
use objc2_core_foundation::{CFMachPort, CFRetained, CFRunLoop, kCFRunLoopCommonModes};
use objc2_core_graphics::{
    CGEvent, CGEventTapCallBack, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventTapProxy, CGEventType, kCGEventMaskForAllEvents,
};
use objc2_foundation::NSAutoreleasePool;
use std::ffi::c_void;

struct EventTapContext {
    callback: Box<dyn FnMut(Event)>,
    tap: Option<CFRetained<CFMachPort>>,
}

fn is_tap_disabled_event(event_type: CGEventType) -> bool {
    matches!(
        event_type,
        CGEventType::TapDisabledByTimeout | CGEventType::TapDisabledByUserInput
    )
}

#[link(name = "Cocoa", kind = "framework")]
unsafe extern "C" {}

unsafe extern "C-unwind" fn raw_callback(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    cg_event: NonNull<CGEvent>,
    user_info: *mut c_void,
) -> *mut CGEvent {
    let context = unsafe { &mut *user_info.cast::<EventTapContext>() };
    if is_tap_disabled_event(event_type) {
        if let Some(tap) = context.tap.as_deref() {
            CGEvent::tap_enable(tap, true);
            eprintln!("rdev: macOS event tap was disabled ({event_type:?}); re-enabled");
        }
        return cg_event.as_ptr();
    }

    let opt = KEYBOARD_STATE.lock();
    if let Ok(mut keyboard) = opt {
        unsafe {
            if let Some(event) = convert(event_type, cg_event, &mut keyboard) {
                (context.callback)(event);
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
        let _pool = NSAutoreleasePool::new();
        let raw_callback: CGEventTapCallBack = Some(raw_callback);
        let mut context = Box::new(EventTapContext {
            callback: Box::new(callback),
            tap: None,
        });
        let tap = CGEvent::tap_create(
            CGEventTapLocation::HIDEventTap, // HID, Session, AnnotatedSession,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            kCGEventMaskForAllEvents.into(),
            raw_callback,
            (&mut *context as *mut EventTapContext).cast(),
        )
        .ok_or(ListenError::EventTapError)?;
        context.tap = Some(tap.clone());
        let loop_ = CFMachPort::new_run_loop_source(None, Some(&tap), 0)
            .ok_or(ListenError::LoopSourceError)?;

        let current_loop = CFRunLoop::current().unwrap();
        current_loop.add_source(Some(&loop_), kCFRunLoopCommonModes);

        CGEvent::tap_enable(&tap, true);
        CFRunLoop::run();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::is_tap_disabled_event;
    use objc2_core_graphics::CGEventType;

    #[test]
    fn identifies_both_event_tap_disable_reasons() {
        assert!(is_tap_disabled_event(CGEventType::TapDisabledByTimeout));
        assert!(is_tap_disabled_event(CGEventType::TapDisabledByUserInput));
        assert!(!is_tap_disabled_event(CGEventType::FlagsChanged));
    }
}
