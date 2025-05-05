use crate::linux::wayland::keycodes::ukey_from_key;
use crate::rdev::{Button, EventType};
use crate::SimulateError;
use input_linux::{
    EventKind, EventTime, InputEvent, InputId, Key as UKey, KeyEvent, KeyState, RelativeAxis,
    RelativeEvent, SynchronizeEvent, SynchronizeKind, UInputHandle,
};
use libc::{input_event, O_NONBLOCK};
use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::sync::{LazyLock, Mutex};

static HANDLE: LazyLock<Mutex<Option<UInputHandle<File>>>> = LazyLock::new(|| Mutex::new(None));

pub struct Handle;

impl Handle {
    pub fn new() -> Self {
        let mut handle = HANDLE.lock().unwrap();
        if handle.is_none() {
            let file = OpenOptions::new()
                .write(true)
                .custom_flags(O_NONBLOCK)
                .open("/dev/uinput")
                .unwrap();
            let uinput = UInputHandle::new(file);
            uinput.set_evbit(EventKind::Key).unwrap();
            uinput.set_evbit(EventKind::Relative).unwrap();
            uinput.set_relbit(RelativeAxis::X).unwrap();
            uinput.set_relbit(RelativeAxis::Y).unwrap();
            uinput.set_relbit(RelativeAxis::Wheel).unwrap();

            // Enable all keys
            for key in UKey::iter() {
                uinput.set_keybit(key).unwrap();
            }

            let input_id = InputId {
                bustype: input_linux::sys::BUS_VIRTUAL,
                vendor: 0x1234,
                product: 0x5678,
                version: 1,
            };
            let device_name = b"rdev virtual input";
            uinput.create(&input_id, device_name, 0, &[]).unwrap();
            *handle = Some(uinput);
        }
        Handle
    }

    fn send_key_event(
        &self,
        handle: &UInputHandle<File>,
        ukey: UKey,
        state: KeyState,
    ) -> Result<(), SimulateError> {
        const ZERO: EventTime = EventTime::new(0, 0);
        let event = KeyEvent::new(ZERO, ukey, state);
        let event: input_event = InputEvent::from(event).into();
        let sync = SynchronizeEvent::new(ZERO, SynchronizeKind::Report, 0);
        let sync: input_event = InputEvent::from(sync).into();

        handle.write(&[event, sync]).map_err(|_| SimulateError)?;
        Ok(())
    }

    pub fn send(&self, event: &EventType) -> Result<(), SimulateError> {
        let handle = HANDLE.lock().unwrap();
        if let Some(handle) = handle.as_ref() {
            const ZERO: EventTime = EventTime::new(0, 0);
            match event {
                EventType::KeyPress(key) => {
                    if let Some(ukey) = ukey_from_key(*key) {
                        self.send_key_event(handle, ukey, KeyState::PRESSED)?;
                    }
                }
                EventType::KeyRelease(key) => {
                    if let Some(ukey) = ukey_from_key(*key) {
                        self.send_key_event(handle, ukey, KeyState::RELEASED)?;
                    }
                }
                EventType::ButtonPress(button) => {
                    let ukey = match button {
                        Button::Left => UKey::ButtonLeft,
                        Button::Right => UKey::ButtonRight,
                        Button::Middle => UKey::ButtonMiddle,
                        Button::Unknown(_) => return Err(SimulateError),
                    };
                    let event = KeyEvent::new(ZERO, ukey, KeyState::PRESSED);
                    let event: input_event = InputEvent::from(event).into();
                    let sync = SynchronizeEvent::new(ZERO, SynchronizeKind::Report, 0);
                    let sync: input_event = InputEvent::from(sync).into();
                    handle.write(&[event, sync]).map_err(|_| SimulateError)?;
                }
                EventType::ButtonRelease(button) => {
                    let ukey = match button {
                        Button::Left => UKey::ButtonLeft,
                        Button::Right => UKey::ButtonRight,
                        Button::Middle => UKey::ButtonMiddle,
                        Button::Unknown(_) => return Err(SimulateError),
                    };
                    let event = KeyEvent::new(ZERO, ukey, KeyState::RELEASED);
                    let event: input_event = InputEvent::from(event).into();
                    let sync = SynchronizeEvent::new(ZERO, SynchronizeKind::Report, 0);
                    let sync: input_event = InputEvent::from(sync).into();
                    handle.write(&[event, sync]).map_err(|_| SimulateError)?;
                }
                EventType::MouseMove { x, y } => {
                    let event_x = RelativeEvent::new(ZERO, RelativeAxis::X, *x as i32);
                    let event_x: input_event = InputEvent::from(event_x).into();
                    let event_y = RelativeEvent::new(ZERO, RelativeAxis::Y, *y as i32);
                    let event_y: input_event = InputEvent::from(event_y).into();
                    let sync = SynchronizeEvent::new(ZERO, SynchronizeKind::Report, 0);
                    let sync: input_event = InputEvent::from(sync).into();
                    handle
                        .write(&[event_x, event_y, sync])
                        .map_err(|_| SimulateError)?;
                }
                EventType::Wheel { delta_x, delta_y } => {
                    let event_x = RelativeEvent::new(ZERO, RelativeAxis::Wheel, *delta_x as i32);
                    let event_x: input_event = InputEvent::from(event_x).into();
                    let event_y = RelativeEvent::new(ZERO, RelativeAxis::Wheel, *delta_y as i32);
                    let event_y: input_event = InputEvent::from(event_y).into();
                    let sync = SynchronizeEvent::new(ZERO, SynchronizeKind::Report, 0);
                    let sync: input_event = InputEvent::from(sync).into();
                    handle
                        .write(&[event_x, event_y, sync])
                        .map_err(|_| SimulateError)?;
                }
            }
        }
        Ok(())
    }
}

pub fn simulate(event: &EventType) -> Result<(), SimulateError> {
    let handle = Handle::new();
    handle.send(event)?;
    Ok(())
}
