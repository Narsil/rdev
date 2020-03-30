use crate::rdev::{Event, EventType, GrabCallback, GrabError};
use crate::windows::common::{convert, get_name, HOOK};
use std::ptr::null_mut;
use std::time::SystemTime;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winuser::{
    CallNextHookEx, GetMessageA, SetWindowsHookExA, HC_ACTION, WH_KEYBOARD_LL, WH_MOUSE_LL,
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
                // https://stackoverflow.com/questions/42756284/blocking-windows-mouse-click-using-setwindowshookex
                // https://android.developreference.com/article/14560004/Blocking+windows+mouse+click+using+SetWindowsHookEx()
                // https://cboard.cprogramming.com/windows-programming/99678-setwindowshookex-wm_keyboard_ll.html
                let _result = CallNextHookEx(HOOK, code, param, lpdata);
                return 1;
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
