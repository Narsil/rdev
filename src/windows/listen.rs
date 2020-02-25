use crate::rdev::{Button, Callback, Event, EventType};
use crate::windows::keycodes::key_from_code;
use std::ptr::null_mut;
use std::time::SystemTime;
use winapi::shared::minwindef::HKL;
use winapi::shared::windef::HHOOK;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::processthreadsapi::GetCurrentThreadId;
use winapi::um::winuser;
use winapi::um::winuser::{
    CallNextHookEx, GetForegroundWindow, GetKeyState, GetKeyboardLayout, GetKeyboardState,
    GetMessageA, GetWindowThreadProcessId, SetWindowsHookExA, ToUnicodeEx, HC_ACTION,
    KBDLLHOOKSTRUCT, MSLLHOOKSTRUCT, VK_SHIFT, WHEEL_DELTA, WH_KEYBOARD_LL, WH_MOUSE_LL,
    WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP,
    WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_XBUTTONDOWN,
    WM_XBUTTONUP,
};

fn default_callback(event: Event) {
    println!("Default : Event {:?}", event);
}

const TRUE: i32 = 1;
const FALSE: i32 = 0;

static mut GLOBAL_CALLBACK: Callback = default_callback;
static mut HOOK: HHOOK = null_mut();

unsafe fn get_code(lpdata: isize) -> u32 {
    let kb = *(lpdata as *const KBDLLHOOKSTRUCT);
    kb.vkCode
}
unsafe fn get_scan_code(lpdata: isize) -> u32 {
    let kb = *(lpdata as *const KBDLLHOOKSTRUCT);
    kb.scanCode
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

static mut LAST_CODE: u32 = 0;
static mut LAST_SCAN_CODE: u32 = 0;
static mut LAST_STATE: [u8; 256] = [0; 256];
static mut LAST_IS_DEAD: bool = false;

unsafe fn get_name(lpdata: isize) -> Option<String> {
    // https://gist.github.com/akimsko/2011327
    // https://www.experts-exchange.com/questions/23453780/LowLevel-Keystroke-Hook-removes-Accents-on-French-Keyboard.html
    let code = get_code(lpdata);
    let scan_code = get_scan_code(lpdata);
    let mut buff: [u16; 32] = [0; 32];
    let buff_ptr = &mut buff as *mut _ as *mut u16;
    let mut state: [u8; 256] = [0; 256];
    let state_ptr: *mut u8 = &mut state as *const _ as *mut u8;

    let _shift = GetKeyState(VK_SHIFT);
    let current_window_thread_id = GetWindowThreadProcessId(GetForegroundWindow(), null_mut());
    let thread_id = GetCurrentThreadId();
    // Attach to active thread so we can get that keyboard state
    let status = if winuser::AttachThreadInput(thread_id, current_window_thread_id, TRUE) == 1 {
        // Current state of the modifiers in keyboard
        let status = GetKeyboardState(state_ptr);

        // Detach
        winuser::AttachThreadInput(thread_id, current_window_thread_id, FALSE);
        status
    } else {
        // Could not attach, perhaps it is this process?
        GetKeyboardState(state_ptr)
    };

    if status != 1 {
        return None;
    }
    let layout = GetKeyboardLayout(current_window_thread_id);
    let len = ToUnicodeEx(code, scan_code, state_ptr, buff_ptr, 8 - 1, 0, layout);

    let mut is_dead = false;
    let result = if len > 0 {
        match String::from_utf16(&buff[0..len as usize]) {
            Ok(string) => Some(string),
            Err(_) => None,
        }
    } else if len == 0 {
        None
    } else if len == -1 {
        is_dead = true;
        clear_keyboard_buffer(code, scan_code, layout);
        None
    } else {
        None
    };

    if LAST_CODE != 0 && LAST_IS_DEAD {
        let mut buff: [u16; 32] = [0; 32];
        let buff_ptr = &mut buff as *mut _ as *mut u16;
        let last_state_ptr = &LAST_STATE as *const _ as *mut u8;
        ToUnicodeEx(
            LAST_CODE,
            LAST_SCAN_CODE,
            last_state_ptr,
            buff_ptr,
            8 - 1,
            0,
            layout,
        );
        LAST_CODE = 0;
        return result;
    }

    LAST_CODE = code;
    LAST_SCAN_CODE = scan_code;
    LAST_IS_DEAD = is_dead;
    LAST_STATE.clone_from_slice(&state);

    result
}

unsafe fn clear_keyboard_buffer(code: u32, scan_code: u32, layout: HKL) {
    let mut len = -1;
    let mut buff: [u16; 32] = [0; 32];
    let buff_ptr = &mut buff as *mut _ as *mut u16;
    let mut state: [u8; 256] = [0; 256];
    let state_ptr: *mut u8 = &mut state as *const _ as *mut u8;

    while len < 0 {
        len = ToUnicodeEx(code, scan_code, state_ptr, buff_ptr, 8 - 1, 0, layout);
    }
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

pub fn listen(callback: Callback) {
    unsafe {
        GLOBAL_CALLBACK = callback;
        set_key_hook();
        set_mouse_hook();

        GetMessageA(null_mut(), null_mut(), 0, 0);
    }
}
