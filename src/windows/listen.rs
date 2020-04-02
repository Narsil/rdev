use crate::rdev::{Callback, Event, EventType, ListenError};
use crate::windows::common::{convert, get_name, HOOK};
use std::os::raw::c_int;
use std::ptr::null_mut;
use std::time::SystemTime;
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winuser::{
    CallNextHookEx, GetMessageA, SetWindowsHookExA, HC_ACTION, WH_KEYBOARD_LL, WH_MOUSE_LL,
};

fn default_callback(event: Event) {
    println!("Default : Event {:?}", event);
}
static mut GLOBAL_CALLBACK: Callback = default_callback;

unsafe extern "system" fn raw_callback(code: c_int, param: WPARAM, lpdata: LPARAM) -> LRESULT {
    if code == HC_ACTION {
        let opt = convert(param, lpdata);
        if let Some(event_type) = opt {
            let name = match &event_type {
                EventType::KeyPress(_key) => get_name(lpdata),
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

unsafe fn set_key_hook() {
    let hook = SetWindowsHookExA(WH_KEYBOARD_LL, Some(raw_callback), null_mut(), 0);
    if hook.is_null() {
        let error = GetLastError();
        panic!("Can't set system hook! {:?}", error)
    }
    HOOK = hook;
}

unsafe fn set_mouse_hook() {
    let hook = SetWindowsHookExA(WH_MOUSE_LL, Some(raw_callback), null_mut(), 0);
    if hook.is_null() {
        let error = GetLastError();
        panic!("Can't set system hook! {:?}", error)
    }
    HOOK = hook;
}

pub fn listen(callback: Callback) -> Result<(), ListenError> {
    unsafe {
        GLOBAL_CALLBACK = callback;
        set_key_hook();
        set_mouse_hook();

        GetMessageA(null_mut(), null_mut(), 0, 0);
    }
    Ok(())
}
