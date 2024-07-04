use crate::rdev::{Button, EventType, SimulateError};
use core_graphics::event::{
    CGEvent, CGEventTapLocation, CGEventType, CGMouseButton, ScrollEventUnit,
};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use core_graphics::geometry::CGPoint;
use std::convert::TryInto;

use crate::macos::keycodes::code_from_key;

unsafe fn convert_native_with_source(
    event_type: &EventType,
    source: CGEventSource,
) -> Option<CGEvent> {
    match event_type {
        EventType::KeyPress(key) => {
            let code = code_from_key(*key)?;
            CGEvent::new_keyboard_event(source, code, true).ok()
        }
        EventType::KeyRelease(key) => {
            let code = code_from_key(*key)?;
            CGEvent::new_keyboard_event(source, code, false).ok()
        }
        EventType::ButtonPress { button, x, y } => {
            let point = CGPoint { x: *x, y: *y };
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
        EventType::ButtonRelease { button, x, y } => {
            let point = CGPoint { x: *x, y: *y };
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
            let point = CGPoint { x: *x, y: *y };
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
        EventType::Drag { button: _, x, y } => {
            //https://developer.apple.com/documentation/coregraphics/quartz_event_services?language=objc
            //no drag event in quartz_event_services of coregraphics
            //simulate drag event into mousemove event
            let event_type = EventType::MouseMove { x: *x, y: *y };
            convert_native_with_source(&event_type, source)
        }
    }
}

unsafe fn convert_native(event_type: &EventType) -> Option<CGEvent> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).ok()?;
    convert_native_with_source(event_type, source)
}

///cause all of button events contain coordinate which can be used when they be simulated.
///so you don't need this fn when button press/release anymore.
#[allow(dead_code)]
unsafe fn get_current_mouse_location() -> Option<CGPoint> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).ok()?;
    let event = CGEvent::new(source).ok()?;
    Some(event.location())
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
