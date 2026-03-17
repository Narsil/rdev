use crate::rdev::{EventType, Key, KeyboardState};
use crate::windows::common::{get_code, get_scan_code};
use crate::windows::keycodes::code_from_key;
use std::ptr::null_mut;
use winapi::shared::minwindef::{BYTE, HKL, LPARAM, UINT};
use winapi::um::winuser::{
    GetAsyncKeyState, GetForegroundWindow, GetKeyState, GetKeyboardLayout,
    GetWindowThreadProcessId, ToUnicodeEx, VK_CAPITAL, VK_CONTROL, VK_LCONTROL, VK_LMENU,
    VK_LSHIFT, VK_MENU, VK_RCONTROL, VK_RMENU, VK_RSHIFT, VK_SHIFT,
};

// Windows 10, version 1607+:
// keep ToUnicodeEx from mutating the kernel keyboard buffer while we are only observing input.
const TO_UNICODE_NO_STATE_CHANGE: UINT = 0x0004;
const VK_SHIFT_: usize = VK_SHIFT as usize;
const VK_CAPITAL_: usize = VK_CAPITAL as usize;
const VK_CONTROL_: usize = VK_CONTROL as usize;
const VK_MENU_: usize = VK_MENU as usize;
const VK_LCONTROL_: usize = VK_LCONTROL as usize;
const VK_RCONTROL_: usize = VK_RCONTROL as usize;
const VK_LMENU_: usize = VK_LMENU as usize;
const VK_RMENU_: usize = VK_RMENU as usize;
const VK_LSHIFT_: usize = VK_LSHIFT as usize;
const VK_RSHIFT_: usize = VK_RSHIFT as usize;
const HIGHBIT: u8 = 0x80;

pub struct Keyboard {
    last_code: UINT,
    last_scan_code: UINT,
    last_state: [BYTE; 256],
    last_is_dead: bool,
}

impl Keyboard {
    pub fn new() -> Option<Keyboard> {
        Some(Keyboard {
            last_code: 0,
            last_scan_code: 0,
            last_state: [0; 256],
            last_is_dead: false,
        })
    }

    pub(crate) unsafe fn get_name(&mut self, lpdata: LPARAM) -> Option<String> {
        unsafe {
            // https://gist.github.com/akimsko/2011327
            // https://www.experts-exchange.com/questions/23453780/LowLevel-Keystroke-Hook-removes-Accents-on-French-Keyboard.html
            let code = get_code(lpdata);
            let scan_code = get_scan_code(lpdata);

            if !key_can_produce_text(code) {
                return None;
            }

            self.set_global_state()?;
            self.apply_hook_state(code);
            self.get_code_name(code, scan_code)
        }
    }

    pub(crate) unsafe fn set_global_state(&mut self) -> Option<()> {
        let mut state = [0_u8; 256];
        state[VK_SHIFT_] = pressed_state(VK_SHIFT as i32);
        state[VK_LSHIFT_] = pressed_state(VK_LSHIFT as i32);
        state[VK_RSHIFT_] = pressed_state(VK_RSHIFT as i32);
        state[VK_CONTROL_] = pressed_state(VK_CONTROL as i32);
        state[VK_LCONTROL_] = pressed_state(VK_LCONTROL as i32);
        state[VK_RCONTROL_] = pressed_state(VK_RCONTROL as i32);
        state[VK_MENU_] = pressed_state(VK_MENU as i32);
        state[VK_LMENU_] = pressed_state(VK_LMENU as i32);
        state[VK_RMENU_] = pressed_state(VK_RMENU as i32);
        state[VK_CAPITAL_] = toggled_state(VK_CAPITAL as i32);

        self.last_state = state;
        Some(())
    }

    fn apply_hook_state(&mut self, code: UINT) {
        if let Some(state) = self.last_state.get_mut(code as usize) {
            *state |= HIGHBIT;
        }

        match code as usize {
            VK_LSHIFT_ => self.last_state[VK_SHIFT_] |= HIGHBIT,
            VK_RSHIFT_ => self.last_state[VK_SHIFT_] |= HIGHBIT,
            VK_LCONTROL_ => self.last_state[VK_CONTROL_] |= HIGHBIT,
            VK_RCONTROL_ => self.last_state[VK_CONTROL_] |= HIGHBIT,
            VK_LMENU_ => self.last_state[VK_MENU_] |= HIGHBIT,
            VK_RMENU_ => self.last_state[VK_MENU_] |= HIGHBIT,
            VK_CAPITAL_ => self.last_state[VK_CAPITAL_] ^= 1,
            _ => {}
        }
    }

    pub(crate) unsafe fn get_code_name(&mut self, code: UINT, scan_code: UINT) -> Option<String> {
        unsafe {
            let current_window_thread_id =
                GetWindowThreadProcessId(GetForegroundWindow(), null_mut());
            let state_ptr = self.last_state.as_mut_ptr();
            const BUF_LEN: i32 = 32;
            let mut buff = [0_u16; BUF_LEN as usize];
            let buff_ptr = buff.as_mut_ptr();
            let layout = GetKeyboardLayout(current_window_thread_id);
            let len = ToUnicodeEx(
                code,
                scan_code,
                state_ptr,
                buff_ptr,
                BUF_LEN,
                TO_UNICODE_NO_STATE_CHANGE,
                layout,
            );

            let mut is_dead = false;
            let result = match len {
                0 => None,
                -1 => {
                    is_dead = true;
                    self.clear_keyboard_buffer(code, scan_code, layout);
                    None
                }
                len if len > 0 => String::from_utf16(&buff[..len as usize]).ok(),
                _ => None,
            };

            if self.last_code != 0 && self.last_is_dead {
                buff = [0; 32];
                let buff_ptr = buff.as_mut_ptr();
                let last_state_ptr = self.last_state.as_mut_ptr();
                ToUnicodeEx(
                    self.last_code,
                    self.last_scan_code,
                    last_state_ptr,
                    buff_ptr,
                    BUF_LEN,
                    TO_UNICODE_NO_STATE_CHANGE,
                    layout,
                );
                self.last_code = 0;
            } else {
                self.last_code = code;
                self.last_scan_code = scan_code;
                self.last_is_dead = is_dead;
            }
            result
        }
    }

    unsafe fn clear_keyboard_buffer(&self, code: UINT, scan_code: UINT, layout: HKL) {
        unsafe {
            const BUF_LEN: i32 = 32;
            let mut buff = [0_u16; BUF_LEN as usize];
            let buff_ptr = buff.as_mut_ptr();
            let mut state = [0_u8; 256];
            let state_ptr = state.as_mut_ptr();

            let mut len = -1;
            while len < 0 {
                len = ToUnicodeEx(
                    code,
                    scan_code,
                    state_ptr,
                    buff_ptr,
                    BUF_LEN,
                    TO_UNICODE_NO_STATE_CHANGE,
                    layout,
                );
            }
        }
    }
}

fn pressed_state(vk: i32) -> BYTE {
    if unsafe { GetAsyncKeyState(vk) } < 0 {
        HIGHBIT
    } else {
        0
    }
}

fn toggled_state(vk: i32) -> BYTE {
    (unsafe { GetKeyState(vk) } & 1) as BYTE
}

fn key_can_produce_text(code: UINT) -> bool {
    matches!(
        code,
        0x20
            | 0x30..=0x39
            | 0x41..=0x5A
            | 0x60..=0x6F
            | 0xBA..=0xC0
            | 0xDB..=0xE2
    )
}

impl KeyboardState for Keyboard {
    fn add(&mut self, event_type: &EventType) -> Option<String> {
        match event_type {
            EventType::KeyPress(key) => match key {
                Key::ShiftLeft => {
                    self.last_state[VK_SHIFT_] |= HIGHBIT;
                    self.last_state[VK_LSHIFT_] |= HIGHBIT;
                    None
                }
                Key::ShiftRight => {
                    self.last_state[VK_SHIFT_] |= HIGHBIT;
                    self.last_state[VK_RSHIFT_] |= HIGHBIT;
                    None
                }
                Key::CapsLock => {
                    self.last_state[VK_CAPITAL_] ^= 1;
                    None
                }
                key => {
                    let code = code_from_key(*key)?;
                    unsafe { self.get_code_name(code.into(), 0) }
                }
            },
            EventType::KeyRelease(key) => match key {
                Key::ShiftLeft => {
                    self.last_state[VK_SHIFT_] &= !HIGHBIT;
                    self.last_state[VK_LSHIFT_] &= !HIGHBIT;
                    None
                }
                Key::ShiftRight => {
                    self.last_state[VK_SHIFT_] &= !HIGHBIT;
                    self.last_state[VK_RSHIFT_] &= HIGHBIT;
                    None
                }
                _ => None,
            },

            _ => None,
        }
    }

    fn reset(&mut self) {
        self.last_state[16] = 0;
        self.last_state[20] = 0;
    }
}
