extern crate winapi;

use crate::rdev::{Event, EventType, SimulateError, Callback};
use winapi::um::winuser::{WH_KEYBOARD_LL,SetWindowsHookExW, CallNextHookEx};
use winapi::um::errhandlingapi::{GetLastError};
use std::ptr::{null_mut};
use std::{thread, time};


fn default_callback(event: Event) {
    println!("Default : Event {:?}", event);
}

static mut GLOBAL_CALLBACK: Callback = default_callback;

unsafe extern "system" fn raw_callback(code: i32, param: usize, lpdata: isize) -> isize{
    println!("YOUHOU {:?} {:?} {:?}" ,code, param, lpdata );
    CallNextHookEx(null_mut(), code, param, lpdata)
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
        println!("Hook {:?}", hook);
    }
    let ten_secs = time::Duration::from_secs(10);
    thread::sleep(ten_secs);


}

pub fn simulate(_event_type:  &EventType) -> Result<(), SimulateError>{
    Err(SimulateError)
}