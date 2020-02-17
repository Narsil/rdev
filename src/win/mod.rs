extern crate winapi;

use crate::rdev::{Event, EventType, SimulateError, Callback};
use std::time::SystemTime;
use winapi::um::winuser::{GetMessageA, WH_KEYBOARD_LL,SetWindowsHookExW, CallNextHookEx, WM_KEYDOWN, HC_ACTION };
use winapi::um::errhandlingapi::{GetLastError};
use winapi::shared::windef::HHOOK;
use std::ptr::{null_mut};


fn default_callback(event: Event) {
    println!("Default : Event {:?}", event);
}

static mut GLOBAL_CALLBACK: Callback = default_callback;
static mut HOOK: HHOOK = null_mut();

unsafe extern "system" fn raw_callback(code: i32, param: usize, lpdata: isize) -> isize{
    println!("YOUHOU {:?} {:?} {:?}" ,code, param, lpdata );
    if code == HC_ACTION{
        let event_type = if param == WM_KEYDOWN as usize{
            EventType::KeyPress{code: 1}
        }else{
            EventType::KeyRelease{code: 1}
        };

    
        let event = Event{event_type, time:SystemTime::now(), name: None};
        GLOBAL_CALLBACK(event)
    } 


    CallNextHookEx(HOOK, code, param, lpdata)
}

pub fn listen(callback: Callback){
    unsafe{
        GLOBAL_CALLBACK = callback;
        let hook = SetWindowsHookExW(
            WH_KEYBOARD_LL,
           Some(raw_callback),
            null_mut(),
            0
    
        );
        if hook == null_mut() {
            let error =  GetLastError();
            panic!("Can't set system hook! {:?}", error)
        }
        HOOK = hook;
        println!("Hook {:?}", hook);
        
    GetMessageA(null_mut(), null_mut(), 0, 0);
    }
}

pub fn simulate(_event_type:  &EventType) -> Result<(), SimulateError>{
    Err(SimulateError)
}