use crate::rdev::{Event, EventType, GrabError};
use crate::windows::common::{HOOK, HookError, KEYBOARD, convert, set_key_hook, set_mouse_hook};
use std::mem::zeroed;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::SystemTime;
use winapi::um::processthreadsapi::GetCurrentThreadId;
use winapi::um::winuser::{
    CallNextHookEx, DispatchMessageA, GetMessageA, HC_ACTION, MSG, PostThreadMessageA,
    TranslateMessage, WM_QUIT,
};

static mut GLOBAL_CALLBACK: Option<Box<dyn FnMut(Event) -> Option<Event>>> = None;
static HOOK_THREAD_ID: AtomicU32 = AtomicU32::new(0);

unsafe extern "system" fn raw_callback(code: i32, param: usize, lpdata: isize) -> isize {
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
                    if callback(event).is_none() {
                        // https://stackoverflow.com/questions/42756284/blocking-windows-mouse-click-using-setwindowshookex
                        // https://android.developreference.com/article/14560004/Blocking+windows+mouse+click+using+SetWindowsHookEx()
                        // https://cboard.cprogramming.com/windows-programming/99678-setwindowshookex-wm_keyboard_ll.html
                        // let _result = CallNextHookEx(HOOK, code, param, lpdata);
                        return 1;
                    }
                }
            }
        }
        CallNextHookEx(HOOK, code, param, lpdata)
    }
}
impl From<HookError> for GrabError {
    fn from(error: HookError) -> Self {
        match error {
            HookError::Mouse(code) => GrabError::MouseHookError(code),
            HookError::Key(code) => GrabError::KeyHookError(code),
        }
    }
}

pub fn grab<T>(callback: T) -> Result<(), GrabError>
where
    T: FnMut(Event) -> Option<Event> + 'static,
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

pub fn stop_grab() {
    let tid = HOOK_THREAD_ID.load(Ordering::SeqCst);
    if tid != 0 {
        unsafe {
            PostThreadMessageA(tid, WM_QUIT, 0, 0);
        }
    }
}
