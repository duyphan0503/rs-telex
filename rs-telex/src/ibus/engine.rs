use crate::ibus::types::*;
use crate::telex::{TelexAction, TelexBuffer};
use zbus::object_server::SignalEmitter;
use zbus::{Result, interface};

pub struct IBusEngineService {
    pub buffer: TelexBuffer,
    pub caps: u32,
    pub enabled: bool,
    pub temporary_english: bool,
    pub undo_history: Option<(usize, String, String)>,
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
            undo_history: None,
        }
    }
}

#[interface(name = "org.freedesktop.IBus.Engine")]
impl IBusEngineService {
    /// Process incoming key events from the IBus daemon.
    ///
    /// Returns true if the key was consumed by the IME, or false if it should be
    /// forwarded to the active application.
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

        super::log::log_info(&format!(
            "process_key_event: keyval={:#x} ('{}'), keycode={}, state={:#x}, enabled={}",
            keyval,
            char::from_u32(keyval).unwrap_or('?'),
            _keycode,
            state,
            self.enabled
        ));

        // 2. Modifiers detection
        let has_ctrl = (state & IBUS_CONTROL_MASK) != 0;
        let has_shift = (state & IBUS_SHIFT_MASK) != 0;
        let has_alt = (state & IBUS_MOD1_MASK) != 0;
        let has_super = (state & IBUS_SUPER_MASK) != 0;

        // 3. Quick Mode Toggle between VI and EN:
        // - CapsLock key
        // - Ctrl + Shift
        // - Alt + Z
        if keyval == IBUS_KEY_CAPS_LOCK
            || (has_ctrl && (keyval == IBUS_KEY_SHIFT_L || keyval == IBUS_KEY_SHIFT_R))
            || (has_shift && (keyval == IBUS_KEY_CONTROL_L || keyval == IBUS_KEY_CONTROL_R))
            || (has_alt && (keyval == 'z' as u32 || keyval == 'Z' as u32))
        {
            self.enabled = !self.enabled;
            self.buffer.reset();
            super::log::log_info(&format!("Mode toggled: enabled={}", self.enabled));
            return false;
        }

        // 4. Escape key: temporary English mode or buffer reset
        if keyval == IBUS_KEY_ESCAPE {
            if !self.buffer.is_empty() {
                self.buffer.reset();
                self.temporary_english = false;
                return false;
            }
            self.temporary_english = !self.temporary_english;
            return false;
        }

        // 5. If disabled (EN mode) or temporary English, pass-through ALL keys immediately
        if !self.enabled || self.temporary_english {
            if !self.buffer.is_empty() {
                self.buffer.reset();
            }
            return false;
        }

        if has_alt || has_super {
            self.buffer.reset();
            return false;
        }

        // 6. Ctrl+Z undo word
        if has_ctrl {
            if (keyval == 'z' as u32 || keyval == 'Z' as u32)
                && let Some((backspaces, raw, _)) = self.undo_history.take()
            {
                let supports_surrounding = (self.caps & IBUS_CAP_SURROUNDING_TEXT) != 0;
                if supports_surrounding {
                    let _ = Self::delete_surrounding_text(
                        &emitter,
                        -(backspaces as i32),
                        backspaces as u32,
                    )
                    .await;
                    let text_val = make_ibus_text(&raw);
                    let _ = Self::commit_text(&emitter, text_val).await;
                } else {
                    for _ in 0..backspaces {
                        let _ = Self::forward_key_event(&emitter, IBUS_KEY_BACKSPACE, 14, 0).await;
                        let _ = Self::forward_key_event(
                            &emitter,
                            IBUS_KEY_BACKSPACE,
                            14,
                            IBUS_RELEASE_MASK,
                        )
                        .await;
                    }
                    let text_val = make_ibus_text(&raw);
                    let _ = Self::commit_text(&emitter, text_val).await;
                }
                self.buffer.reset();
                return true;
            }
            self.buffer.reset();
            return false;
        }

        // 7. Navigation keys and word boundaries
        match keyval {
            IBUS_KEY_BACKSPACE => {
                if !self.buffer.is_empty() {
                    self.buffer.pop_char();
                }
                return false;
            }
            IBUS_KEY_RETURN | IBUS_KEY_TAB | IBUS_KEY_LEFT | IBUS_KEY_RIGHT | IBUS_KEY_UP
            | IBUS_KEY_DOWN => {
                self.buffer.reset();
                self.temporary_english = false;
                return false;
            }
            IBUS_KEY_SPACE => {
                self.buffer.reset();
                self.temporary_english = false;
                return false;
            }
            _ => {}
        }

        // 8. Check printable character
        let ch = match char::from_u32(keyval) {
            Some(c) if !c.is_control() => c,
            _ => {
                self.buffer.reset();
                return false;
            }
        };

        // 8. Process character with TelexBuffer
        let raw_char = ch;
        let action = self.buffer.process_char(ch);
        super::log::log_info(&format!(
            "Telex action: {:?}, raw='{}', composed='{}', caps={:#x}",
            action, self.buffer.raw, self.buffer.composed, self.caps
        ));
        match action {
            TelexAction::PassThrough(_) => false,
            TelexAction::Replace {
                backspaces,
                new_text,
            } => {
                super::log::log_info(&format!(
                    "Executing Replace: backspaces={}, new_text='{}'",
                    backspaces, new_text
                ));

                // Always use standard synthetic Backspace forwarding for reliable atomic replacement across all Linux apps/browsers
                for _ in 0..backspaces {
                    let _ = Self::forward_key_event(&emitter, IBUS_KEY_BACKSPACE, 0, 0).await;
                    let _ =
                        Self::forward_key_event(&emitter, IBUS_KEY_BACKSPACE, 0, IBUS_RELEASE_MASK)
                            .await;
                }
                let text_val = make_ibus_text(&new_text);
                let res_com = Self::commit_text(&emitter, text_val).await;
                super::log::log_info(&format!("Replace commit result: {:?}", res_com));

                self.undo_history = Some((
                    new_text.chars().count(),
                    format!("{}{}", self.buffer.raw, raw_char),
                    new_text,
                ));
                true
            }
        }
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

    async fn focus_in(&mut self) {
        super::log::log_info("focus_in");
        self.buffer.reset();
    }

    async fn focus_in_id(&mut self, object_path: &str, client: &str) {
        super::log::log_info(&format!(
            "focus_in_id: object_path='{}', client='{}'",
            object_path, client
        ));
        self.buffer.reset();
    }

    async fn focus_out(&mut self) {
        super::log::log_info("focus_out");
        self.buffer.reset();
    }

    async fn focus_out_id(&mut self, object_path: &str) {
        super::log::log_info(&format!("focus_out_id: object_path='{}'", object_path));
        self.buffer.reset();
    }

    async fn reset(&mut self) {
        super::log::log_info("reset");
        self.buffer.reset();
    }

    async fn enable(&mut self) {
        super::log::log_info("enable");
        self.enabled = true;
        self.buffer.reset();
    }

    async fn disable(&mut self) {
        super::log::log_info("disable");
        self.enabled = false;
        self.buffer.reset();
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
