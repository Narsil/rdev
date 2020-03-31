use std::convert::TryInto;
use winapi::um::winuser::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

pub fn display_size() -> (u64, u64) {
    let w = unsafe { GetSystemMetrics(SM_CXSCREEN).try_into().unwrap() };
    let h = unsafe { GetSystemMetrics(SM_CYSCREEN).try_into().unwrap() };
    (w, h)
}
