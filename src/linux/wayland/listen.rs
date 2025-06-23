extern crate libc;
use super::keyboard::Keyboard;
use super::keycodes::key_from_code;
use crate::rdev::{Event, KeyboardState, ListenError};
use crate::{Button, EventType};
use input::event::PointerEvent;
use input::event::keyboard::{KeyState, KeyboardEventTrait};
use input::event::pointer::{Axis, ButtonState};
use input::{Event as LibEvent, Libinput, LibinputInterface};
use libc::{O_RDONLY, O_RDWR, O_WRONLY};
use std::fs::{File, OpenOptions};
use std::os::unix::{fs::OpenOptionsExt, io::OwnedFd};
use std::path::Path;
use std::time::{Duration, SystemTime};

struct Interface;

fn convert_type(libevent: LibEvent) -> Option<EventType> {
    match libevent {
        LibEvent::Keyboard(key) => {
            let k = key_from_code(key.key());
            let state: KeyState = key.key_state();
            match state {
                KeyState::Pressed => Some(EventType::KeyPress(k)),
                KeyState::Released => Some(EventType::KeyRelease(k)),
            }
        }
        LibEvent::Pointer(PointerEvent::Button(btn)) => {
            let rdev_btn = match btn.button() {
                272 => Some(Button::Left),
                273 => Some(Button::Right),
                274 => Some(Button::Middle),
                _ => None,
            };
            if let Some(rdev_btn) = rdev_btn {
                let state: ButtonState = btn.button_state();
                match state {
                    ButtonState::Pressed => Some(EventType::ButtonPress(rdev_btn)),
                    ButtonState::Released => Some(EventType::ButtonRelease(rdev_btn)),
                }
            } else {
                None
            }
        }
        LibEvent::Pointer(PointerEvent::Motion(btn)) => Some(EventType::MouseMove {
            // TODO Convert to absolute X, Y
            x: btn.dx_unaccelerated(),
            y: btn.dy_unaccelerated(),
        }),
        LibEvent::Pointer(PointerEvent::MotionAbsolute(btn)) => Some(EventType::MouseMove {
            x: btn.absolute_x(),
            y: btn.absolute_y(),
        }),
        LibEvent::Pointer(PointerEvent::ScrollWheel(btn)) => Some(EventType::Wheel {
            delta_x: -(btn.scroll_value_v120(Axis::Horizontal) / 120.0) as i64,
            delta_y: -(btn.scroll_value_v120(Axis::Vertical) / 120.0) as i64,
        }),
        _ => {
            // dbg!(format!("Received unhandlded event {lib:?}"));
            None
        }
    }
}
fn convert(keyboard: &mut Keyboard, libevent: LibEvent) -> Option<Event> {
    let event_type = convert_type(libevent)?;
    let name = keyboard.add(&event_type);
    Some(Event {
        time: SystemTime::now(),
        name,
        event_type,
    })
}

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
        #[allow(clippy::bad_bit_mask)]
        OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_RDONLY != 0) | (flags & O_RDWR != 0))
            .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
            .open(path)
            .map(|file| file.into())
            .map_err(|err| err.raw_os_error().unwrap())
    }
    fn close_restricted(&mut self, fd: OwnedFd) {
        drop(File::from(fd));
    }
}

pub fn listen<T>(mut callback: T) -> Result<(), ListenError>
where
    T: FnMut(Event) + 'static,
{
    let mut input = Libinput::new_with_udev(Interface);
    input.udev_assign_seat("seat0").unwrap();
    let mut keyboard = Keyboard::new().map_err(|_| ListenError::KeyboardError)?;
    loop {
        input.dispatch().unwrap();
        for libevent in &mut input {
            if let Some(event) = convert(&mut keyboard, libevent) {
                callback(event);
            }
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}
