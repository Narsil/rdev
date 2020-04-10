use crate::rdev::{Callback, Event, EventType, ListenError};
use crate::windows::common::{convert, set_key_hook, set_mouse_hook, HookError, HOOK};
use crate::windows::keyboard_state::KeyboardState;
use lazy_static::lazy_static;
use std::os::raw::c_int;
use std::ptr::null_mut;
use std::sync::Mutex;
use std::time::SystemTime;
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::um::winuser::{CallNextHookEx, GetMessageA, HC_ACTION};

fn default_callback(event: Event) {
    println!("Default : Event {:?}", event);
}
static mut GLOBAL_CALLBACK: Callback = default_callback;

lazy_static! {
    static ref STATE: Mutex<KeyboardState> = Mutex::new(KeyboardState::new().unwrap());
}

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
                EventType::KeyPress(_key) => (*STATE).lock().unwrap().get_name(lpdata),
                _ => None,
            };
            let event = Event {
                event_type,
                time: SystemTime::now(),
                name,
            };
            GLOBAL_CALLBACK(event);
        }
    }
    CallNextHookEx(HOOK, code, param, lpdata)
}

pub fn listen(callback: Callback) -> Result<(), ListenError> {
    unsafe {
        GLOBAL_CALLBACK = callback;
        set_key_hook(raw_callback)?;
        set_mouse_hook(raw_callback)?;

        GetMessageA(null_mut(), null_mut(), 0, 0);
    }
    Ok(())
}
