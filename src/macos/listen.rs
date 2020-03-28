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
static mut LAST_FLAGS: CGEventFlags = CGEventFlags::CGEventFlagNull;
static mut KEYBOARD_STATE: KeyboardState = KeyboardState { dead_state: 0 };

unsafe fn convert(
    _type: CGEventType,
    cg_event: &CGEvent,
    keyboard_state: &mut KeyboardState,
) -> Option<Event> {
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
        CGEventType::KeyDown => {
            let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
            Some(EventType::KeyPress(key_from_code(code.try_into().ok()?)))
        }
        CGEventType::KeyUp => {
            let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
            Some(EventType::KeyRelease(key_from_code(code.try_into().ok()?)))
        }
        CGEventType::FlagsChanged => {
            let code = cg_event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE);
            let code = code.try_into().ok()?;
            let flags = cg_event.get_flags();
            if flags < LAST_FLAGS {
                LAST_FLAGS = flags;
                Some(EventType::KeyRelease(key_from_code(code)))
            } else {
                LAST_FLAGS = flags;
                Some(EventType::KeyPress(key_from_code(code)))
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
    None
}

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
