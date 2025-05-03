use crate::linux::wayland::keycodes::code_from_key;
use crate::rdev::{EventType, SimulateError};
use crate::Key;
use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::sync::{LazyLock, Mutex};

use input_linux::{
    EventKind, EventTime, InputEvent, InputId, Key as IKey, KeyEvent, KeyState, SynchronizeEvent,
    SynchronizeKind, UInputHandle,
};
use libc::O_NONBLOCK;

struct Handle {
    handle: UInputHandle<File>,
}

impl Handle {
    fn new() -> Result<Self, SimulateError> {
        let uinput_file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(O_NONBLOCK)
            .open("/dev/uinput")
            .or_else(|_| Err(SimulateError))?;
        let uhandle = UInputHandle::new(uinput_file);

        uhandle
            .set_evbit(EventKind::Key)
            .map_err(|_| SimulateError)?;
        for key in input_linux::Key::iter() {
            uhandle.set_keybit(key).map_err(|_| SimulateError)?;
        }

        let input_id = InputId {
            bustype: input_linux::sys::BUS_USB,
            vendor: 0x1234,
            product: 0x5678,
            version: 0,
        };
        let device_name = b"Example device";
        uhandle
            .create(&input_id, device_name, 0, &[])
            .map_err(|_| SimulateError)?;
        Ok(Self { handle: uhandle })
    }

    fn send(&self, event: &EventType) -> Result<(), SimulateError> {
        match event {
            EventType::KeyPress(k) => self.simulate(*k, KeyState::PRESSED)?,
            EventType::KeyRelease(k) => self.simulate(*k, KeyState::RELEASED)?,
            _ => {
                // eprintln!("Unhandled simulation event"),
            }
        }
        Ok(())
    }

    fn simulate(&self, k: Key, state: KeyState) -> Result<(), SimulateError> {
        const ZERO: EventTime = EventTime::new(0, 0);
        let code = code_from_key(k).ok_or(SimulateError)?;
        let key = IKey::from_code(code as u16 - 16).map_err(|_| SimulateError)?;
        // println!("Key {k:?} IKey {key:?} - Code {code}");
        let events = [
            InputEvent::from(KeyEvent::new(ZERO, key, state)).into_raw(),
            // InputEvent::from(KeyEvent::new(ZERO, key, KeyState::RELEASED)).into_raw(),
            InputEvent::from(SynchronizeEvent::new(ZERO, SynchronizeKind::Report, 0)).into_raw(),
        ];
        self.handle.write(&events).map_err(|_| SimulateError)?;
        Ok(())
    }
}

// thread::sleep(Duration::from_secs(5));

// const ZERO: EventTime = EventTime::new(0, 0);
// let key = Key::from_code(48)?;
// let events = [
//     InputEvent::from(KeyEvent::new(ZERO, key, KeyState::PRESSED)).into_raw(),
//     // InputEvent::from(SynchronizeEvent::new(ZERO, SynchronizeKind::Report, 0)).into_raw(),
//     InputEvent::from(KeyEvent::new(ZERO, key, KeyState::RELEASED)).into_raw(),
//     InputEvent::from(SynchronizeEvent::new(ZERO, SynchronizeKind::Report, 0)).into_raw(),
// ];
// uhandle.write(&events)?;
// thread::sleep(Duration::from_secs(1));

// // This call to sleep was not necessary on my machine,
// // but this translation is meant to match exactly
// uhandle.dev_destroy()?;
static HANDLE: LazyLock<Mutex<Handle>> = LazyLock::new(|| Mutex::new(Handle::new().unwrap()));

pub fn simulate(event_type: &EventType) -> Result<(), SimulateError> {
    let handle = HANDLE.lock().unwrap();
    handle.send(event_type)?;
    Ok(())
}
