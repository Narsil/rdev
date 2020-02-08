use crate::rdev::{Event, EventType};
use cocoa::base::nil;
use cocoa::foundation::NSAutoreleasePool;
use core_graphics::event::{CGEventField, CGEventFlags, CGEventType, EventField};
use std::ffi::c_void;
use std::ptr::null_mut;
use std::time::SystemTime;

type CFMachPortRef = *const c_void;
type CFIndex = u64;
type CFAllocatorRef = *const c_void;
type CFRunLoopSourceRef = *const c_void;
type CFRunLoopRef = *const c_void;
type CFRunLoopMode = *const c_void;
type CGEventTapProxy = *const c_void;
type CGEvent = *const c_void;

// https://developer.apple.com/documentation/coregraphics/cgeventtaplocation?language=objc
type CGEventTapLocation = u32;
#[allow(non_upper_case_globals)]
pub const kCGHIDEventTap: u32 = 0;

// https://developer.apple.com/documentation/coregraphics/cgeventtapplacement?language=objc
type CGEventTapPlacement = u32;
#[allow(non_upper_case_globals)]
pub const kCGHeadInsertEventTap: u32 = 0;

// https://developer.apple.com/documentation/coregraphics/cgeventtapoptions?language=objc
type CGEventTapOptions = u32;
#[allow(non_upper_case_globals)]
pub const kCGEventTapOptionDefault: u32 = 0;

// https://developer.apple.com/documentation/coregraphics/cgeventmask?language=objc
type CGEventMask = u64;
#[allow(non_upper_case_globals)]
pub const kCGEventMaskForAllEvents: u64 = (1 << CGEventType::LeftMouseDown as u64)
    + (1 << CGEventType::LeftMouseUp as u64)
    + (1 << CGEventType::RightMouseDown as u64)
    + (1 << CGEventType::RightMouseUp as u64)
    + (1 << CGEventType::MouseMoved as u64)
    + (1 << CGEventType::LeftMouseDragged as u64)
    + (1 << CGEventType::RightMouseDragged as u64)
    + (1 << CGEventType::KeyDown as u64)
    + (1 << CGEventType::KeyUp as u64)
    + (1 << CGEventType::FlagsChanged as u64)
    + (1 << CGEventType::ScrollWheel as u64);

#[cfg(target_os = "macos")]
#[link(name = "Cocoa", kind = "framework")]
extern "C" {
    fn CGEventGetFlags(cg_event: CGEvent) -> CGEventFlags;
    fn CGEventTapCreate(
        tap: CGEventTapLocation,
        place: CGEventTapPlacement,
        options: CGEventTapOptions,
        eventsOfInterest: CGEventMask,
        callback: QCallback,
        user_info: *mut c_void,
    ) -> CFMachPortRef;
    fn CFMachPortCreateRunLoopSource(
        allocator: CFAllocatorRef,
        tap: CFMachPortRef,
        order: CFIndex,
    ) -> CFRunLoopSourceRef;
    fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFRunLoopMode);
    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    fn CFRunLoopRun();
    fn CGEventGetIntegerValueField(event: CGEvent, field: CGEventField) -> i64;

    pub static kCFRunLoopCommonModes: CFRunLoopMode;

}
type QCallback = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    _type: CGEventType,
    cg_event: CGEvent,
    user_info: *mut c_void,
) -> CGEvent;
type Callback = fn(event: Event);

fn default_callback(event: Event) {
    println!("Default {:?}", event)
}
static mut GLOBAL_CALLBACK: Callback = default_callback;
static mut LAST_FLAGS: CGEventFlags = CGEventFlags::CGEventFlagNull;

unsafe fn convert(_type: CGEventType, cg_event: CGEvent) -> Option<Event> {
    let option_type = match _type {
        CGEventType::LeftMouseDown => Some(EventType::ButtonPress { code: 1 }),
        CGEventType::LeftMouseUp => Some(EventType::ButtonRelease { code: 1 }),
        CGEventType::RightMouseDown => Some(EventType::ButtonPress { code: 3 }),
        CGEventType::RightMouseUp => Some(EventType::ButtonRelease { code: 3 }),
        // TODO
        CGEventType::MouseMoved => Some(EventType::MouseMove { x: 0, y: 0 }),
        CGEventType::KeyDown => {
            let code =
                CGEventGetIntegerValueField(cg_event, EventField::KEYBOARD_EVENT_KEYCODE) as u8;
            Some(EventType::KeyPress { code })
        }
        CGEventType::KeyUp => {
            let code =
                CGEventGetIntegerValueField(cg_event, EventField::KEYBOARD_EVENT_KEYCODE) as u8;
            Some(EventType::KeyRelease { code })
        }
        CGEventType::FlagsChanged => {
            let code =
                CGEventGetIntegerValueField(cg_event, EventField::KEYBOARD_EVENT_KEYCODE) as u8;
            let flags = CGEventGetFlags(cg_event);
            if flags < LAST_FLAGS {
                LAST_FLAGS = flags;
                Some(EventType::KeyRelease { code })
            } else {
                LAST_FLAGS = flags;
                Some(EventType::KeyPress { code })
            }
        }
        CGEventType::ScrollWheel => {
            let delta_y = CGEventGetIntegerValueField(
                cg_event,
                EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_1,
            );
            let delta_x = CGEventGetIntegerValueField(
                cg_event,
                EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_2,
            );
            Some(EventType::Wheel { delta_x, delta_y })
        }
        _ => None,
    };
    if let Some(event_type) = option_type {
        if let Ok(event) = Event::new(event_type, SystemTime::now(), None) {
            return Some(event);
        }
    }
    None
}

unsafe extern "C" fn raw_callback(
    _proxy: CGEventTapProxy,
    _type: CGEventType,
    cg_event: CGEvent,
    _user_info: *mut c_void,
) -> CGEvent {
    if let Some(event) = convert(_type, cg_event) {
        GLOBAL_CALLBACK(event);
    }
    cg_event
}

#[link(name = "Cocoa", kind = "framework")]
pub fn listen(callback: Callback) {
    unsafe {
        GLOBAL_CALLBACK = callback;
        let _pool = NSAutoreleasePool::new(nil);
        let tap = CGEventTapCreate(
            kCGHIDEventTap,
            kCGHeadInsertEventTap,
            kCGEventTapOptionDefault,
            kCGEventMaskForAllEvents,
            raw_callback,
            null_mut(),
        );
        if tap.is_null() {
            panic!("We failed to create Event tap !");
        }
        let _loop = CFMachPortCreateRunLoopSource(null_mut(), tap, 0);
        if _loop.is_null() {
            panic!("We failed to create loop source!");
        }

        let current_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(current_loop, _loop, kCFRunLoopCommonModes);

        CGEventTapEnable(tap, true);
        CFRunLoopRun();
    }
}
