use crate::rdev::{Event, EventType, ListenError};
use crate::windows::common::{
    convert_keyboard, convert_raw_input, register_raw_input_mouse, set_key_hook, HookError, HOOK,
    KEYBOARD,
};
use std::mem::zeroed;
use std::os::raw::c_int;
use std::ptr::null_mut;
use std::time::SystemTime;
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{
    CallNextHookEx, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW,
    RegisterClassW, TranslateMessage, CW_USEDEFAULT, HC_ACTION, HWND_MESSAGE, MSG, WM_INPUT,
    WNDCLASSW, WS_OVERLAPPEDWINDOW,
};

static mut GLOBAL_CALLBACK: Option<Box<dyn FnMut(Event)>> = None;

impl From<HookError> for ListenError {
    fn from(error: HookError) -> Self {
        match error {
            HookError::Mouse(code) => ListenError::MouseHookError(code),
            HookError::Key(code) => ListenError::KeyHookError(code),
            HookError::RawInput(code) => ListenError::MouseHookError(code),
        }
    }
}

/// Keyboard hook callback
unsafe extern "system" fn keyboard_callback(code: c_int, param: WPARAM, lpdata: LPARAM) -> LRESULT {
    unsafe {
        if code == HC_ACTION {
            let opt = convert_keyboard(param, lpdata);
            if let Some(event_type) = opt {
                let name = match &event_type {
                    EventType::KeyPress(_key) => match (*KEYBOARD).lock() {
                        Ok(mut keyboard) => keyboard.get_name(lpdata),
                        Err(_) => None,
                    },
                    _ => None,
                };
                let event = Event {
                    event_type,
                    time: SystemTime::now(),
                    name,
                };
                let ptr = &raw mut GLOBAL_CALLBACK;
                if let Some(callback) = &mut *ptr {
                    callback(event);
                }
            }
        }
        CallNextHookEx(HOOK, code, param, lpdata)
    }
}

/// Window procedure for receiving Raw Input messages
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        if msg == WM_INPUT {
            let events = convert_raw_input(lparam);
            for event_type in events {
                let event = Event {
                    event_type,
                    time: SystemTime::now(),
                    name: None,
                };
                let ptr = &raw mut GLOBAL_CALLBACK;
                if let Some(callback) = &mut *ptr {
                    callback(event);
                }
            }
            return 0;
        }
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

/// Create a message-only window for receiving Raw Input
unsafe fn create_message_window() -> Option<HWND> {
    unsafe {
        let class_name: Vec<u16> = "RdevRawInputClass\0".encode_utf16().collect();
        let hinstance = GetModuleHandleW(null_mut());

        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(window_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: null_mut(),
            lpszMenuName: null_mut(),
            lpszClassName: class_name.as_ptr(),
        };

        if RegisterClassW(&wc) == 0 {
            // Class might already be registered, continue anyway
        }

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            null_mut(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            HWND_MESSAGE, // Message-only window
            null_mut(),
            hinstance,
            null_mut(),
        );

        if hwnd.is_null() {
            None
        } else {
            Some(hwnd)
        }
    }
}

pub fn listen<T>(callback: T) -> Result<(), ListenError>
where
    T: FnMut(Event) + 'static,
{
    unsafe {
        GLOBAL_CALLBACK = Some(Box::new(callback));

        // Create message window for Raw Input
        let hwnd = create_message_window().ok_or(ListenError::MouseHookError(0))?;

        // Register for Raw Input mouse events (provides true relative motion)
        register_raw_input_mouse(hwnd)?;

        // Set keyboard hook (hooks still work well for keyboard)
        set_key_hook(keyboard_callback)?;

        // Message loop
        let mut msg: MSG = zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}
