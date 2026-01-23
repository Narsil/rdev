use crate::rdev::{Button, EventType};
use crate::windows::keyboard::Keyboard;
use crate::windows::keycodes::key_from_code;
use lazy_static::lazy_static;
use std::convert::TryInto;
use std::mem::{size_of, MaybeUninit};
use std::os::raw::{c_int, c_short};
use std::ptr::null_mut;
use std::sync::Mutex;
use winapi::shared::hidusage::{HID_USAGE_GENERIC_MOUSE, HID_USAGE_PAGE_GENERIC};
use winapi::shared::minwindef::{DWORD, HIWORD, LPARAM, LRESULT, UINT, WORD, WPARAM};
use winapi::shared::ntdef::LONG;
use winapi::shared::windef::HHOOK;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winuser::{
    GetRawInputData, KBDLLHOOKSTRUCT, MSLLHOOKSTRUCT, RAWINPUT, RAWINPUTDEVICE,
    RAWINPUTHEADER, RIDEV_INPUTSINK, RID_INPUT, RIM_TYPEMOUSE, RegisterRawInputDevices,
    SetWindowsHookExA, WH_KEYBOARD_LL, WH_MOUSE_LL, WHEEL_DELTA, WM_KEYDOWN, WM_KEYUP,
    WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEHWHEEL, WM_MOUSEMOVE,
    WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_SYSKEYDOWN, WM_SYSKEYUP, WM_XBUTTONDOWN,
    WM_XBUTTONUP,
};
pub const TRUE: i32 = 1;
pub const FALSE: i32 = 0;

// Mouse button flags from RAWMOUSE
pub const RI_MOUSE_LEFT_BUTTON_DOWN: WORD = 0x0001;
pub const RI_MOUSE_LEFT_BUTTON_UP: WORD = 0x0002;
pub const RI_MOUSE_RIGHT_BUTTON_DOWN: WORD = 0x0004;
pub const RI_MOUSE_RIGHT_BUTTON_UP: WORD = 0x0008;
pub const RI_MOUSE_MIDDLE_BUTTON_DOWN: WORD = 0x0010;
pub const RI_MOUSE_MIDDLE_BUTTON_UP: WORD = 0x0020;
pub const RI_MOUSE_BUTTON_4_DOWN: WORD = 0x0040;
pub const RI_MOUSE_BUTTON_4_UP: WORD = 0x0080;
pub const RI_MOUSE_BUTTON_5_DOWN: WORD = 0x0100;
pub const RI_MOUSE_BUTTON_5_UP: WORD = 0x0200;
pub const RI_MOUSE_WHEEL: WORD = 0x0400;
pub const RI_MOUSE_HWHEEL: WORD = 0x0800;

pub static mut HOOK: HHOOK = null_mut();
lazy_static! {
    pub(crate) static ref KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard::new().unwrap());
}

pub unsafe fn get_code(lpdata: LPARAM) -> DWORD {
    unsafe {
        let kb = *(lpdata as *const KBDLLHOOKSTRUCT);
        kb.vkCode
    }
}
pub unsafe fn get_scan_code(lpdata: LPARAM) -> DWORD {
    unsafe {
        let kb = *(lpdata as *const KBDLLHOOKSTRUCT);
        kb.scanCode
    }
}
pub unsafe fn get_point(lpdata: LPARAM) -> (LONG, LONG) {
    unsafe {
        let mouse = *(lpdata as *const MSLLHOOKSTRUCT);
        (mouse.pt.x, mouse.pt.y)
    }
}
// https://docs.microsoft.com/en-us/previous-versions/windows/desktop/legacy/ms644986(v=vs.85)
/// confusingly, this function returns a WORD (unsigned), but may be
/// interpreted as either signed or unsigned depending on context
pub unsafe fn get_delta(lpdata: LPARAM) -> WORD {
    unsafe {
        let mouse = *(lpdata as *const MSLLHOOKSTRUCT);
        HIWORD(mouse.mouseData)
    }
}
pub unsafe fn get_button_code(lpdata: LPARAM) -> WORD {
    unsafe {
        let mouse = *(lpdata as *const MSLLHOOKSTRUCT);
        HIWORD(mouse.mouseData)
    }
}

/// Convert keyboard hook events
pub unsafe fn convert_keyboard(param: WPARAM, lpdata: LPARAM) -> Option<EventType> {
    unsafe {
        match param.try_into() {
            Ok(WM_KEYDOWN) | Ok(WM_SYSKEYDOWN) => {
                let code = get_code(lpdata);
                let key = key_from_code(code as u16);
                Some(EventType::KeyPress(key))
            }
            Ok(WM_KEYUP) | Ok(WM_SYSKEYUP) => {
                let code = get_code(lpdata);
                let key = key_from_code(code as u16);
                Some(EventType::KeyRelease(key))
            }
            _ => None,
        }
    }
}

/// Convert Raw Input mouse events - provides true relative motion
pub unsafe fn convert_raw_input(lparam: LPARAM) -> Vec<EventType> {
    unsafe {
        let mut events = Vec::new();
        let mut size: UINT = 0;

        // Get the size of the raw input data
        if GetRawInputData(
            lparam as _,
            RID_INPUT,
            null_mut(),
            &mut size,
            size_of::<RAWINPUTHEADER>() as UINT,
        ) == UINT::MAX
        {
            return events;
        }

        // Use a properly aligned RAWINPUT buffer to avoid undefined behavior
        // RAWINPUT is variable-size, but for mouse input it fits in the struct
        let mut raw: MaybeUninit<RAWINPUT> = MaybeUninit::uninit();
        let mut actual_size = size_of::<RAWINPUT>() as UINT;
        
        if GetRawInputData(
            lparam as _,
            RID_INPUT,
            raw.as_mut_ptr() as _,
            &mut actual_size,
            size_of::<RAWINPUTHEADER>() as UINT,
        ) == UINT::MAX
        {
            return events;
        }

        let raw = raw.assume_init();

        // Only process mouse input
        if raw.header.dwType != RIM_TYPEMOUSE {
            return events;
        }

        let mouse = raw.data.mouse();
        let button_flags = mouse.usButtonFlags;

        // Handle mouse movement (relative only - no absolute position tracking)
        let delta_x = mouse.lLastX as f64;
        let delta_y = mouse.lLastY as f64;

        if delta_x != 0.0 || delta_y != 0.0 {
            // Report only relative deltas; x/y are set to 0 since Raw Input
            // only provides relative motion and we don't track absolute position
            events.push(EventType::MouseMove {
                x: 0.0,
                y: 0.0,
                delta_x,
                delta_y,
            });
        }

        // Handle mouse buttons
        if button_flags & RI_MOUSE_LEFT_BUTTON_DOWN != 0 {
            events.push(EventType::ButtonPress(Button::Left));
        }
        if button_flags & RI_MOUSE_LEFT_BUTTON_UP != 0 {
            events.push(EventType::ButtonRelease(Button::Left));
        }
        if button_flags & RI_MOUSE_RIGHT_BUTTON_DOWN != 0 {
            events.push(EventType::ButtonPress(Button::Right));
        }
        if button_flags & RI_MOUSE_RIGHT_BUTTON_UP != 0 {
            events.push(EventType::ButtonRelease(Button::Right));
        }
        if button_flags & RI_MOUSE_MIDDLE_BUTTON_DOWN != 0 {
            events.push(EventType::ButtonPress(Button::Middle));
        }
        if button_flags & RI_MOUSE_MIDDLE_BUTTON_UP != 0 {
            events.push(EventType::ButtonRelease(Button::Middle));
        }
        if button_flags & RI_MOUSE_BUTTON_4_DOWN != 0 {
            events.push(EventType::ButtonPress(Button::Unknown(4)));
        }
        if button_flags & RI_MOUSE_BUTTON_4_UP != 0 {
            events.push(EventType::ButtonRelease(Button::Unknown(4)));
        }
        if button_flags & RI_MOUSE_BUTTON_5_DOWN != 0 {
            events.push(EventType::ButtonPress(Button::Unknown(5)));
        }
        if button_flags & RI_MOUSE_BUTTON_5_UP != 0 {
            events.push(EventType::ButtonRelease(Button::Unknown(5)));
        }

        // Handle scroll wheel
        if button_flags & RI_MOUSE_WHEEL != 0 {
            let delta = mouse.usButtonData as c_short;
            events.push(EventType::Wheel {
                delta_x: 0,
                delta_y: (delta / WHEEL_DELTA as c_short) as i64,
            });
        }
        if button_flags & RI_MOUSE_HWHEEL != 0 {
            let delta = mouse.usButtonData as c_short;
            events.push(EventType::Wheel {
                delta_x: (delta / WHEEL_DELTA as c_short) as i64,
                delta_y: 0,
            });
        }

        events
    }
}

/// Legacy convert function for backward compatibility (uses hooks, no true deltas)
pub unsafe fn convert(param: WPARAM, lpdata: LPARAM) -> Option<EventType> {
    unsafe {
        match param.try_into() {
            Ok(WM_KEYDOWN) | Ok(WM_SYSKEYDOWN) => {
                let code = get_code(lpdata);
                let key = key_from_code(code as u16);
                Some(EventType::KeyPress(key))
            }
            Ok(WM_KEYUP) | Ok(WM_SYSKEYUP) => {
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
                // Low-level mouse hook only provides absolute coordinates.
                Some(EventType::MouseMove {
                    x: x as f64,
                    y: y as f64,
                    delta_x: 0.0,
                    delta_y: 0.0,
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
        }
    }
}

type RawCallback = unsafe extern "system" fn(code: c_int, param: WPARAM, lpdata: LPARAM) -> LRESULT;
pub enum HookError {
    Mouse(DWORD),
    Key(DWORD),
    RawInput(DWORD),
}

pub unsafe fn set_key_hook(callback: RawCallback) -> Result<(), HookError> {
    unsafe {
        let hook = SetWindowsHookExA(WH_KEYBOARD_LL, Some(callback), null_mut(), 0);

        if hook.is_null() {
            let error = GetLastError();
            return Err(HookError::Key(error));
        }
        HOOK = hook;
        Ok(())
    }
}

pub unsafe fn set_mouse_hook(callback: RawCallback) -> Result<(), HookError> {
    unsafe {
        let hook = SetWindowsHookExA(WH_MOUSE_LL, Some(callback), null_mut(), 0);
        if hook.is_null() {
            let error = GetLastError();
            return Err(HookError::Mouse(error));
        }
        HOOK = hook;
        Ok(())
    }
}

/// Register for Raw Input mouse events
pub unsafe fn register_raw_input_mouse(hwnd: winapi::shared::windef::HWND) -> Result<(), HookError> {
    unsafe {
        let rid = RAWINPUTDEVICE {
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            usUsage: HID_USAGE_GENERIC_MOUSE,
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: hwnd,
        };

        if RegisterRawInputDevices(&rid, 1, size_of::<RAWINPUTDEVICE>() as UINT) == 0 {
            let error = GetLastError();
            return Err(HookError::RawInput(error));
        }
        Ok(())
    }
}
