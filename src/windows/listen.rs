use crate::rdev::{Button, Callback, Event, EventType, ListenError};
use crate::rdev::{Callback, Event, EventType, ListenError};
use crate::windows::common::{convert, get_name, HOOK};
use crate::windows::keycodes::key_from_code;
use std::convert::TryInto;
use std::os::raw::{c_int, c_short};
use std::ptr::null_mut;
use std::ptr::null_mut;
use std::time::SystemTime;
use std::time::SystemTime;
use winapi::shared::minwindef::{BYTE, DWORD, HIWORD, HKL, LPARAM, LRESULT, UINT, WORD, WPARAM};
use winapi::shared::ntdef::{LONG, WCHAR};
use winapi::shared::windef::HHOOK;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winuser::{
    CallNextHookEx, GetMessageA, SetWindowsHookExA, HC_ACTION, WH_KEYBOARD_LL, WH_MOUSE_LL,
};

fn default_callback(event: Event) {
    println!("Default : Event {:?}", event);
}
static mut GLOBAL_CALLBACK: Callback = default_callback;
static mut HOOK: HHOOK = null_mut();

unsafe fn get_code(lpdata: LPARAM) -> DWORD {
    let kb = *(lpdata as *const KBDLLHOOKSTRUCT);
    kb.vkCode
}
unsafe fn get_scan_code(lpdata: LPARAM) -> DWORD {
    let kb = *(lpdata as *const KBDLLHOOKSTRUCT);
    kb.scanCode
}
unsafe fn get_point(lpdata: LPARAM) -> (LONG, LONG) {
    let mouse = *(lpdata as *const MSLLHOOKSTRUCT);
    (mouse.pt.x, mouse.pt.y)
}

// https://docs.microsoft.com/en-us/previous-versions/windows/desktop/legacy/ms644986(v=vs.85)
/// confusingly, this function returns a WORD (unsigned), but may be
/// interpreted as either signed or unsigned depending on context
unsafe fn get_delta(lpdata: LPARAM) -> WORD {
    let mouse = *(lpdata as *const MSLLHOOKSTRUCT);
    HIWORD(mouse.mouseData)
}
unsafe fn get_button_code(lpdata: LPARAM) -> WORD {
    let mouse = *(lpdata as *const MSLLHOOKSTRUCT);
    HIWORD(mouse.mouseData)
}

static mut LAST_CODE: UINT = 0;
static mut LAST_SCAN_CODE: UINT = 0;
static mut LAST_STATE: [BYTE; 256] = [0; 256];
static mut LAST_IS_DEAD: bool = false;

unsafe fn get_name(lpdata: LPARAM) -> Option<String> {
    // https://gist.github.com/akimsko/2011327
    // https://www.experts-exchange.com/questions/23453780/LowLevel-Keystroke-Hook-removes-Accents-on-French-Keyboard.html
    let code = get_code(lpdata);
    let scan_code = get_scan_code(lpdata);

    const BUF_LEN: i32 = 32;
    let mut buff = [0 as WCHAR; BUF_LEN as usize];
    let buff_ptr = buff.as_mut_ptr();
    let mut state = [0 as BYTE; 256];
    let state_ptr = state.as_mut_ptr();

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
    let result = match len {
        0 => None,
        -1 => {
            is_dead = true;
            clear_keyboard_buffer(code, scan_code, layout);
            None
        }
        len if len > 0 => String::from_utf16(&buff[..len as usize]).ok(),
        _ => None,
    };

    if LAST_CODE != 0 && LAST_IS_DEAD {
        buff = [0; 32];
        let buff_ptr = buff.as_mut_ptr();
        let last_state_ptr = LAST_STATE.as_mut_ptr();
        ToUnicodeEx(
            LAST_CODE,
            LAST_SCAN_CODE,
            last_state_ptr,
            buff_ptr,
            BUF_LEN,
            0,
            layout,
        );
        LAST_CODE = 0;
    } else {
        LAST_CODE = code;
        LAST_SCAN_CODE = scan_code;
        LAST_IS_DEAD = is_dead;
        LAST_STATE.copy_from_slice(&state);
    }
    result
}

unsafe fn clear_keyboard_buffer(code: UINT, scan_code: UINT, layout: HKL) {
    const BUF_LEN: i32 = 32;
    let mut buff = [0 as WCHAR; BUF_LEN as usize];
    let buff_ptr = buff.as_mut_ptr();
    let mut state = [0 as BYTE; 256];
    let state_ptr = state.as_mut_ptr();

    let mut len = -1;
    while len < 0 {
        len = ToUnicodeEx(code, scan_code, state_ptr, buff_ptr, BUF_LEN, 0, layout);
    }
}

unsafe extern "system" fn raw_callback(code: c_int, param: WPARAM, lpdata: LPARAM) -> LRESULT {
    if code == HC_ACTION {
        let opt = match param.try_into() {
            Ok(WM_KEYDOWN) => {
                let code = get_code(lpdata);
                let key = key_from_code(code as u16);
                Some(EventType::KeyPress(key))
            }
            Ok(WM_KEYUP) => {
                let code = get_code(lpdata);
                let key = key_from_code(code as u16);
                Some(EventType::KeyRelease(key))
            }
            Ok(WM_LBUTTONDOWN) => Some(EventType::ButtonPress(Button::Left)),
            Ok(WM_LBUTTONUP) => Some(EventType::ButtonRelease(Button::Left)),
            Ok(WM_MBUTTONDOWN) => Some(EventType::ButtonPress(Button::Middle)),
            Ok(WM_MBUTTONUP) => Some(EventType::ButtonRelease(Button::Middle)),
            Ok(WM_RBUTTONDOWN) => Some(EventType::ButtonPress(Button::Right)),
            Ok(WM_RBUTTONUP) => Some(EventType::ButtonRelease(Button::Right)),
            Ok(WM_XBUTTONDOWN) => {
                let code = get_button_code(lpdata) as u8;
                Some(EventType::ButtonPress(Button::Unknown(code)))
            }
            Ok(WM_XBUTTONUP) => {
                let code = get_button_code(lpdata) as u8;
                Some(EventType::ButtonRelease(Button::Unknown(code)))
            }
            Ok(WM_MOUSEMOVE) => {
                let (x, y) = get_point(lpdata);
                Some(EventType::MouseMove {
                    x: x as f64,
                    y: y as f64,
                })
            }
            Ok(WM_MOUSEWHEEL) => {
                let delta = get_delta(lpdata) as c_short;
                Some(EventType::Wheel {
                    delta_x: 0,
                    delta_y: (delta / WHEEL_DELTA) as i64,
                })
            }
            Ok(WM_MOUSEHWHEEL) => {
                let delta = get_delta(lpdata) as c_short;
                Some(EventType::Wheel {
                    delta_x: (delta / WHEEL_DELTA) as i64,
                    delta_y: 0,
                })
            }
            _ => None,
        };

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
