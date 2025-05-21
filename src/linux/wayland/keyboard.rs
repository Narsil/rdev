use super::keycodes::code_from_key;
use super::xkb_keycodes::internal_to_xkb_keycode;
use crate::rdev::{EventType, Key, KeyboardState};
use xkbcommon::xkb;

#[allow(dead_code)]
// TODO this does not handle keyboard mapping change
// which is handled by the compositor and depend on window to window
pub struct Keyboard {
    context: xkb::Context,
    keymap: xkb::Keymap,
    state: xkb::State,
    shift: bool,
    caps_lock: bool,
    alt: bool,
    ctrl: bool,
    meta: bool,
    shift_idx: u32,
    caps_idx: u32,
    alt_idx: u32,
    ctrl_idx: u32,
    meta_idx: u32,
    current_layout: String,
    current_variant: String,
    current_model: String,
}

impl Keyboard {
    pub fn new() -> Result<Self, crate::rdev::SimulateError> {
        let context = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);

        // Get current keyboard layout information from environment
        let layout = std::env::var("XKB_DEFAULT_LAYOUT").unwrap_or_else(|_| "us".to_string());
        let variant = std::env::var("XKB_DEFAULT_VARIANT").unwrap_or_else(|_| "".to_string());
        let model = std::env::var("XKB_DEFAULT_MODEL").unwrap_or_else(|_| "pc104".to_string());
        let rules = std::env::var("XKB_DEFAULT_RULES").unwrap_or_else(|_| "evdev".to_string());

        let keymap = xkb::Keymap::new_from_names(
            &context,
            &rules,
            &model,
            &layout,
            &variant,
            Some("terminate:ctrl_alt_bksp".to_string()),
            xkb::KEYMAP_COMPILE_NO_FLAGS,
        )
        .ok_or(crate::rdev::SimulateError)?;

        let state = xkb::State::new(&keymap);
        // Lookup modifier indices
        let shift_idx = keymap.mod_get_index("Shift");
        let caps_idx = keymap.mod_get_index("Lock");
        let alt_idx = keymap.mod_get_index("Mod1");
        let ctrl_idx = keymap.mod_get_index("Control");
        let meta_idx = keymap.mod_get_index("Mod4");

        Ok(Self {
            context,
            keymap,
            state,
            shift: false,
            caps_lock: false,
            alt: false,
            ctrl: false,
            meta: false,
            shift_idx,
            caps_idx,
            alt_idx,
            ctrl_idx,
            meta_idx,
            current_layout: layout,
            current_variant: variant,
            current_model: model,
        })
    }

    fn update_modifiers(&mut self) {
        let mut depressed = 0;
        if self.shift {
            depressed |= 1 << self.shift_idx;
        }
        if self.caps_lock {
            depressed |= 1 << self.caps_idx;
        }
        if self.alt {
            depressed |= 1 << self.alt_idx;
        }
        if self.ctrl {
            depressed |= 1 << self.ctrl_idx;
        }
        if self.meta {
            depressed |= 1 << self.meta_idx;
        }
        self.state.update_mask(depressed, 0, 0, 0, 0, 0);
    }

    fn get_key_utf8(&self, keycode: u32) -> Option<String> {
        let keycode = xkb::Keycode::from(keycode);
        let keysym = self.state.key_get_one_sym(keycode);
        if keysym == xkb::keysyms::KEY_NoSymbol.into() {
            return None;
        }
        let utf8 = self.state.key_get_utf8(keycode);
        if utf8.is_empty() {
            return None;
        }
        Some(utf8)
    }
}

impl KeyboardState for Keyboard {
    fn add(&mut self, event_type: &EventType) -> Option<String> {
        match event_type {
            EventType::KeyPress(key) => match key {
                Key::ShiftLeft | Key::ShiftRight => {
                    self.shift = true;
                    self.update_modifiers();
                    None
                }
                Key::CapsLock => {
                    self.caps_lock = !self.caps_lock;
                    self.update_modifiers();
                    None
                }
                Key::Alt => {
                    self.alt = true;
                    self.update_modifiers();
                    None
                }
                Key::ControlLeft | Key::ControlRight => {
                    self.ctrl = true;
                    self.update_modifiers();
                    None
                }
                Key::MetaLeft | Key::MetaRight => {
                    self.meta = true;
                    self.update_modifiers();
                    None
                }
                key => {
                    let internal_code = code_from_key(*key)?;
                    let xkb_code = internal_to_xkb_keycode(internal_code);
                    self.get_key_utf8(xkb_code)
                }
            },
            EventType::KeyRelease(key) => match key {
                Key::ShiftLeft | Key::ShiftRight => {
                    self.shift = false;
                    self.update_modifiers();
                    None
                }
                Key::Alt => {
                    self.alt = false;
                    self.update_modifiers();
                    None
                }
                Key::ControlLeft | Key::ControlRight => {
                    self.ctrl = false;
                    self.update_modifiers();
                    None
                }
                Key::MetaLeft | Key::MetaRight => {
                    self.meta = false;
                    self.update_modifiers();
                    None
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn reset(&mut self) {
        self.shift = false;
        self.caps_lock = false;
        self.alt = false;
        self.ctrl = false;
        self.meta = false;
        self.update_modifiers();
    }
}
