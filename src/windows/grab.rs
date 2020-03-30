use crate::rdev::{Event, EventType, GrabCallback, GrabError};
use crate::windows::common::{convert, get_name, HOOK};
use std::ptr::null_mut;
use std::time::SystemTime;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winuser::{
    CallNextHookEx, GetMessageA, SetWindowsHookExA, HC_ACTION, LPMSG, WH_KEYBOARD_LL, WH_MOUSE_LL,
    WM_NULL,
};

fn default_callback(event: Event) -> Option<Event> {
    println!("Default : Event {:?}", event);
    Some(event)
}
static mut GLOBAL_CALLBACK: GrabCallback = default_callback;

unsafe extern "system" fn raw_callback(code: i32, param: usize, lpdata: isize) -> isize {
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
            if GLOBAL_CALLBACK(event).is_none() {
                let result = CallNextHookEx(HOOK, code, param, lpdata);
                let msg: LPMSG = param as LPMSG;
                (*msg).message = WM_NULL;
                return result;
            }
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

pub fn grab(callback: GrabCallback) -> Result<(), GrabError> {
    unsafe {
        GLOBAL_CALLBACK = callback;
        set_key_hook();
        set_mouse_hook();

        GetMessageA(null_mut(), null_mut(), 0, 0);
    }
    Ok(())
}
