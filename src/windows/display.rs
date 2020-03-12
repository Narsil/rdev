use winapi::um::winuser::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

pub fn display_size() -> (u64, u64) {
    let w = unsafe { GetSystemMetrics(SM_CXSCREEN) as u64 };
    let h = unsafe { GetSystemMetrics(SM_CYSCREEN) as u64 };
    (w, h)
}
