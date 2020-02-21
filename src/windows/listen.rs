use crate::rdev::{Button, Callback, Event, EventType};
use crate::windows::keycodes::key_from_code;
use std::ptr::null_mut;
use std::time::SystemTime;
use winapi::shared::windef::HHOOK;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winuser::{
    CallNextHookEx, GetMessageA, SetWindowsHookExA, HC_ACTION, KBDLLHOOKSTRUCT, MSLLHOOKSTRUCT,
    WHEEL_DELTA, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN,
    WM_RBUTTONUP, WM_XBUTTONDOWN, WM_XBUTTONUP,
};

fn default_callback(event: Event) {
    println!("Default : Event {:?}", event);
}

static mut GLOBAL_CALLBACK: Callback = default_callback;
static mut HOOK: HHOOK = null_mut();

unsafe fn get_code(lpdata: isize) -> u32 {
    let kb = *(lpdata as *const KBDLLHOOKSTRUCT);
    kb.vkCode
}
unsafe fn get_point(lpdata: isize) -> (i32, i32) {
    let mouse = *(lpdata as *const MSLLHOOKSTRUCT);
    (mouse.pt.x, mouse.pt.y)
}
/// https://docs.microsoft.com/en-us/previous-versions/windows/desktop/legacy/ms644986(v=vs.85)
unsafe fn get_delta(lpdata: isize) -> i32 {
    let mouse = *(lpdata as *const MSLLHOOKSTRUCT);
    (mouse.mouseData as i32 >> 16) as i32
}
unsafe fn get_button_code(lpdata: isize) -> i32 {
    let mouse = *(lpdata as *const MSLLHOOKSTRUCT);
    (mouse.mouseData as i32 >> 16) as i32
}

unsafe extern "system" fn raw_callback(code: i32, param: usize, lpdata: isize) -> isize {
    if code == HC_ACTION {
        let opt = match param {
            x if x == WM_KEYDOWN as usize => {
                let code = get_code(lpdata);
                let key = key_from_code(code as u16);
                Some(EventType::KeyPress(key))
            }
            x if x == WM_KEYUP as usize => {
                let code = get_code(lpdata);
                let key = key_from_code(code as u16);
                Some(EventType::KeyRelease(key))
            }
            x if x == WM_LBUTTONDOWN as usize => Some(EventType::ButtonPress(Button::Left)),
            x if x == WM_LBUTTONUP as usize => Some(EventType::ButtonRelease(Button::Left)),
            x if x == WM_MBUTTONDOWN as usize => Some(EventType::ButtonPress(Button::Middle)),
            x if x == WM_MBUTTONUP as usize => Some(EventType::ButtonRelease(Button::Middle)),
            x if x == WM_RBUTTONDOWN as usize => Some(EventType::ButtonPress(Button::Right)),
            x if x == WM_RBUTTONUP as usize => Some(EventType::ButtonRelease(Button::Right)),
            x if x == WM_XBUTTONDOWN as usize => {
                let code = get_button_code(lpdata) as u8;
                Some(EventType::ButtonPress(Button::Unknown(code)))
            }
            x if x == WM_XBUTTONUP as usize => {
                let code = get_button_code(lpdata) as u8;

                Some(EventType::ButtonRelease(Button::Unknown(code)))
            }

            x if x == WM_MOUSEMOVE as usize => {
                let (x, y) = get_point(lpdata);
                Some(EventType::MouseMove {
                    x: x as f64,
                    y: y as f64,
                })
            }
            x if x == WM_MOUSEWHEEL as usize => {
                let delta = get_delta(lpdata);
                Some(EventType::Wheel {
                    delta_x: 0,
                    delta_y: (delta as i64) / WHEEL_DELTA as i64,
                })
            }
            x if x == WM_MOUSEHWHEEL as usize => {
                let delta = get_delta(lpdata);

                Some(EventType::Wheel {
                    delta_x: (delta as i64) / WHEEL_DELTA as i64,
                    delta_y: 0,
                })
            }
            _ => None,
        };

        if let Some(event_type) = opt {
            let event = Event {
                event_type,
                time: SystemTime::now(),
                name: None,
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

pub fn listen(callback: Callback) {
    unsafe {
        GLOBAL_CALLBACK = callback;
        set_key_hook();
        set_mouse_hook();

        GetMessageA(null_mut(), null_mut(), 0, 0);
    }
}
