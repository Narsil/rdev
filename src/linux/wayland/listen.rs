extern crate libc;
use super::keycodes::key_from_code;
use crate::rdev::{Event, ListenError};
use crate::EventType;
use input::event::keyboard::{KeyState, KeyboardEventTrait};
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
        _ => None,
    }
}
fn convert(libevent: LibEvent) -> Option<Event> {
    let event_type = convert_type(libevent)?;
    Some(Event {
        time: SystemTime::now(),
        name: None,
        event_type,
    })
}

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
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
    loop {
        input.dispatch().unwrap();
        for libevent in &mut input {
            if let Some(event) = convert(libevent) {
                callback(event);
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}
