#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

#[cfg(feature = "bindgen")]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Standard keysym and mask constants used by IBus and X11
pub const IBUS_KEY_BackSpace: u32 = 0xff08;
pub const IBUS_RELEASE_MASK: u32 = 1 << 30;
pub const IBUS_SHIFT_MASK: u32 = 1 << 0;
pub const IBUS_CONTROL_MASK: u32 = 1 << 2;
pub const IBUS_MOD1_MASK: u32 = 1 << 3; // Alt
pub const IBUS_SUPER_MASK: u32 = 1 << 26;

pub const IBUS_CAP_PREEDIT_TEXT: u32 = 1 << 0;
pub const IBUS_CAP_AUXILIARY_TEXT: u32 = 1 << 1;
pub const IBUS_CAP_LOOKUP_TABLE: u32 = 1 << 2;
pub const IBUS_CAP_FOCUS: u32 = 1 << 3;
pub const IBUS_CAP_PROPERTY: u32 = 1 << 4;
pub const IBUS_CAP_SURROUNDING_TEXT: u32 = 1 << 5;
