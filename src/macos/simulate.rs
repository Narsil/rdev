use crate::rdev::{Button, EventType, SimulateError};
use crate::Key;
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTapLocation, CGEventType, CGMouseButton, EventField,
    ScrollEventUnit,
};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::CGPoint;
use std::convert::TryInto;

use crate::macos::common::LAST_FLAGS;
use crate::macos::keycodes::code_from_key;

// Track the current state of modifier keys
static mut CURRENT_FLAGS: CGEventFlags = CGEventFlags::CGEventFlagNull;

unsafe fn convert_native_with_source(
    event_type: &EventType,
    source: CGEventSource,
) -> Option<CGEvent> {
    match event_type {
        EventType::KeyPress(key) => {
            let code = code_from_key(*key)?;
            // For modifier keys, we need to use FlagsChanged event type
            if is_modifier_key(*key) {
                let event = CGEvent::new(source).ok()?;
                event.set_type(CGEventType::FlagsChanged);
                event.set_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE, code as i64);

                // Get current flags and update them
                let mut new_flags = LAST_FLAGS;
                match key {
                    Key::ShiftLeft | Key::ShiftRight => {
                        new_flags.insert(CGEventFlags::CGEventFlagShift);
                    }
                    Key::ControlLeft | Key::ControlRight => {
                        new_flags.insert(CGEventFlags::CGEventFlagControl);
                    }
                    Key::AltGr | Key::Alt => {
                        new_flags.insert(CGEventFlags::CGEventFlagAlternate);
                    }
                    Key::MetaLeft | Key::MetaRight => {
                        new_flags.insert(CGEventFlags::CGEventFlagCommand);
                    }
                    _ => {}
                }
                event.set_flags(new_flags);
                Some(event)
            } else {
                // For non-modifier keys, use regular key events
                let event = CGEvent::new_keyboard_event(source, code, true).ok()?;
                event.set_flags(LAST_FLAGS);
                Some(event)
            }
        }
        EventType::KeyRelease(key) => {
            let code = code_from_key(*key)?;
            // For modifier keys, we need to use FlagsChanged event type
            if is_modifier_key(*key) {
                let event = CGEvent::new(source).ok()?;
                event.set_type(CGEventType::FlagsChanged);
                event.set_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE, code as i64);

                // Get current flags and update them
                let mut new_flags = LAST_FLAGS;
                match key {
                    Key::ShiftLeft | Key::ShiftRight => {
                        new_flags.remove(CGEventFlags::CGEventFlagShift);
                    }
                    Key::ControlLeft | Key::ControlRight => {
                        new_flags.remove(CGEventFlags::CGEventFlagControl);
                    }
                    Key::AltGr | Key::Alt => {
                        new_flags.remove(CGEventFlags::CGEventFlagAlternate);
                    }
                    Key::MetaLeft | Key::MetaRight => {
                        new_flags.remove(CGEventFlags::CGEventFlagCommand);
                    }
                    _ => {}
                }
                event.set_flags(new_flags);
                Some(event)
            } else {
                // For non-modifier keys, use regular key events
                let event = CGEvent::new_keyboard_event(source, code, false).ok()?;
                event.set_flags(LAST_FLAGS);
                Some(event)
            }
        }
        EventType::ButtonPress(button) => {
            let point = get_current_mouse_location()?;
            let event = match button {
                Button::Left => CGEventType::LeftMouseDown,
                Button::Right => CGEventType::RightMouseDown,
                _ => return None,
            };
            CGEvent::new_mouse_event(
                source,
                event,
                point,
                CGMouseButton::Left, // ignored because we don't use OtherMouse EventType
            )
            .ok()
        }
        EventType::ButtonRelease(button) => {
            let point = get_current_mouse_location()?;
            let event = match button {
                Button::Left => CGEventType::LeftMouseUp,
                Button::Right => CGEventType::RightMouseUp,
                _ => return None,
            };
            CGEvent::new_mouse_event(
                source,
                event,
                point,
                CGMouseButton::Left, // ignored because we don't use OtherMouse EventType
            )
            .ok()
        }
        EventType::MouseMove { x, y } => {
            let point = CGPoint { x: (*x), y: (*y) };
            CGEvent::new_mouse_event(source, CGEventType::MouseMoved, point, CGMouseButton::Left)
                .ok()
        }
        EventType::Wheel { delta_x, delta_y } => {
            let wheel_count = 2;
            CGEvent::new_scroll_event(
                source,
                ScrollEventUnit::PIXEL,
                wheel_count,
                (*delta_y).try_into().ok()?,
                (*delta_x).try_into().ok()?,
                0,
            )
            .ok()
        }
    }
}

unsafe fn convert_native(event_type: &EventType) -> Option<CGEvent> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).ok()?;
    convert_native_with_source(event_type, source)
}

unsafe fn get_current_mouse_location() -> Option<CGPoint> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).ok()?;
    let event = CGEvent::new(source).ok()?;
    Some(event.location())
}

fn is_modifier_key(key: Key) -> bool {
    matches!(
        key,
        Key::ShiftLeft
            | Key::ShiftRight
            | Key::ControlLeft
            | Key::ControlRight
            | Key::Alt
            | Key::AltGr
            | Key::MetaLeft
            | Key::MetaRight
    )
}

#[link(name = "Cocoa", kind = "framework")]
extern "C" {}

pub fn simulate(event_type: &EventType) -> Result<(), SimulateError> {
    unsafe {
        if let Some(cg_event) = convert_native(event_type) {
            cg_event.post(CGEventTapLocation::HID);
            Ok(())
        } else {
            Err(SimulateError)
        }
    }
}
