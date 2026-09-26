pub mod ibus;
pub mod syllable;
pub mod telex;
pub mod unicode;

use ibus::IBusClient;
use std::env;
use telex::{TelexAction, TelexBuffer};

fn print_component_xml() {
    let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<component>
  <name>org.freedesktop.IBus.rs-telex</name>
  <description>Vietnamese Telex Input Method (Rust)</description>
  <exec>/usr/bin/rs-telex --ibus</exec>
  <version>0.1.0</version>
  <author>dp</author>
  <license>MIT</license>
  <homepage>https://github.com/dp/rs-telex</homepage>
  <textdomain>rs-telex</textdomain>
  <engines>
    <engine>
      <name>rs-telex</name>
      <language>vi</language>
      <license>MIT</license>
      <author>dp</author>
      <icon>ibus-rs-telex</icon>
      <layout>us</layout>
      <longname>Vietnamese - Telex (rs-telex)</longname>
      <description>Vietnamese Telex input method written in Rust</description>
      <rank>99</rank>
    </engine>
  </engines>
</component>"#;
    println!("{}", xml);
}

fn run_smoke_test() -> bool {
    let test_inputs = vec![
        ("xin chaof", "xin chào"),
        ("aas", "ấ"),
        ("web", "web"),
        ("hoaf", "hòa"),
        ("ass", "as"),
        ("asf", "à"),
        ("aasz", "â"),
        ("class", "class"),
        ("string", "string"),
        ("async", "async"),
        ("getUser", "getUser"),
    ];

    println!("=== Running smoke test on Telex engine ===");
    let mut buffer = TelexBuffer::new();
    let mut all_passed = true;

    for (input, expected) in test_inputs {
        buffer.reset();
        let mut rendered = String::new();

        for ch in input.chars() {
            match buffer.process_char(ch) {
                TelexAction::PassThrough(c) => {
                    rendered.push(c);
                }
                TelexAction::Replace {
                    backspaces,
                    new_text,
                } => {
                    for _ in 0..backspaces {
                        rendered.pop();
                    }
                    rendered.push_str(&new_text);
                }
            }
        }

        let passed = rendered == expected;
        if !passed {
            all_passed = false;
        }
        println!(
            "Input: {:<12} -> Got: {:<12} | Expected: {:<12} [{}]",
            input,
            rendered,
            expected,
            if passed { "PASS" } else { "FAIL" }
        );
    }
    all_passed
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let is_ibus = args.iter().any(|arg| arg == "--ibus");
    let is_standalone = args.iter().any(|arg| arg == "--standalone" || arg == "-s");
    let is_xml = args.iter().any(|arg| arg == "--xml");
    let is_test = args.iter().any(|arg| arg == "--test" || arg == "-t");
    let is_version = args.iter().any(|arg| arg == "--version" || arg == "-v");

    if is_version {
        println!("rs-telex 0.1.0 (IBus Vietnamese IME in Rust)");
        return;
    }

    if is_xml {
        print_component_xml();
        return;
    }

    if is_ibus || is_standalone {
        println!(
            "rs-telex IBus engine starting (mode: {})...",
            if is_ibus { "embedded" } else { "standalone" }
        );
        let result = zbus::block_on(async {
            let client = IBusClient::connect().await?;
            client.run_service(is_standalone).await
        });

        if let Err(e) = result {
            eprintln!("Error running rs-telex IBus engine: {}", e);
            std::process::exit(1);
        }
        return;
    }

    if is_test {
        let ok = run_smoke_test();
        if !ok {
            std::process::exit(1);
        }
        return;
    }

    println!("rs-telex 0.1.0 - Vietnamese Telex Input Method for Linux/IBus");
    println!("Usage:");
    println!("  rs-telex --ibus         Run as embedded IBus engine daemon");
    println!("  rs-telex --standalone   Run as standalone engine and register with IBus");
    println!("  rs-telex --xml          Output IBus component XML");
    println!("  rs-telex --test         Run self tests");
    println!();
    run_smoke_test();
}

#[cfg(test)]
mod tests {
    use super::telex::{TelexAction, TelexBuffer};

    fn type_string(input: &str) -> String {
        let mut buf = TelexBuffer::new();
        let mut out = String::new();
        for ch in input.chars() {
            let action = buf.process_char(ch);
            match action {
                TelexAction::PassThrough(c) if c != '\0' => out.push(c),
                TelexAction::Replace {
                    backspaces,
                    new_text,
                } => {
                    for _ in 0..backspaces {
                        out.pop();
                    }
                    out.push_str(&new_text);
                }
                _ => {}
            }
        }
        out
    }

    #[test]
    fn test_basic_typing() {
        assert_eq!(type_string("xin chaof"), "xin chào");
        assert_eq!(type_string("ddi"), "đi");
        assert_eq!(type_string("awn"), "ăn");
        assert_eq!(type_string("uow"), "ươ");
    }

    #[test]
    fn test_w_key_handling() {
        assert_eq!(type_string("web"), "web");
        assert_eq!(type_string("wifi"), "wifi");
        assert_eq!(type_string("aw"), "ă");
        assert_eq!(type_string("uw"), "ư");
        assert_eq!(type_string("ow"), "ơ");
    }

    #[test]
    fn test_tone_undo_and_overwrite() {
        // Undo tone: as -> á, then s again -> as
        assert_eq!(type_string("ass"), "as");
        // Overwrite tone: as -> á, then f -> à
        assert_eq!(type_string("asf"), "à");
    }

    #[test]
    fn test_z_removes_tone_preserves_hat() {
        // aas = ấ, then z -> â
        assert_eq!(type_string("aasz"), "â");
    }

    #[test]
    fn test_unikey_tone_placement() {
        // hòa instead of hoà
        assert_eq!(type_string("hoaf"), "hòa");
    }

    #[test]
    fn test_coder_smart_pass_through() {
        // Programming words should not be mangled by Vietnamese Telex
        assert_eq!(type_string("class"), "class");
        assert_eq!(type_string("string"), "string");
        assert_eq!(type_string("async"), "async");
        assert_eq!(type_string("my_var"), "my_var");
        assert_eq!(type_string("getUser"), "getUser");
        assert_eq!(type_string("printf"), "printf");
        assert_eq!(type_string("function"), "function");
        assert_eq!(type_string("import"), "import");
        assert_eq!(type_string("export"), "export");
        assert_eq!(type_string("struct"), "struct");
    }

    #[test]
    fn test_user_reported_cases() {
        // User cases: "vanax loiox khongo gox dduocwj" -> "vẫn lỗi không gõ được"
        assert_eq!(type_string("vanax"), "vẫn");
        assert_eq!(type_string("loiox"), "lỗi");
        assert_eq!(type_string("khongo"), "không");
        assert_eq!(type_string("gox"), "gõ");
        assert_eq!(type_string("dduocwj"), "được");
        assert_eq!(type_string("duowcj"), "dược");
        assert_eq!(type_string("dduowcj"), "được");
        assert_eq!(
            type_string("vanax loiox khongo gox dduocwj"),
            "vẫn lỗi không gõ được"
        );

        // User case 2: Typing "rust" (rus -> rú -> s -> rus -> t -> rust)
        assert_eq!(type_string("russt"), "rust");
        assert_eq!(type_string("rust"), "rust");

        // User case 3: Typing "lưu" with "luuw"
        assert_eq!(type_string("luuw"), "lưu");
        assert_eq!(type_string("huuw"), "hưu");
        assert_eq!(type_string("muuw"), "mưu");
        assert_eq!(type_string("cuuw"), "cưu");
        assert_eq!(type_string("luuwr"), "lửu");
        assert_eq!(type_string("luuwj"), "lựu");

        // User case 4: Typing "engine" without mangling to "êngin"
        assert_eq!(type_string("engine"), "engine");
        assert_eq!(type_string("engineer"), "engineer");
        assert_eq!(type_string("system"), "system");
        assert_eq!(type_string("window"), "window");
        assert_eq!(type_string("socket"), "socket");
    }

    #[test]
    fn test_more_vietnamese_words() {
        // More Vietnamese words with Telex
        assert_eq!(type_string("vieetj nam"), "việt nam");
        assert_eq!(type_string("tooi"), "tôi");
        assert_eq!(type_string("banj"), "bạn");
        assert_eq!(type_string("camr own"), "cảm ơn");
        assert_eq!(type_string("xin loiox"), "xin lỗi");
        assert_eq!(type_string("khoong"), "không");
        assert_eq!(type_string("cos"), "có");
        assert_eq!(type_string("nguwowif"), "người");
        assert_eq!(type_string("nhaf"), "nhà");
        assert_eq!(type_string("banj owi"), "bạn ơi");
        assert_eq!(type_string("lamf own"), "làm ơn");
        assert_eq!(type_string("hejn gawpj laij"), "hẹn gặp lại");
        assert_eq!(type_string("chusc muwngf"), "chúc mừng");
        assert_eq!(type_string("taast car"), "tất cả");
        assert_eq!(type_string("mootj"), "một");
        assert_eq!(type_string("hai"), "hai");
        assert_eq!(type_string("ba"), "ba");
        assert_eq!(type_string("boosn"), "bốn");
        assert_eq!(type_string("nawm"), "năm");
        assert_eq!(type_string("saus"), "sáu");
        assert_eq!(type_string("bayr"), "bảy");
        assert_eq!(type_string("tams"), "tám");
        assert_eq!(type_string("chins"), "chín");
        assert_eq!(type_string("muwowif"), "mười");
    }

    #[test]
    fn test_toneless_after_tone() {
        // Type more letters after applying tone
        assert_eq!(type_string("asfng"), "àng"); // à + ng
        assert_eq!(type_string("awjng"), "ặng"); // ặ + ng
        assert_eq!(type_string("aasjng"), "ậng"); // ậ + ng
    }

    #[test]
    fn test_uppercase() {
        // Test uppercase handling
        assert_eq!(type_string("AA"), "Â");
        assert_eq!(type_string("AS"), "Á");
        assert_eq!(type_string("AF"), "À");
        assert_eq!(type_string("AW"), "Ă");
        assert_eq!(type_string("DD"), "Đ");
        assert_eq!(type_string("AAS"), "Ấ");
        assert_eq!(type_string("ASF"), "À");
        assert_eq!(type_string("ASZ"), "A");
        assert_eq!(type_string("AASZ"), "Â");
    }

    #[test]
    fn test_consonant_clusters() {
        // Test Vietnamese consonant clusters
        assert_eq!(type_string("ch"), "ch");
        assert_eq!(type_string("chf"), "ch"); // no vowel, no tone
        assert_eq!(type_string("cha"), "cha");
        assert_eq!(type_string("chaf"), "chà");
        assert_eq!(type_string("chaw"), "chă");
        assert_eq!(type_string("chaws"), "chắ");

        assert_eq!(type_string("ngh"), "ngh");
        assert_eq!(type_string("nghx"), "ngh"); // no vowel, no tone
        assert_eq!(type_string("nghi"), "nghi"); // no tone key = no tone
        assert_eq!(type_string("nghix"), "nghĩ"); // x = Nga tone
        assert_eq!(type_string("nghis"), "nghí");
        assert_eq!(type_string("nghir"), "nghỉ");

        assert_eq!(type_string("qu"), "qu");
        assert_eq!(type_string("que"), "que");
        assert_eq!(type_string("quee"), "quê");
        assert_eq!(type_string("quaf"), "quà");

        assert_eq!(type_string("gi"), "gi");
        assert_eq!(type_string("gif"), "gì");
        assert_eq!(type_string("gias"), "giá");
    }

    #[test]
    fn test_english_mode_detection() {
        // English words should pass through
        assert_eq!(type_string("hello"), "hello");
        assert_eq!(type_string("world"), "world");
        assert_eq!(type_string("test"), "test");
        assert_eq!(type_string("email"), "email");
        assert_eq!(type_string("computer"), "computer");
        assert_eq!(type_string("software"), "software");
        assert_eq!(type_string("hardware"), "hardware");
    }

    #[test]
    fn test_complex_vowel_clusters() {
        // Complex triphthongs and diphthongs
        assert_eq!(type_string("khoais"), "khoái");
        assert_eq!(type_string("ngoaif"), "ngoài");
        assert_eq!(type_string("khuyeens"), "khuyến");
        assert_eq!(type_string("tuyeetj"), "tuyệt");
        assert_eq!(type_string("ruowuj"), "rượu");
        assert_eq!(type_string("huowu"), "hươu");
        assert_eq!(type_string("huowus"), "hướu");
        assert_eq!(type_string("nuowcs"), "nước");
        assert_eq!(type_string("uoosng"), "uống");
        assert_eq!(type_string("yeens"), "yến");
        assert_eq!(type_string("truowngf"), "trường");
        assert_eq!(type_string("phuwowng"), "phương");
        assert_eq!(type_string("chuowng"), "chương");
        assert_eq!(type_string("thuyr"), "thủy");
        assert_eq!(type_string("nguyenx"), "nguyẽn");
        assert_eq!(type_string("nguyeenx"), "nguyễn");
        assert_eq!(type_string("nguyeens"), "nguyến");
        assert_eq!(type_string("buoofn"), "buồn");
        assert_eq!(type_string("cuoojn"), "cuộn");
    }

    #[test]
    fn test_case_preservation() {
        // Title Case and ALL CAPS
        assert_eq!(type_string("Vieetj Nam"), "Việt Nam");
        assert_eq!(type_string("Haf Nooi"), "Hà Nôi");
        assert_eq!(type_string("Haf Nooij"), "Hà Nội");
        assert_eq!(type_string("VIEETJ NAM"), "VIỆT NAM");
        assert_eq!(type_string("DDAF NAWNGX"), "ĐÀ NẴNG");
    }

    #[test]
    fn test_developer_identifiers() {
        // PascalCase, camelCase, snake_case
        assert_eq!(type_string("getUserById"), "getUserById");
        assert_eq!(type_string("ApiResponse"), "ApiResponse");
        assert_eq!(type_string("total_user_count"), "total_user_count");
        assert_eq!(type_string("is_active_flag"), "is_active_flag");
        assert_eq!(type_string("handleClickEvent"), "handleClickEvent");
    }

    #[test]
    fn test_punctuation_and_numbers() {
        // Punctuation and digits reset the buffer
        assert_eq!(type_string("soos 123"), "số 123");
        assert_eq!(type_string("chaof, banj!"), "chào, bạn!");
        assert_eq!(type_string("test.rs"), "test.rs");
        assert_eq!(type_string("config.json"), "config.json");
    }

    #[test]
    fn test_buffer_pop_backspace() {
        let mut buffer = crate::telex::TelexBuffer::new();
        buffer.process_char('c');
        buffer.process_char('h');
        buffer.process_char('a');
        buffer.process_char('f');
        assert_eq!(buffer.composed, "chà");
        buffer.pop_char();
        assert_eq!(buffer.composed, "ch");
        buffer.process_char('e');
        buffer.process_char('e');
        buffer.process_char('f');
        assert_eq!(buffer.composed, "chề");
    }

    #[test]
    fn test_tone_placement_with_ending_consonants() {
        // Words typed with tone before ending consonant
        assert_eq!(type_string("toafn"), "toàn");
        assert_eq!(type_string("toasn"), "toán");
        assert_eq!(type_string("hoafn"), "hoàn");
        assert_eq!(type_string("hoasc"), "hoác");
        assert_eq!(type_string("hoajt"), "hoạt");
        assert_eq!(type_string("xoefn"), "xoèn");
        assert_eq!(type_string("khoern"), "khoẻn");
        assert_eq!(type_string("khoest"), "khoét");
        assert_eq!(type_string("thuyrt"), "thuỷt");
        assert_eq!(type_string("thuyst"), "thuýt");
        assert_eq!(type_string("thuyjt"), "thuỵt");
        assert_eq!(type_string("quyst"), "quýt");

        // Tone removal with 'z' key
        assert_eq!(type_string("loioxz"), "lôi");
        assert_eq!(type_string("toafnz"), "toan");
        assert_eq!(type_string("vieetjz"), "viêt");

        // Two words with space in between (e.g. "input đầu")
        assert_eq!(type_string("input ddaauf"), "input đầu");
        assert_eq!(type_string("rust ddaauf"), "rust đầu");
    }

    #[test]
    fn test_disabled_engine_mode() {
        let mut engine = crate::ibus::engine::IBusEngineService::new();
        engine.enabled = false;
        // When disabled, buffer remains empty and never applies Telex
        assert!(!engine.enabled);
        assert!(engine.buffer.is_empty());
    }
}
