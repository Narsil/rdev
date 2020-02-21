use crate::rdev::{EventType, SimulateError};
use crate::windows::keycodes::code_from_key;
use std::mem::{size_of, transmute, transmute_copy};
use winapi::ctypes::c_int;
use winapi::um::winuser::{
    GetSystemMetrics, SendInput, INPUT, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_KEYUP,
    LPINPUT, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_HWHEEL, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN,
    MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEEVENTF_XDOWN, MOUSEEVENTF_XUP, MOUSEINPUT,
    SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, WHEEL_DELTA,
};
/// Not defined in win32 but define here for clarity
static KEYEVENTF_KEYDOWN: u32 = 0;

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
    if value != 1 {
        Err(SimulateError)
    } else {
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
    if value != 1 {
        Err(SimulateError)
    } else {
        Ok(())
    }
}

pub fn simulate(event_type: &EventType) -> Result<(), SimulateError> {
    match event_type {
        EventType::KeyPress(key) => {
            if let Some(code) = code_from_key(key) {
                keyboard_event(KEYEVENTF_KEYDOWN, code, 0)
            } else {
                Err(SimulateError)
            }
        }
        EventType::KeyRelease(key) => {
            if let Some(code) = code_from_key(key) {
                keyboard_event(KEYEVENTF_KEYUP, code, 0)
            } else {
                Err(SimulateError)
            }
        }
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
