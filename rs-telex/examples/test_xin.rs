use rs_telex::telex::{TelexAction, TelexBuffer};

fn type_string(input: &str) -> String {
    let mut buf = TelexBuffer::new();
    let mut out = String::new();
    for ch in input.chars() {
        println!("Processing: {:?}, composed: {:?}", ch, buf.composed);
        let action = buf.process_char(ch);
        println!("  Action: {:?}", action);
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
        println!("  Out: {:?}", out);
    }
    out
}

fn main() {
    println!("Testing 'xin chaof':");
    let result = type_string("xin chaof");
    println!("Result: {:?}", result);
    println!("Expected: \"xin chào\"");
}
