use crate::rdev::{Event, EventType, ListenError};
use crate::windows::common::{convert, set_key_hook, set_mouse_hook, HookError, HOOK, KEYBOARD};
use std::os::raw::c_int;
use std::ptr::null_mut;
use std::sync::mpsc;
use std::time::SystemTime;
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::um::winuser::{CallNextHookEx, PeekMessageA, HC_ACTION};

static mut GLOBAL_CALLBACK: Option<Box<dyn FnMut(Event)>> = None;

impl From<HookError> for ListenError {
    fn from(error: HookError) -> Self {
        match error {
            HookError::Mouse(code) => ListenError::MouseHookError(code),
            HookError::Key(code) => ListenError::KeyHookError(code),
        }
    }
}

unsafe extern "system" fn raw_callback(code: c_int, param: WPARAM, lpdata: LPARAM) -> LRESULT {
    if code == HC_ACTION {
        let opt = convert(param, lpdata);
        if let Some(event_type) = opt {
            let name = match &event_type {
                EventType::KeyPress(_key) => match (*KEYBOARD).lock() {
                    Ok(mut keyboard) => keyboard.get_name(lpdata),
                    Err(_) => None,
                },
                _ => None,
            };
            let event = Event {
                event_type,
                time: SystemTime::now(),
                name,
            };
            if let Some(ref mut callback) = GLOBAL_CALLBACK {
                callback(event);
            }
        }
    }
    CallNextHookEx(HOOK, code, param, lpdata)
}

pub fn listen<T>(callback: T) -> Result<(), ListenError>
where
    T: FnMut(Event) + 'static,
{
    unsafe {
        GLOBAL_CALLBACK = Some(Box::new(callback));
        set_key_hook(raw_callback)?;
        set_mouse_hook(raw_callback)?;

        let (sender, receiver) = mpsc::channel();
        STOP_LOOP = Some(Box::new(move || {
            sender.send(true).unwrap();
        }));
        loop {
            if let Ok(stop_listen) = receiver.try_recv() {
                if stop_listen {
                    break;
                }
            }
            PeekMessageA(null_mut(), null_mut(), 0, 0, 0);
        }
    }
    Ok(())
}

pub fn stop_listen() {
    unsafe {
        if let Some(stop_loop) = STOP_LOOP.as_ref() {
            stop_loop();
            STOP_LOOP = None;
        }
    }
}

type DynFn = dyn Fn() + 'static;
pub static mut STOP_LOOP: Option<Box<DynFn>> = None;
