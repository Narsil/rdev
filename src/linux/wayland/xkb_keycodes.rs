use std::os::raw::c_uint;

// Maps our internal keycodes to XKB keycodes
pub fn internal_to_xkb_keycode(internal_code: c_uint) -> u32 {
    // XKB keycodes start at 8, so we need to offset them
    internal_code + 8
}
