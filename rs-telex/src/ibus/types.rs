use std::collections::HashMap;
use zvariant::{Structure, StructureBuilder, Value};

// Standard IBus Key Codes and Masks
pub const IBUS_KEY_BACKSPACE: u32 = 0xff08;
pub const IBUS_KEY_TAB: u32 = 0xff09;
pub const IBUS_KEY_RETURN: u32 = 0xff0d;
pub const IBUS_KEY_ESCAPE: u32 = 0xff1b;
pub const IBUS_KEY_DELETE: u32 = 0xffff;
pub const IBUS_KEY_SPACE: u32 = 0x0020;
pub const IBUS_KEY_CAPS_LOCK: u32 = 0xffe5;
pub const IBUS_KEY_SHIFT_L: u32 = 0xffe1;
pub const IBUS_KEY_SHIFT_R: u32 = 0xffe2;
pub const IBUS_KEY_CONTROL_L: u32 = 0xffe3;
pub const IBUS_KEY_CONTROL_R: u32 = 0xffe4;
pub const IBUS_KEY_ALT_L: u32 = 0xffe9;
pub const IBUS_KEY_ALT_R: u32 = 0xffea;
pub const IBUS_KEY_SUPER_L: u32 = 0xffeb;
pub const IBUS_KEY_SUPER_R: u32 = 0xffec;

pub const IBUS_KEY_LEFT: u32 = 0xff51;
pub const IBUS_KEY_UP: u32 = 0xff52;
pub const IBUS_KEY_RIGHT: u32 = 0xff53;
pub const IBUS_KEY_DOWN: u32 = 0xff54;

pub const IBUS_RELEASE_MASK: u32 = 1 << 30;
pub const IBUS_SHIFT_MASK: u32 = 1 << 0;
pub const IBUS_LOCK_MASK: u32 = 1 << 1;
pub const IBUS_CONTROL_MASK: u32 = 1 << 2;
pub const IBUS_MOD1_MASK: u32 = 1 << 3; // Alt
pub const IBUS_SUPER_MASK: u32 = 1 << 26;

// Client Capabilities
pub const IBUS_CAP_PREEDIT_TEXT: u32 = 1 << 0;
pub const IBUS_CAP_AUXILIARY_TEXT: u32 = 1 << 1;
pub const IBUS_CAP_LOOKUP_TABLE: u32 = 1 << 2;
pub const IBUS_CAP_FOCUS: u32 = 1 << 3;
pub const IBUS_CAP_PROPERTY: u32 = 1 << 4;
pub const IBUS_CAP_SURROUNDING_TEXT: u32 = 1 << 5;

// Bus and Interface names
pub const IBUS_SERVICE: &str = "org.freedesktop.IBus";
pub const IBUS_PATH: &str = "/org/freedesktop/IBus";
pub const IBUS_INTERFACE: &str = "org.freedesktop.IBus";
pub const IBUS_FACTORY_PATH: &str = "/org/freedesktop/IBus/Factory";
pub const IBUS_FACTORY_INTERFACE: &str = "org.freedesktop.IBus.Factory";
pub const IBUS_ENGINE_INTERFACE: &str = "org.freedesktop.IBus.Engine";

pub const ENGINE_NAME: &str = "rs-telex";
pub const COMPONENT_NAME: &str = "org.freedesktop.IBus.rs-telex";

/// Builds an `IBusText` D-Bus value for CommitText.
///
/// Signature: `(sa{sv}sv)` containing:
/// - "IBusText"
/// - attachments dict
/// - actual text string
/// - "IBusAttrList" struct
pub fn make_ibus_text(text: &str) -> Value<'static> {
    let empty_attachments: HashMap<String, Value<'static>> = HashMap::new();
    let empty_attrs: Vec<Value<'static>> = Vec::new();

    let attr_list = Structure::from((
        "IBusAttrList".to_string(),
        empty_attachments.clone(),
        empty_attrs,
    ));

    let text_struct = Structure::from((
        "IBusText".to_string(),
        empty_attachments,
        text.to_string(),
        Value::from(attr_list),
    ));

    Value::from(text_struct)
}

/// Builds an `IBusEngineDesc` D-Bus value.
///
/// Signature: `(sa{sv}ssssssssussssssss)`
pub fn make_engine_desc() -> Value<'static> {
    let empty_attachments: HashMap<String, Value<'static>> = HashMap::new();
    let desc = StructureBuilder::new()
        .append_field(Value::from("IBusEngineDesc".to_string()))
        .append_field(Value::from(empty_attachments))
        .append_field(Value::from(ENGINE_NAME.to_string()))
        .append_field(Value::from("Telex".to_string()))
        .append_field(Value::from(
            "Vietnamese Telex Input Method (Rust)".to_string(),
        ))
        .append_field(Value::from("vi".to_string()))
        .append_field(Value::from("MIT".to_string()))
        .append_field(Value::from("duyphan0503".to_string()))
        .append_field(Value::from("ibus-rs-telex".to_string()))
        .append_field(Value::from("us".to_string()))
        .append_field(Value::from(99u32))
        .append_field(Value::from(String::new()))
        .append_field(Value::from("vi".to_string()))
        .append_field(Value::from(String::new()))
        .append_field(Value::from(String::new()))
        .append_field(Value::from(String::new()))
        .append_field(Value::from("0.1.0".to_string()))
        .append_field(Value::from("rs-telex".to_string()))
        .append_field(Value::from(String::new()))
        .build()
        .unwrap();
    Value::from(desc)
}

/// Builds an `IBusComponent` D-Bus value for RegisterComponent.
pub fn make_component() -> Value<'static> {
    let empty_attachments: HashMap<String, Value<'static>> = HashMap::new();
    let engine = make_engine_desc();
    let comp = Structure::from((
        "IBusComponent".to_string(),
        empty_attachments,
        COMPONENT_NAME.to_string(),
        "Vietnamese Telex Input Method Component".to_string(),
        "0.1.0".to_string(),
        "MIT".to_string(),
        "duyphan0503".to_string(),
        "https://github.com/duyphan0503/rs-telex".to_string(),
        "/usr/libexec/ibus-engine-rs-telex --ibus".to_string(),
        "rs-telex".to_string(),
        Vec::<Value<'static>>::new(),
        vec![engine],
    ));
    Value::from(comp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_ibus_text_signature() {
        let val = make_ibus_text("test");
        let sig = val.value_signature();
        let sig_str = format!("{}", sig);
        println!("make_ibus_text inner signature: {}", sig_str);
        assert_eq!(sig_str, "(sa{sv}sv)");
    }
}
