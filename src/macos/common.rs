#![allow(clippy::upper_case_acronyms)]
use crate::macos::keyboard::Keyboard;
use crate::rdev::{Button, Event, EventType};
use core::ptr::NonNull;
use lazy_static::lazy_static;
use objc2_core_graphics::{CGEvent, CGEventField, CGEventFlags, CGEventType};
use std::convert::TryInto;
use std::sync::Mutex;
use std::time::SystemTime;

use crate::macos::keycodes::key_from_code_or_name;

use super::keycodes::key_from_special_key;

lazy_static! {
    pub static ref KEYBOARD_STATE: Mutex<Keyboard> = Mutex::new(Keyboard::new().unwrap());
}

const CONTROL_LEFT: u16 = 59;
const CONTROL_RIGHT: u16 = 62;
const SHIFT_LEFT: u16 = 56;
const SHIFT_RIGHT: u16 = 60;
const ALT_LEFT: u16 = 58;
const ALT_RIGHT: u16 = 61;
const META_LEFT: u16 = 55;
const META_RIGHT: u16 = 54;
const CAPS_LOCK: u16 = 57;
const FUNCTION: u16 = 63;

// Device-dependent modifier masks from IOKit/hidsystem/ev_keymap.h. Unlike
// CGEventFlags' aggregate masks, these identify the physical left/right key.
const NX_DEVICELCTLKEYMASK: u64 = 0x0000_0001;
const NX_DEVICELSHIFTKEYMASK: u64 = 0x0000_0002;
const NX_DEVICERSHIFTKEYMASK: u64 = 0x0000_0004;
const NX_DEVICELCMDKEYMASK: u64 = 0x0000_0008;
const NX_DEVICERCMDKEYMASK: u64 = 0x0000_0010;
const NX_DEVICELALTKEYMASK: u64 = 0x0000_0020;
const NX_DEVICERALTKEYMASK: u64 = 0x0000_0040;
const NX_DEVICERCTLKEYMASK: u64 = 0x0000_2000;

pub(super) fn modifier_mask_for_keycode(code: u16) -> Option<CGEventFlags> {
    let mask = match code {
        CONTROL_LEFT => NX_DEVICELCTLKEYMASK,
        CONTROL_RIGHT => NX_DEVICERCTLKEYMASK,
        SHIFT_LEFT => NX_DEVICELSHIFTKEYMASK,
        SHIFT_RIGHT => NX_DEVICERSHIFTKEYMASK,
        ALT_LEFT => NX_DEVICELALTKEYMASK,
        ALT_RIGHT => NX_DEVICERALTKEYMASK,
        META_LEFT => NX_DEVICELCMDKEYMASK,
        META_RIGHT => NX_DEVICERCMDKEYMASK,
        CAPS_LOCK => CGEventFlags::MaskAlphaShift.bits(),
        FUNCTION => CGEventFlags::MaskSecondaryFn.bits(),
        _ => return None,
    };
    Some(CGEventFlags::from_bits_retain(mask))
}

fn modifier_event_type(code: u16, flags: CGEventFlags) -> Option<EventType> {
    let mask = modifier_mask_for_keycode(code)?;
    let key = key_from_code_or_name(code, None);
    if flags.contains(mask) {
        Some(EventType::KeyPress(key))
    } else {
        Some(EventType::KeyRelease(key))
    }
}

pub fn set_is_main_thread(b: bool) {
    KEYBOARD_STATE.lock().unwrap().set_is_main_thread(b);
}

pub unsafe fn convert(
    _type: CGEventType,
    cg_event: NonNull<CGEvent>,
    keyboard_state: &mut Keyboard,
) -> Option<Event> {
    unsafe {
        let option_type = match _type {
            CGEventType::LeftMouseDown => Some(EventType::ButtonPress(Button::Left)),
            CGEventType::LeftMouseUp => Some(EventType::ButtonRelease(Button::Left)),
            CGEventType::RightMouseDown => Some(EventType::ButtonPress(Button::Right)),
            CGEventType::RightMouseUp => Some(EventType::ButtonRelease(Button::Right)),
            CGEventType::MouseMoved => {
                let point = CGEvent::location(Some(cg_event.as_ref()));
                // let point = cg_event.location();
                Some(EventType::MouseMove {
                    x: point.x,
                    y: point.y,
                })
            }
            CGEventType::LeftMouseDragged => {
                let point = CGEvent::location(Some(cg_event.as_ref()));
                Some(EventType::MouseMove {
                    x: point.x,
                    y: point.y,
                })
            }
            CGEventType::RightMouseDragged => {
                let point = CGEvent::location(Some(cg_event.as_ref()));
                Some(EventType::MouseMove {
                    x: point.x,
                    y: point.y,
                })
            }
            CGEventType::KeyDown => {
                // println!("Received {cg_event:?}");
                let code = CGEvent::integer_value_field(
                    Some(cg_event.as_ref()),
                    CGEventField::KeyboardEventKeycode,
                );
                let flags = CGEvent::flags(Some(cg_event.as_ref()));
                let name = keyboard_state.create_string_for_key(code, flags);
                let key = key_from_code_or_name(code.try_into().ok()?, name.as_deref());
                Some(EventType::KeyPress(key))
            }
            CGEventType::KeyUp => {
                let code = CGEvent::integer_value_field(
                    Some(cg_event.as_ref()),
                    CGEventField::KeyboardEventKeycode,
                );
                let flags = CGEvent::flags(Some(cg_event.as_ref()));
                let name = keyboard_state.create_string_for_key(code, flags);
                let key = key_from_code_or_name(code.try_into().ok()?, name.as_deref());
                Some(EventType::KeyRelease(key))
            }
            CGEventType::FlagsChanged => {
                let code = CGEvent::integer_value_field(
                    Some(cg_event.as_ref()),
                    CGEventField::KeyboardEventKeycode,
                );
                let code = code.try_into().ok()?;
                let flags = CGEvent::flags(Some(cg_event.as_ref()));
                modifier_event_type(code, flags)
            }
            CGEventType::ScrollWheel => {
                let delta_y = CGEvent::integer_value_field(
                    Some(cg_event.as_ref()),
                    CGEventField::ScrollWheelEventDeltaAxis1,
                );
                let delta_x = CGEvent::integer_value_field(
                    Some(cg_event.as_ref()),
                    CGEventField::ScrollWheelEventDeltaAxis2,
                );
                Some(EventType::Wheel { delta_x, delta_y })
            }
            CGEventType(14) => {
                // Core graphics doesnt support NX_SYSDEFINED yet

                let subtype =
                    CGEvent::integer_value_field(Some(cg_event.as_ref()), CGEventField(99));
                let data1 =
                    CGEvent::integer_value_field(Some(cg_event.as_ref()), CGEventField(149));
                let key_flags = data1 & 0x0000ffff;
                let key_pressed = ((key_flags & 0xff00) >> 8) == 0xa;
                let _key_repeat = (key_flags & 0x1) == 0x1;
                let key_code = (data1 & 0xffff0000) >> 16;

                // Mouse buttons like middle click/back/forward are subtype 7
                // Subtype 8 means keyboard event
                if subtype != 8 {
                    return None;
                }

                // println!("Received {key_code:?}");
                if let Some(code) = key_from_special_key(key_code.try_into().ok()?) {
                    if key_pressed {
                        Some(EventType::KeyPress(code))
                    } else {
                        Some(EventType::KeyRelease(code))
                    }
                } else {
                    // If we don't handle the key avoid creating an event since it can create duplicates with other keys
                    None
                }
            }
            _ev => {
                // println!("Received {_ev:?}");
                None
            }
        };
        if let Some(event_type) = option_type {
            let name = match event_type {
                EventType::KeyPress(_) => {
                    let code = CGEvent::integer_value_field(
                        Some(cg_event.as_ref()),
                        CGEventField::KeyboardEventKeycode,
                    );
                    let flags = CGEvent::flags(Some(cg_event.as_ref()));
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

#[cfg(test)]
mod tests {
    use super::{
        NX_DEVICELSHIFTKEYMASK, NX_DEVICERSHIFTKEYMASK, SHIFT_LEFT, SHIFT_RIGHT,
        modifier_event_type,
    };
    use crate::{EventType, Key};
    use objc2_core_graphics::CGEventFlags;

    fn flags(bits: u64) -> CGEventFlags {
        CGEventFlags::from_bits_retain(bits)
    }

    #[test]
    fn classifies_modifier_from_keycode_without_cached_state() {
        let left_down = flags(CGEventFlags::MaskShift.bits() | NX_DEVICELSHIFTKEYMASK);
        assert!(matches!(
            modifier_event_type(SHIFT_LEFT, left_down),
            Some(EventType::KeyPress(Key::ShiftLeft))
        ));

        let right_down = flags(CGEventFlags::MaskShift.bits() | NX_DEVICERSHIFTKEYMASK);
        assert!(matches!(
            modifier_event_type(SHIFT_LEFT, right_down),
            Some(EventType::KeyRelease(Key::ShiftLeft))
        ));
        assert!(matches!(
            modifier_event_type(SHIFT_RIGHT, right_down),
            Some(EventType::KeyPress(Key::ShiftRight))
        ));
    }

    #[test]
    fn ignores_flags_changed_for_unknown_keycode() {
        assert!(modifier_event_type(0, CGEventFlags::empty()).is_none());
    }
}
