use std::mem::zeroed;
use crate::rdev::{Event, EventType, ListenError};
use crate::windows::common::{HOOK, HookError, KEYBOARD, convert, set_key_hook, set_mouse_hook};
use std::os::raw::c_int;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::SystemTime;
use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::um::processthreadsapi::GetCurrentThreadId;
use winapi::um::winuser::{CallNextHookEx, GetMessageA, HC_ACTION, PostThreadMessageA, WM_QUIT, MSG, TranslateMessage, DispatchMessageA};

static mut GLOBAL_CALLBACK: Option<Box<dyn FnMut(Event)>> = None;
static HOOK_THREAD_ID: AtomicU32 = AtomicU32::new(0);

impl From<HookError> for ListenError {
    fn from(error: HookError) -> Self {
        match error {
            HookError::Mouse(code) => ListenError::MouseHookError(code),
            HookError::Key(code) => ListenError::KeyHookError(code),
        }
    }
}

unsafe extern "system" fn raw_callback(code: c_int, param: WPARAM, lpdata: LPARAM) -> LRESULT {
    unsafe {
        if code == HC_ACTION {
            let opt = convert(param, lpdata);
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

pub fn listen<T>(callback: T) -> Result<(), ListenError>
where
    T: FnMut(Event) + 'static,
{
    unsafe {
        GLOBAL_CALLBACK = Some(Box::new(callback));
        set_key_hook(raw_callback)?;
        set_mouse_hook(raw_callback)?;

        let tid = GetCurrentThreadId();
        HOOK_THREAD_ID.store(tid, Ordering::SeqCst);

        let mut msg: MSG = zeroed();
        while GetMessageA(&mut msg, Default::default(), 0, 0) == 1 {
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }
    }
    Ok(())
}

pub fn stop_listen() {
    let tid = HOOK_THREAD_ID.load(Ordering::SeqCst);
    if tid != 0 {
        unsafe {
            PostThreadMessageA(tid, WM_QUIT, 0, 0);
        }
    }
}
