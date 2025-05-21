use crate::SimulateError;
use crate::linux::wayland::keycodes::ukey_from_key;
use crate::rdev::{Button, EventType};
use input_linux::{
    EventKind, EventTime, InputEvent, InputId, Key as UKey, KeyEvent, KeyState, RelativeAxis,
    RelativeEvent, SynchronizeEvent, SynchronizeKind, UInputHandle,
};
use libc::{O_NONBLOCK, input_event};
use std::fs::{File, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::sync::{LazyLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

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
            uinput.set_relbit(RelativeAxis::WheelHiRes).unwrap();

            // Enable all keys
            for key in UKey::iter() {
                uinput.set_keybit(key).unwrap();
            }
            // Enable all keys
            for rel in RelativeAxis::iter() {
                uinput.set_relbit(rel).unwrap();
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

    fn get_current_time() -> EventTime {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        EventTime::new(now.as_secs() as i64, now.subsec_micros() as i64)
    }

    fn send_key_event(
        &self,
        handle: &UInputHandle<File>,
        ukey: UKey,
        state: KeyState,
    ) -> Result<(), SimulateError> {
        let time = Self::get_current_time();
        let event = KeyEvent::new(time, ukey, state);
        let event: input_event = InputEvent::from(event).into();
        let sync = SynchronizeEvent::new(time, SynchronizeKind::Report, 0);
        let sync: input_event = InputEvent::from(sync).into();

        handle.write(&[event, sync]).map_err(|_| SimulateError)?;
        Ok(())
    }

    pub fn send(&self, event: &EventType) -> Result<(), SimulateError> {
        let handle = HANDLE.lock().unwrap();
        if let Some(handle) = handle.as_ref() {
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
                    let time = Self::get_current_time();
                    let event = KeyEvent::new(time, ukey, KeyState::PRESSED);
                    let event: input_event = InputEvent::from(event).into();
                    let sync = SynchronizeEvent::new(time, SynchronizeKind::Report, 0);
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
                    let time = Self::get_current_time();
                    let event = KeyEvent::new(time, ukey, KeyState::RELEASED);
                    let event: input_event = InputEvent::from(event).into();
                    let sync = SynchronizeEvent::new(time, SynchronizeKind::Report, 0);
                    let sync: input_event = InputEvent::from(sync).into();
                    handle.write(&[event, sync]).map_err(|_| SimulateError)?;
                }
                EventType::MouseMove { x, y } => {
                    let time = Self::get_current_time();
                    // let reset_x = RelativeEvent::new(time, RelativeAxis::X, i32::MIN);
                    // let reset_x: input_event = InputEvent::from(reset_x).into();
                    // let reset_y = RelativeEvent::new(time, RelativeAxis::Y, i32::MIN);
                    // let reset_y: input_event = InputEvent::from(reset_y).into();
                    // let rsync = SynchronizeEvent::new(time, SynchronizeKind::Report, 0);
                    // let rsync: input_event = InputEvent::from(rsync).into();
                    // handle
                    //     .write(&[reset_x, reset_y, rsync])
                    //     .map_err(|_| SimulateError)?;

                    let event_x = RelativeEvent::new(time, RelativeAxis::X, *x as i32);
                    let event_x: input_event = InputEvent::from(event_x).into();
                    let event_y = RelativeEvent::new(time, RelativeAxis::Y, *y as i32);
                    let event_y: input_event = InputEvent::from(event_y).into();
                    let sync = SynchronizeEvent::new(time, SynchronizeKind::Report, 0);
                    let sync: input_event = InputEvent::from(sync).into();
                    handle
                        .write(&[event_x, event_y, sync])
                        .map_err(|_| SimulateError)?;
                }
                EventType::Wheel { delta_x, delta_y } => {
                    let time = Self::get_current_time();
                    let event_x =
                        RelativeEvent::new(time, RelativeAxis::WheelHiRes, (*delta_x * 120) as i32);
                    let event_x: input_event = InputEvent::from(event_x).into();
                    let event_y =
                        RelativeEvent::new(time, RelativeAxis::WheelHiRes, (*delta_y * 120) as i32);
                    let event_y: input_event = InputEvent::from(event_y).into();
                    let sync = SynchronizeEvent::new(time, SynchronizeKind::Report, 0);
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
