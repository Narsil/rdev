use objc2_core_foundation::{CFRetained, CGPoint};
use objc2_core_graphics::{
    CGEvent, CGEventField, CGEventFlags, CGEventSource, CGEventSourceStateID, CGEventTapLocation,
    CGEventType, CGMouseButton, CGScrollEventUnit,
};

use crate::Key;
use crate::rdev::{Button, EventType, SimulateError};
use std::convert::TryInto;

use crate::macos::common::LAST_FLAGS;
use crate::macos::keycodes::code_from_key;

unsafe fn convert_native_with_source(
    event_type: &EventType,
    source: CFRetained<CGEventSource>,
) -> Option<CFRetained<CGEvent>> {
    unsafe {
        match event_type {
            EventType::KeyPress(key) => {
                let code = code_from_key(*key)?;
                // For modifier keys, we need to use FlagsChanged event type
                if is_modifier_key(*key) {
                    let event = CGEvent::new(Some(&source))?;
                    CGEvent::set_type(Some(&event), CGEventType::FlagsChanged);
                    CGEvent::set_integer_value_field(
                        Some(&event),
                        CGEventField::KeyboardEventKeycode,
                        code as i64,
                    );

                    // Get current flags and update them
                    let mut new_flags = LAST_FLAGS.lock().unwrap();
                    match key {
                        Key::ShiftLeft | Key::ShiftRight => {
                            new_flags.insert(CGEventFlags::MaskShift);
                        }
                        Key::ControlLeft | Key::ControlRight => {
                            new_flags.insert(CGEventFlags::MaskControl);
                        }
                        Key::AltGr | Key::Alt => {
                            new_flags.insert(CGEventFlags::MaskAlternate);
                        }
                        Key::MetaLeft | Key::MetaRight => {
                            new_flags.insert(CGEventFlags::MaskCommand);
                        }
                        _ => {}
                    }
                    CGEvent::set_flags(Some(&event), *new_flags);
                    // event.set_flags(*new_flags);
                    Some(event)
                } else {
                    // For non-modifier keys, use regular key events
                    let event = CGEvent::new_keyboard_event(Some(&source), code, true)?;
                    CGEvent::set_flags(Some(&event), *LAST_FLAGS.lock().unwrap());
                    Some(event)
                }
            }
            EventType::KeyRelease(key) => {
                let code = code_from_key(*key)?;
                // For modifier keys, we need to use FlagsChanged event type
                if is_modifier_key(*key) {
                    let event = CGEvent::new(Some(&source))?;
                    CGEvent::set_type(Some(&event), CGEventType::FlagsChanged);
                    CGEvent::set_integer_value_field(
                        Some(&event),
                        CGEventField::KeyboardEventKeycode,
                        code as i64,
                    );

                    // Get current flags and update them
                    let mut new_flags = LAST_FLAGS.lock().unwrap();
                    match key {
                        Key::ShiftLeft | Key::ShiftRight => {
                            new_flags.remove(CGEventFlags::MaskShift);
                        }
                        Key::ControlLeft | Key::ControlRight => {
                            new_flags.remove(CGEventFlags::MaskControl);
                        }
                        Key::AltGr | Key::Alt => {
                            new_flags.remove(CGEventFlags::MaskAlternate);
                        }
                        Key::MetaLeft | Key::MetaRight => {
                            new_flags.remove(CGEventFlags::MaskCommand);
                        }
                        _ => {}
                    }
                    CGEvent::set_flags(Some(&event), *new_flags);
                    Some(event)
                } else {
                    // For non-modifier keys, use regular key events
                    let event = CGEvent::new_keyboard_event(Some(&source), code, false)?;
                    CGEvent::set_flags(Some(&event), *LAST_FLAGS.lock().unwrap());
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
                    Some(&source),
                    event,
                    point,
                    CGMouseButton::Left, // ignored because we don't use OtherMouse EventType
                )
            }
            EventType::ButtonRelease(button) => {
                let point = get_current_mouse_location()?;
                let event = match button {
                    Button::Left => CGEventType::LeftMouseUp,
                    Button::Right => CGEventType::RightMouseUp,
                    _ => return None,
                };
                CGEvent::new_mouse_event(
                    Some(&source),
                    event,
                    point,
                    CGMouseButton::Left, // ignored because we don't use OtherMouse EventType
                )
            }
            EventType::MouseMove { x, y } => {
                let point = CGPoint { x: (*x), y: (*y) };
                CGEvent::new_mouse_event(
                    Some(&source),
                    CGEventType::MouseMoved,
                    point,
                    CGMouseButton::Left,
                )
            }
            EventType::Wheel { delta_x, delta_y } => {
                let wheel_count = 2;
                CGEvent::new_scroll_wheel_event2(
                    Some(&source),
                    CGScrollEventUnit::Pixel,
                    wheel_count,
                    (*delta_y).try_into().ok()?,
                    (*delta_x).try_into().ok()?,
                    0,
                )
            }
        }
    }
}

unsafe fn convert_native(event_type: &EventType) -> Option<CFRetained<CGEvent>> {
    unsafe {
        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)?;
        convert_native_with_source(event_type, source)
    }
}

unsafe fn get_current_mouse_location() -> Option<CGPoint> {
    unsafe {
        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)?;
        let event = CGEvent::new(Some(&source))?;
        Some(CGEvent::location(Some(&event)))
    }
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
unsafe extern "C" {}

pub fn simulate(event_type: &EventType) -> Result<(), SimulateError> {
    unsafe {
        if let Some(cg_event) = convert_native(event_type) {
            CGEvent::post(CGEventTapLocation::HIDEventTap, Some(&cg_event));
            Ok(())
        } else {
            Err(SimulateError)
        }
    }
}
