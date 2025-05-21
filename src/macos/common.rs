#![allow(clippy::upper_case_acronyms)]
use crate::macos::keyboard::Keyboard;
use crate::rdev::{Button, Event, EventType};
use cocoa::base::id;
use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGEventType, EventField};
use lazy_static::lazy_static;
use std::convert::TryInto;
use std::os::raw::c_void;
use std::sync::Mutex;
use std::time::SystemTime;

use crate::macos::keycodes::key_from_code;

pub type CFMachPortRef = *const c_void;
pub type CFIndex = u64;
pub type CFAllocatorRef = id;
pub type CFRunLoopSourceRef = id;
pub type CFRunLoopRef = id;
pub type CFRunLoopMode = id;
pub type CGEventTapProxy = id;
pub type CGEventRef = CGEvent;

// https://developer.apple.com/documentation/coregraphics/cgeventtapplacement?language=objc
pub type CGEventTapPlacement = u32;
#[allow(non_upper_case_globals)]
pub const kCGHeadInsertEventTap: u32 = 0;

// https://developer.apple.com/documentation/coregraphics/cgeventtapoptions?language=objc
#[allow(non_upper_case_globals)]
#[repr(u32)]
pub enum CGEventTapOption {
    #[cfg(feature = "unstable_grab")]
    Default = 0,
    ListenOnly = 1,
}

lazy_static! {
    pub static ref LAST_FLAGS: Mutex<CGEventFlags> = Mutex::new(CGEventFlags::CGEventFlagNull);
    pub static ref KEYBOARD_STATE: Mutex<Keyboard> = Mutex::new(Keyboard::new().unwrap());
}

// https://developer.apple.com/documentation/coregraphics/cgeventmask?language=objc
pub type CGEventMask = u64;
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
unsafe extern "C" {
    #[allow(improper_ctypes)]
    pub fn CGEventTapCreate(
        tap: CGEventTapLocation,
        place: CGEventTapPlacement,
        options: CGEventTapOption,
        eventsOfInterest: CGEventMask,
        callback: QCallback,
        user_info: id,
    ) -> CFMachPortRef;
    pub fn CFMachPortCreateRunLoopSource(
        allocator: CFAllocatorRef,
        tap: CFMachPortRef,
        order: CFIndex,
    ) -> CFRunLoopSourceRef;
    pub fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFRunLoopMode);
    pub fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    pub fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    pub fn CFRunLoopRun();

    pub static kCFRunLoopCommonModes: CFRunLoopMode;

}

// TODO Remove this, this was added as the coded
// existed and worked, but clippy is complaining.
// There's probably a better fix.
#[allow(improper_ctypes_definitions)]
pub type QCallback = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    _type: CGEventType,
    cg_event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef;

pub fn set_is_main_thread(b: bool) {
    KEYBOARD_STATE.lock().unwrap().set_is_main_thread(b);
}

pub unsafe fn convert(
    _type: CGEventType,
    cg_event: &CGEvent,
    keyboard_state: &mut Keyboard,
) -> Option<Event> {
    unsafe {
        let option_type = match _type {
            CGEventType::LeftMouseDown => Some(EventType::ButtonPress(Button::Left)),
            CGEventType::LeftMouseUp => Some(EventType::ButtonRelease(Button::Left)),
            CGEventType::RightMouseDown => Some(EventType::ButtonPress(Button::Right)),
            CGEventType::RightMouseUp => Some(EventType::ButtonRelease(Button::Right)),
            CGEventType::MouseMoved => {
                let point = cg_event.location();
                Some(EventType::MouseMove {
                    x: point.x,
                    y: point.y,
                })
            }
            CGEventType::LeftMouseDragged => {
                let point = cg_event.location();
                Some(EventType::MouseMove {
                    x: point.x,
                    y: point.y,
                })
            }
            CGEventType::RightMouseDragged => {
                let point = cg_event.location();
                Some(EventType::MouseMove {
                    x: point.x,
                    y: point.y,
                })
            }
            CGEventType::KeyDown => {
                let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
                let key = key_from_code(code.try_into().ok()?);
                Some(EventType::KeyPress(key))
            }
            CGEventType::KeyUp => {
                let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
                let key = key_from_code(code.try_into().ok()?);
                Some(EventType::KeyRelease(key))
            }
            CGEventType::FlagsChanged => {
                let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
                let code = code.try_into().ok()?;
                let flags = cg_event.get_flags();
                let key = key_from_code(code);

                // Determine if this is a press or release based on flag changes
                let mut global_flags = LAST_FLAGS.lock().unwrap();
                if flags.contains(CGEventFlags::CGEventFlagShift)
                    && !global_flags.contains(CGEventFlags::CGEventFlagShift)
                {
                    *global_flags = flags;
                    Some(EventType::KeyPress(key))
                } else if !flags.contains(CGEventFlags::CGEventFlagShift)
                    && global_flags.contains(CGEventFlags::CGEventFlagShift)
                {
                    *global_flags = flags;
                    Some(EventType::KeyRelease(key))
                } else if flags.contains(CGEventFlags::CGEventFlagControl)
                    && !global_flags.contains(CGEventFlags::CGEventFlagControl)
                {
                    *global_flags = flags;
                    Some(EventType::KeyPress(key))
                } else if !flags.contains(CGEventFlags::CGEventFlagControl)
                    && global_flags.contains(CGEventFlags::CGEventFlagControl)
                {
                    *global_flags = flags;
                    Some(EventType::KeyRelease(key))
                } else if flags.contains(CGEventFlags::CGEventFlagAlternate)
                    && !global_flags.contains(CGEventFlags::CGEventFlagAlternate)
                {
                    *global_flags = flags;
                    Some(EventType::KeyPress(key))
                } else if !flags.contains(CGEventFlags::CGEventFlagAlternate)
                    && global_flags.contains(CGEventFlags::CGEventFlagAlternate)
                {
                    *global_flags = flags;
                    Some(EventType::KeyRelease(key))
                } else if flags.contains(CGEventFlags::CGEventFlagCommand)
                    && !global_flags.contains(CGEventFlags::CGEventFlagCommand)
                {
                    *global_flags = flags;
                    Some(EventType::KeyPress(key))
                } else if !flags.contains(CGEventFlags::CGEventFlagCommand)
                    && global_flags.contains(CGEventFlags::CGEventFlagCommand)
                {
                    *global_flags = flags;
                    Some(EventType::KeyRelease(key))
                } else {
                    None
                }
            }
            CGEventType::ScrollWheel => {
                let delta_y = cg_event
                    .get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_1);
                let delta_x = cg_event
                    .get_integer_value_field(EventField::SCROLL_WHEEL_EVENT_POINT_DELTA_AXIS_2);
                Some(EventType::Wheel { delta_x, delta_y })
            }
            _ => None,
        };
        if let Some(event_type) = option_type {
            let name = match event_type {
                EventType::KeyPress(_) => {
                    let code =
                        cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u32;
                    let flags = cg_event.get_flags();
                    keyboard_state.create_string_for_key(code, flags)
                }
                _ => None,
            };
            return Some(Event {
                event_type,
                time: SystemTime::now(),
                name,
            });
        }
    }
    None
}
