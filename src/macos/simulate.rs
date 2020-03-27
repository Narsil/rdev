use crate::rdev::{Button, EventType, SimulateError};
use core_graphics::event::{
    CGEvent, CGEventTapLocation, CGEventType, CGKeyCode, CGMouseButton, ScrollEventUnit,
};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::CGPoint;

use crate::macos::keycodes::code_from_key;

unsafe fn convert_native_with_source(
    event_type: &EventType,
    source: CGEventSource,
) -> Result<CGEvent, ()> {
    match event_type {
        EventType::KeyPress(key) => {
            if let Some(code) = code_from_key(*key) {
                return CGEvent::new_keyboard_event(source, code as CGKeyCode, true);
            }
            Err(())
        }
        EventType::KeyRelease(key) => {
            if let Some(code) = code_from_key(*key) {
                return CGEvent::new_keyboard_event(source, code as CGKeyCode, false);
            }
            Err(())
        }
        EventType::ButtonPress(button) => {
            let point = get_current_mouse_location();
            match button {
                Button::Left => {
                    CGEvent::new_mouse_event(
                        source,
                        CGEventType::LeftMouseDown,
                        point,
                        CGMouseButton::Left, // This is actually ignored because we don't use OtherMouse EventType
                    )
                }
                Button::Right => {
                    CGEvent::new_mouse_event(
                        source,
                        CGEventType::RightMouseDown,
                        point,
                        CGMouseButton::Left, // This is actually ignored because we don't use OtherMouse EventType
                    )
                }
                _ => Err(()),
            }
        }
        EventType::ButtonRelease(button) => {
            let point = get_current_mouse_location();
            match button {
                Button::Left => {
                    CGEvent::new_mouse_event(
                        source,
                        CGEventType::LeftMouseUp,
                        point,
                        CGMouseButton::Left, // This is actually ignored because we don't use OtherMouse EventType
                    )
                }
                Button::Right => {
                    CGEvent::new_mouse_event(
                        source,
                        CGEventType::RightMouseUp,
                        point,
                        CGMouseButton::Left, // This is actually ignored because we don't use OtherMouse EventType
                    )
                }
                _ => Err(()),
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
