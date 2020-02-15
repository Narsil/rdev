use crate::rdev::{Event, EventType, SimulateError};
use cocoa::base::{id, nil};
use cocoa::foundation::NSAutoreleasePool;
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTapLocation, CGEventType, CGKeyCode, CGMouseButton, EventField,
    ScrollEventUnit,
};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::CGPoint;
use std::os::raw::c_void;
use std::time::SystemTime;

type CFMachPortRef = *const c_void;
type CFIndex = u64;
type CFAllocatorRef = id;
type CFRunLoopSourceRef = id;
type CFRunLoopRef = id;
type CFRunLoopMode = id;
type CGEventTapProxy = id;

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
    fn CGEventTapCreate(
        tap: CGEventTapLocation,
        place: CGEventTapPlacement,
        options: CGEventTapOptions,
        eventsOfInterest: CGEventMask,
        callback: QCallback,
        user_info: id,
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

unsafe fn convert(_type: CGEventType, cg_event: &CGEvent) -> Option<Event> {
    let option_type = match _type {
        CGEventType::LeftMouseDown => Some(EventType::ButtonPress { code: 1 }),
        CGEventType::LeftMouseUp => Some(EventType::ButtonRelease { code: 1 }),
        CGEventType::RightMouseDown => Some(EventType::ButtonPress { code: 3 }),
        CGEventType::RightMouseUp => Some(EventType::ButtonRelease { code: 3 }),
        CGEventType::MouseMoved => {
            let point = cg_event.location();
            Some(EventType::MouseMove {
                x: point.x,
                y: point.y,
            })
        }
        CGEventType::KeyDown => {
            let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u8;
            Some(EventType::KeyPress { code })
        }
        CGEventType::KeyUp => {
            let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u8;
            Some(EventType::KeyRelease { code })
        }
        CGEventType::FlagsChanged => {
            let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u8;
            let flags = cg_event.get_flags();
            if flags < LAST_FLAGS {
                LAST_FLAGS = flags;
                Some(EventType::KeyRelease { code })
            } else {
                LAST_FLAGS = flags;
                Some(EventType::KeyPress { code })
            }
        }
        CGEventType::ScrollWheel => {
            let delta_y =
                cg_event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_1);
            let delta_x =
                cg_event.get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_2);
            Some(EventType::Wheel { delta_x, delta_y })
        }
        _ => None,
    };
    if let Some(event_type) = option_type {
        return Some(Event {
            event_type,
            time: SystemTime::now(),
            name: None,
        });
    }
    None
}

unsafe extern "C" fn raw_callback(
    _proxy: CGEventTapProxy,
    _type: CGEventType,
    cg_event: CGEvent,
    _user_info: *mut c_void,
) -> CGEvent {
    if let Some(event) = convert(_type, &cg_event) {
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
            CGEventTapLocation::HID,
            kCGHeadInsertEventTap,
            kCGEventTapOptionDefault,
            kCGEventMaskForAllEvents,
            raw_callback,
            nil,
        );
        if tap.is_null() {
            panic!("We failed to create Event tap !");
        }
        let _loop = CFMachPortCreateRunLoopSource(nil, tap, 0);
        if _loop.is_null() {
            panic!("We failed to create loop source!");
        }

        let current_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(current_loop, _loop, kCFRunLoopCommonModes);

        CGEventTapEnable(tap, true);
        CFRunLoopRun();
    }
}

unsafe fn convert_native_with_source(
    event_type: &EventType,
    source: CGEventSource,
) -> Result<CGEvent, ()> {
    match event_type {
        EventType::KeyPress { code } => {
            CGEvent::new_keyboard_event(source, *code as CGKeyCode, true)
        }
        EventType::KeyRelease { code } => {
            CGEvent::new_keyboard_event(source, *code as CGKeyCode, false)
        }
        EventType::ButtonPress { code } => {
            // TODO
            let point = get_current_mouse_location();
            if *code == 1 {
                CGEvent::new_mouse_event(
                    source,
                    CGEventType::LeftMouseDown,
                    point,
                    CGMouseButton::Left, // This is actually ignored because we don't use OtherMouse EventType
                )
            } else if *code == 3 {
                CGEvent::new_mouse_event(
                    source,
                    CGEventType::RightMouseDown,
                    point,
                    CGMouseButton::Left, // This is actually ignored because we don't use OtherMouse EventType
                )
            } else {
                Err(())
            }
        }
        EventType::ButtonRelease { code } => {
            let point = get_current_mouse_location();
            if *code == 1 {
                CGEvent::new_mouse_event(
                    source,
                    CGEventType::LeftMouseUp,
                    point,
                    CGMouseButton::Left, // This is actually ignored because we don't use OtherMouse EventType
                )
            } else if *code == 3 {
                CGEvent::new_mouse_event(
                    source,
                    CGEventType::RightMouseUp,
                    point,
                    CGMouseButton::Left, // This is actually ignored because we don't use OtherMouse EventType
                )
            } else {
                Err(())
            }
        }
        EventType::MouseMove { x, y } => {
            let point = CGPoint { x: *x, y: *y };
            CGEvent::new_mouse_event(source, CGEventType::MouseMoved, point, CGMouseButton::Left)
        }
        EventType::Wheel { delta_x, delta_y } => {
            let wheel_count = 2;
            CGEvent::new_scroll_event(
                source,
                ScrollEventUnit::PIXEL,
                wheel_count,
                *delta_y as i32,
                *delta_x as i32,
                0,
            )
        }
    }
}

unsafe fn convert_native(event_type: &EventType) -> Result<CGEvent, ()> {
    match CGEventSource::new(CGEventSourceStateID::HIDSystemState) {
        Ok(source) => convert_native_with_source(event_type, source),
        Err(_) => Err(()),
    }
}

unsafe fn get_current_mouse_location() -> CGPoint {
    match CGEventSource::new(CGEventSourceStateID::HIDSystemState) {
        Ok(source) => match CGEvent::new(source) {
            Ok(event) => event.location(),
            Err(_) => CGPoint { x: -1.0, y: -1.0 },
        },
        Err(_) => CGPoint { x: -1.0, y: -1.0 },
    }
}

#[link(name = "Cocoa", kind = "framework")]
pub fn simulate(event_type: &EventType) -> Result<(), SimulateError> {
    unsafe {
        if let Ok(cg_event) = convert_native(event_type) {
            cg_event.post(CGEventTapLocation::HID);
            Ok(())
        } else {
            Err(SimulateError)
        }
    }
}
