extern crate winapi;

use crate::rdev::{Callback, Event, EventType, SimulateError};
use std::mem::{size_of, transmute, transmute_copy};
use std::ptr::null_mut;
use std::time::SystemTime;
use winapi::ctypes::c_int;
use winapi::shared::windef::{HHOOK};
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winuser::{
    CallNextHookEx, GetMessageA, GetSystemMetrics, SendInput, SetWindowsHookExA, HC_ACTION, INPUT,
    INPUT_KEYBOARD, INPUT_MOUSE, KBDLLHOOKSTRUCT, KEYBDINPUT, KEYEVENTF_KEYUP, LPINPUT,
    MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_HWHEEL, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, MOUSEINPUT,
    MSLLHOOKSTRUCT, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, WHEEL_DELTA, WH_KEYBOARD_LL,
    WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN, WM_MBUTTONUP,
    WM_MOUSEHWHEEL, WM_MOUSEMOVE, WM_MOUSEWHEEL, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_XBUTTONDOWN,
    WM_XBUTTONUP, 
};

static KEYEVENTF_KEYDOWN: u32 = 0; // Not defined in win32 but define here for clarity

fn default_callback(event: Event) {
    println!("Default : Event {:?}", event);
}

static mut GLOBAL_CALLBACK: Callback = default_callback;
static mut HOOK: HHOOK = null_mut();

unsafe fn get_code(lpdata: isize) -> u8 {
    let kb = *(lpdata as *const KBDLLHOOKSTRUCT);
    kb.vkCode as u8
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
                Some(EventType::KeyPress { code })
            }
            x if x == WM_KEYUP as usize => {
                let code = get_code(lpdata);
                Some(EventType::KeyRelease { code })
            }
            x if x == WM_LBUTTONDOWN as usize => Some(EventType::ButtonPress { code: 1 }),
            x if x == WM_LBUTTONUP as usize => Some(EventType::ButtonRelease { code: 1 }),
            x if x == WM_MBUTTONDOWN as usize => Some(EventType::ButtonPress { code: 2 }),
            x if x == WM_MBUTTONUP as usize => Some(EventType::ButtonRelease { code: 2 }),
            x if x == WM_RBUTTONDOWN as usize => Some(EventType::ButtonPress { code: 3 }),
            x if x == WM_RBUTTONUP as usize => Some(EventType::ButtonRelease { code: 3 }),
            x if x == WM_XBUTTONDOWN as usize => {
                let code = get_button_code(lpdata) as u8 + 3;
                Some(EventType::ButtonPress { code })
            }
            x if x == WM_XBUTTONUP as usize => {
                let code = get_button_code(lpdata) as u8 + 3;

                Some(EventType::ButtonRelease { code })
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
    if hook == null_mut() {
        let error = GetLastError();
        panic!("Can't set system hook! {:?}", error)
    }
    HOOK = hook;
}
unsafe fn set_mouse_hook() {
    let hook = SetWindowsHookExA(WH_MOUSE_LL, Some(raw_callback), null_mut(), 0);
    if hook == null_mut() {
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

fn mouse_event(flags: u32, data: u32, dx: i32, dy: i32) -> Result<(), SimulateError> {
    let mut input = INPUT {
        type_: INPUT_MOUSE,
        u: unsafe {
            transmute(MOUSEINPUT {
                dx,
                dy,
                mouseData: data,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            })
        },
    };
    let value = unsafe { SendInput(1, &mut input as LPINPUT, size_of::<INPUT>() as c_int) };
    if value!= 1{
        Err(SimulateError)
    }else{
        Ok(())
    }
}

fn keyboard_event(flags: u32, vk: u16, scan: u16) -> Result<(), SimulateError> {
    let mut input = INPUT {
        type_: INPUT_KEYBOARD,
        u: unsafe {
            transmute_copy(&KEYBDINPUT {
                wVk: vk,
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            })
        },
    };
    let value = unsafe { SendInput(1, &mut input as LPINPUT, size_of::<INPUT>() as c_int) };
    if value != 1{
        Err(SimulateError)
    }else{
        Ok(())
    }
}

pub fn simulate(event_type: &EventType) -> Result<(), SimulateError> {
    match event_type {
        EventType::KeyPress { code } => keyboard_event(KEYEVENTF_KEYDOWN, *code as u16, 0),
        EventType::KeyRelease { code } => keyboard_event(KEYEVENTF_KEYUP, *code as u16, 0),
        EventType::ButtonPress { code } => match code {
            1 => mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0),
            2 => mouse_event(MOUSEEVENTF_MIDDLEDOWN, 0, 0, 0),
            3 => mouse_event(MOUSEEVENTF_RIGHTDOWN, 0, 0, 0),
            _ => mouse_event(MOUSEEVENTF_XDOWN, 0, 0, (code - 3) as i32),
        },
        EventType::ButtonRelease { code } => match code {
            1 => mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0),
            2 => mouse_event(MOUSEEVENTF_MIDDLEUP, 0, 0, 0),
            3 => mouse_event(MOUSEEVENTF_RIGHTUP, 0, 0, 0),
            _ => mouse_event(MOUSEEVENTF_XUP, 0, 0, (code - 3) as i32),
        },
        EventType::Wheel { delta_x, delta_y } => {
            let result_x = if *delta_x != 0 {
                mouse_event(
                    MOUSEEVENTF_HWHEEL,
                    0,
                    (delta_x * WHEEL_DELTA as i64) as i32,
                    0,
                )
            } else {
                Ok(())
            };

            let result_y = if *delta_y != 0 {
                mouse_event(
                    MOUSEEVENTF_WHEEL,
                    0,
                    0,
                    (delta_y * WHEEL_DELTA as i64) as i32,
                )
            } else {
                Ok(())
            };

            if result_x.is_ok() && result_y.is_ok() {
                Ok(())
            } else {
                Err(SimulateError)
            }
        }
        EventType::MouseMove { x, y } => {
            let width = unsafe { GetSystemMetrics(SM_CXVIRTUALSCREEN) };
            let height = unsafe { GetSystemMetrics(SM_CYVIRTUALSCREEN) };


            mouse_event(
                MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
                0,
                (*x * 65335.0) as i32 / width,
                (*y * 65335.0) as i32 / height,
            )
        }
    }
}
