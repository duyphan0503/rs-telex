use crate::ibus::types::*;
use crate::telex::TelexBuffer;
use zbus::object_server::SignalEmitter;
use zbus::{Result, interface};

fn is_modifier_key(keyval: u32) -> bool {
    matches!(
        keyval,
        IBUS_KEY_SHIFT_L
            | IBUS_KEY_SHIFT_R
            | IBUS_KEY_CONTROL_L
            | IBUS_KEY_CONTROL_R
            | IBUS_KEY_ALT_L
            | IBUS_KEY_ALT_R
            | IBUS_KEY_SUPER_L
            | IBUS_KEY_SUPER_R
            | IBUS_KEY_CAPS_LOCK
            | 0xfe03 // ISO_Level3_Shift (AltGr)
            | 0xfe08 // ISO_Group_Shift
            | 0xffe1..=0xffee
    )
}

pub struct IBusEngineService {
    pub buffer: TelexBuffer,
    pub caps: u32,
    pub enabled: bool,
    pub temporary_english: bool,
}

impl Default for IBusEngineService {
    fn default() -> Self {
        Self::new()
    }
}

impl IBusEngineService {
    pub fn new() -> Self {
        Self {
            buffer: TelexBuffer::new(),
            caps: 0,
            enabled: true,
            temporary_english: false,
        }
    }

    /// Commit whatever is in the current buffer cleanly to the document and hide the preedit layer
    async fn commit_current_buffer(&mut self, emitter: &SignalEmitter<'_>) {
        if !self.buffer.is_empty() {
            let composed = self.buffer.composed.clone();
            let text_val = make_ibus_text(&composed);
            let _ = Self::commit_text(emitter, text_val).await;
            let _ = Self::hide_preedit_text(emitter).await;
            self.buffer.reset();
        }
    }
}

#[interface(name = "org.freedesktop.IBus.Engine")]
impl IBusEngineService {
    /// Process incoming key events from the IBus daemon.
    ///
    /// Zero-Underline In-Place Direct Rendering:
    /// - Renders in-place with plain text attributes (no underline, no popup box).
    /// - 100% identical look-and-feel to Unikey direct typing.
    /// - Instantaneous, flicker-free character replacement with zero race conditions.
    async fn process_key_event(
        &mut self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
        keyval: u32,
        _keycode: u32,
        state: u32,
    ) -> bool {
        // 1. Release events: pass-through
        if (state & IBUS_RELEASE_MASK) != 0 {
            return false;
        }

        // 2. Ignore bare modifier key events (Shift, Ctrl, Alt, CapsLock, Super...)
        if is_modifier_key(keyval) {
            return false;
        }

        super::log::log_info(&format!(
            "process_key_event: keyval={:#x}, state={:#x}, enabled={}",
            keyval, state, self.enabled
        ));

        // 3. Modifiers detection
        let has_ctrl = (state & IBUS_CONTROL_MASK) != 0;
        let has_shift = (state & IBUS_SHIFT_MASK) != 0;
        let has_alt = (state & IBUS_MOD1_MASK) != 0;
        let has_super = (state & IBUS_SUPER_MASK) != 0;

        // 4. Quick Mode Toggle between VI and EN:
        // - Ctrl + Shift
        // - Alt + Z
        if (has_ctrl && (keyval == IBUS_KEY_SHIFT_L || keyval == IBUS_KEY_SHIFT_R))
            || (has_shift && (keyval == IBUS_KEY_CONTROL_L || keyval == IBUS_KEY_CONTROL_R))
            || (has_alt && (keyval == 'z' as u32 || keyval == 'Z' as u32))
        {
            self.commit_current_buffer(&emitter).await;
            self.enabled = !self.enabled;
            super::log::log_info(&format!("Mode toggled: enabled={}", self.enabled));
            return false;
        }

        // 5. Escape key: cancel current composing word and reset
        if keyval == IBUS_KEY_ESCAPE {
            if !self.buffer.is_empty() {
                let _ = Self::hide_preedit_text(&emitter).await;
                self.buffer.reset();
                self.temporary_english = false;
                return true;
            }
            self.temporary_english = !self.temporary_english;
            return false;
        }

        // 6. If disabled (EN mode) or temporary English, pass-through ALL keys immediately
        if !self.enabled || self.temporary_english {
            self.commit_current_buffer(&emitter).await;
            return false;
        }

        // 7. System shortcuts (Ctrl+C, Ctrl+V, Alt+Tab, etc.)
        if has_ctrl || has_alt || has_super {
            self.commit_current_buffer(&emitter).await;
            return false;
        }

        // 8. Word delimiters and navigation keys
        if keyval == IBUS_KEY_SPACE {
            if !self.buffer.is_empty() {
                let mut text = self.buffer.composed.clone();
                text.push(' ');
                let text_val = make_ibus_text(&text);
                let _ = Self::commit_text(&emitter, text_val).await;
                let _ = Self::hide_preedit_text(&emitter).await;
                self.buffer.reset();
                self.temporary_english = false;
                return true;
            }
            return false;
        }

        if keyval == IBUS_KEY_RETURN || keyval == IBUS_KEY_TAB {
            if !self.buffer.is_empty() {
                self.commit_current_buffer(&emitter).await;
                self.temporary_english = false;
                return false; // let Return/Tab execute in target application
            }
            return false;
        }

        if keyval == IBUS_KEY_BACKSPACE {
            if !self.buffer.is_empty() {
                self.buffer.pop_char();
                if self.buffer.is_empty() {
                    let _ = Self::hide_preedit_text(&emitter).await;
                } else {
                    let composed = self.buffer.composed.clone();
                    let cursor_pos = composed.chars().count() as u32;
                    // Zero-underline plain text
                    let text_val = make_ibus_text(&composed);
                    let _ = Self::update_preedit_text(&emitter, text_val, cursor_pos, true).await;
                }
                return true;
            }
            return false;
        }

        if matches!(
            keyval,
            IBUS_KEY_LEFT
                | IBUS_KEY_RIGHT
                | IBUS_KEY_UP
                | IBUS_KEY_DOWN
                | IBUS_KEY_DELETE
                | 0xff50..=0xff6b // Home, End, PageUp, PageDown, Insert
                | 0xffbe..=0xffcb // F1..F12
        ) {
            self.commit_current_buffer(&emitter).await;
            self.temporary_english = false;
            return false;
        }

        // 9. Extract valid character
        let ch = if (0x20..=0x7e).contains(&keyval) {
            char::from_u32(keyval)
        } else if (0x01000000..=0x0110ffff).contains(&keyval) {
            char::from_u32(keyval - 0x01000000)
        } else {
            None
        };

        let ch = match ch {
            Some(c) if !c.is_control() => c,
            _ => {
                self.commit_current_buffer(&emitter).await;
                return false;
            }
        };

        // 10. Non-alphabetic characters (punctuation, symbols, digits)
        // Auto-commit current word and append the symbol/digit immediately
        if !ch.is_alphabetic() {
            if !self.buffer.is_empty() {
                let mut text = self.buffer.composed.clone();
                text.push(ch);
                let text_val = make_ibus_text(&text);
                let _ = Self::commit_text(&emitter, text_val).await;
                let _ = Self::hide_preedit_text(&emitter).await;
                self.buffer.reset();
                return true;
            }
            return false;
        }

        // 11. Vietnamese Telex processing via Zero-Underline In-Place rendering
        let _ = self.buffer.process_char(ch);
        let composed = self.buffer.composed.clone();
        let cursor_pos = composed.chars().count() as u32;
        // make_ibus_text has NO underline attribute — displays as natural plain text
        let text_val = make_ibus_text(&composed);
        let _ = Self::update_preedit_text(&emitter, text_val, cursor_pos, true).await;
        let _ = Self::show_preedit_text(&emitter).await;
        super::log::log_info(&format!(
            "In-place rendered: raw='{}', composed='{}'",
            self.buffer.raw, self.buffer.composed
        ));
        true
    }

    async fn set_capabilities(&mut self, caps: u32) {
        super::log::log_info(&format!("set_capabilities: {:#x}", caps));
        self.caps = caps;
    }

    async fn set_surrounding_text(
        &mut self,
        _text: zvariant::Value<'_>,
        cursor_pos: u32,
        anchor_pos: u32,
    ) {
        super::log::log_info(&format!(
            "set_surrounding_text: cursor_pos={}, anchor_pos={}",
            cursor_pos, anchor_pos
        ));
    }

    async fn focus_in(&mut self, #[zbus(signal_emitter)] emitter: SignalEmitter<'_>) {
        super::log::log_info("focus_in");
        self.commit_current_buffer(&emitter).await;
    }

    async fn focus_in_id(
        &mut self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
        object_path: &str,
        client: &str,
    ) {
        super::log::log_info(&format!(
            "focus_in_id: object_path='{}', client='{}'",
            object_path, client
        ));
        self.commit_current_buffer(&emitter).await;
    }

    async fn focus_out(&mut self, #[zbus(signal_emitter)] emitter: SignalEmitter<'_>) {
        super::log::log_info("focus_out");
        self.commit_current_buffer(&emitter).await;
    }

    async fn focus_out_id(
        &mut self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
        object_path: &str,
    ) {
        super::log::log_info(&format!("focus_out_id: object_path='{}'", object_path));
        self.commit_current_buffer(&emitter).await;
    }

    async fn reset(&mut self, #[zbus(signal_emitter)] emitter: SignalEmitter<'_>) {
        super::log::log_info("reset");
        self.commit_current_buffer(&emitter).await;
    }

    async fn enable(&mut self, #[zbus(signal_emitter)] emitter: SignalEmitter<'_>) {
        super::log::log_info("enable");
        self.enabled = true;
        self.commit_current_buffer(&emitter).await;
    }

    async fn disable(&mut self, #[zbus(signal_emitter)] emitter: SignalEmitter<'_>) {
        super::log::log_info("disable");
        self.enabled = false;
        self.commit_current_buffer(&emitter).await;
    }

    async fn set_cursor_location(&mut self, x: i32, y: i32, w: i32, h: i32) {
        super::log::log_info(&format!(
            "set_cursor_location: x={}, y={}, w={}, h={}",
            x, y, w, h
        ));
    }

    async fn property_activate(&mut self, prop_name: &str, prop_state: u32) {
        super::log::log_info(&format!(
            "property_activate: prop_name='{}', prop_state={}",
            prop_name, prop_state
        ));
    }

    // Signals emitted by the Engine to IBus InputContext
    #[zbus(signal)]
    async fn commit_text(emitter: &SignalEmitter<'_>, text: zvariant::Value<'_>) -> Result<()>;

    #[zbus(signal)]
    async fn update_preedit_text(
        emitter: &SignalEmitter<'_>,
        text: zvariant::Value<'_>,
        cursor_pos: u32,
        visible: bool,
    ) -> Result<()>;

    #[zbus(signal)]
    async fn show_preedit_text(emitter: &SignalEmitter<'_>) -> Result<()>;

    #[zbus(signal)]
    async fn hide_preedit_text(emitter: &SignalEmitter<'_>) -> Result<()>;

    #[zbus(signal)]
    async fn delete_surrounding_text(
        emitter: &SignalEmitter<'_>,
        offset_from_cursor: i32,
        nchars: u32,
    ) -> Result<()>;

    #[zbus(signal)]
    async fn forward_key_event(
        emitter: &SignalEmitter<'_>,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> Result<()>;
}
