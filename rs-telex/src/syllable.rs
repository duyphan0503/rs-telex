/// Checks if a cluster of initial consonants is valid in Vietnamese.
/// Valid initial consonant clusters:
/// b, c, ch, d, đ, g, gh, gi, h, k, kh, l, m, n, ng, ngh, nh, p, ph, qu, r, s, t, th, tr, v, x
pub fn is_valid_vietnamese_initial_cluster(prefix: &str) -> bool {
    let lower = prefix.to_ascii_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    if chars.is_empty() {
        return true;
    }

    // Special allowance during typing: "dd" will become "đ"
    if lower == "dd" {
        return true;
    }

    let first = chars[0];
    if !matches!(
        first,
        'b' | 'c'
            | 'd'
            | 'đ'
            | 'g'
            | 'h'
            | 'k'
            | 'l'
            | 'm'
            | 'n'
            | 'p'
            | 'q'
            | 'r'
            | 's'
            | 't'
            | 'v'
            | 'x'
    ) {
        return false;
    }

    if chars.len() == 1 {
        return true;
    }

    let cluster: String = chars.iter().collect();

    if chars.len() == 2 {
        return matches!(
            cluster.as_str(),
            "ch" | "gh" | "gi" | "kh" | "ng" | "nh" | "ph" | "qu" | "th" | "tr"
        );
    }

    if chars.len() == 3 {
        return cluster.as_str() == "ngh";
    }

    false
}

/// Extracts the contiguous vowel cluster from a word (e.g. "loi" -> "oi", "khuyen" -> "uye", "nguoi" -> "uoi", "lio" -> "io")
pub fn extract_vowel_cluster(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let lower: String = word.to_ascii_lowercase();

    // Strip initial 'qu' or 'gi' semivowels
    let start_idx = if (lower.starts_with("qu") && chars.len() >= 2)
        || (lower.starts_with("gi")
            && chars.len() > 2
            && chars[1] == 'i'
            && crate::unicode::is_vietnamese_vowel(chars[2]))
    {
        2
    } else {
        0
    };

    let mut vowels = String::new();
    let mut in_vowel = false;
    for &c in &chars[start_idx..] {
        if crate::unicode::is_vietnamese_vowel(c) {
            in_vowel = true;
            vowels.push(c);
        } else if in_vowel {
            break;
        }
    }
    vowels
}

/// Checks if a vowel cluster is a valid Vietnamese diphthong or triphthong.
pub fn is_valid_vietnamese_vowel_cluster(vowels: &str) -> bool {
    let lower: String = vowels
        .chars()
        .flat_map(|c| crate::unicode::remove_tone(c).to_lowercase())
        .collect();

    if lower.chars().count() <= 1 {
        return true;
    }

    matches!(
        lower.as_str(),
        // 2 vowels
        "ai" | "ao" | "au" | "ay" | "âu" | "ây"
            | "eo" | "êu"
            | "ia" | "iê" | "ie" | "iu"
            | "oa" | "oă" | "oe" | "oi" | "ôi" | "ơi"
            | "ua" | "uâ" | "uô" | "uo" | "uơ" | "ui" | "uy" | "uê" | "ue"
            | "ưa" | "ươ" | "ưu" | "ye" | "yê"
            // 3 vowels
            | "ieu" | "iêu"
            | "oai" | "oay" | "oeo"
            | "uoi" | "uôi" | "uơi" | "ươi" | "uya" | "uye" | "uyê" | "uyu" | "uou" | "uơu" | "ươu"
            | "uoc" | "uom" | "uon" | "uong" | "uot"
            | "ươc" | "ươm" | "ươn" | "ương" | "ươp" | "ươt"
            | "yeu" | "yêu"
    )
}

/// Detects if the current word matches common code tokens:
/// - snake_case (contains '_')
/// - camelCase (internal capital letters like 'getUser')
/// - English/Code starting consonant clusters (cl-, pr-, str-, pl-, fl-, gr-, etc.)
/// - Specific programming keywords
pub fn is_programming_or_english(word: &str) -> bool {
    if word.is_empty() {
        return false;
    }

    // snake_case
    if word.contains('_') {
        return true;
    }

    let chars: Vec<char> = word.chars().collect();

    // camelCase - first char lowercase, then uppercase somewhere (e.g. "getUser", "myVar")
    if chars.len() > 1
        && chars[0].is_ascii_lowercase()
        && chars[1..].iter().any(|c| c.is_ascii_uppercase())
    {
        return true;
    }

    // Check leading consonants (e.g. "str" in "string", "cl" in "class", "pr" in "printf")
    let leading_consonants: String = chars
        .iter()
        .take_while(|c| c.is_alphabetic() && !crate::unicode::is_vietnamese_vowel(**c))
        .collect();

    if leading_consonants.chars().count() >= 2
        && !is_valid_vietnamese_initial_cluster(&leading_consonants)
    {
        return true;
    }

    // Check trailing consonant clusters (consonants after a vowel)
    // Vietnamese only allows single ending consonants, or 'ch', 'ng', 'nh'.
    let mut seen_vowel = false;
    let mut trailing_consonants = String::new();

    for &c in &chars {
        if crate::unicode::is_vietnamese_vowel(c) {
            seen_vowel = true;
            trailing_consonants.clear();
        } else if seen_vowel && c.is_alphabetic() {
            trailing_consonants.push(c.to_ascii_lowercase());
            if trailing_consonants.chars().count() >= 2
                && !matches!(trailing_consonants.as_str(), "ch" | "ng" | "nh")
            {
                return true;
            }
        }
    }

    // Exact programming keyword matches and common tech words
    let lower = word.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "async"
            | "struct"
            | "const"
            | "from"
            | "import"
            | "export"
            | "null"
            | "void"
            | "test"
            | "true"
            | "false"
            | "function"
            | "return"
            | "class"
            | "string"
            | "printf"
            | "println"
            | "main"
            | "default"
            | "break"
            | "continue"
            | "package"
            | "software"
            | "hardware"
            | "computer"
            | "server"
            | "client"
            | "script"
            | "rust"
            | "engine"
            | "engineer"
            | "english"
            | "system"
            | "service"
            | "socket"
            | "thread"
            | "window"
            | "buffer"
            | "process"
            | "device"
            | "event"
            | "action"
            | "target"
            | "filter"
            | "update"
            | "config"
            | "release"
            | "commit"
            | "cargo"
            | "error"
            | "value"
            | "input"
            | "output"
            | "status"
    ) {
        return true;
    }

    // Detect multi-syllable English structure: 2 distinct vowel groups separated by non-Vietnamese consonant transitions
    // E.g. "engin", "docke", "windo", "servi"
    let mut vowel_groups = 0;
    let mut in_vowel = false;
    for &c in &chars {
        if crate::unicode::is_vietnamese_vowel(c) {
            if !in_vowel {
                vowel_groups += 1;
                in_vowel = true;
            }
        } else {
            in_vowel = false;
        }
    }
    if vowel_groups >= 2 {
        // Vietnamese single words can have dipthongs/tripthongs together (e.g. "nghiêng", "chuyên"),
        // but separated vowel groups ("e...i" in "engin", "a...o" in "action") are English/foreign words!
        return true;
    }

    false
}

/// Checks if a word has invalid trailing consonant clusters or invalid ending consonants in composed text.
pub fn has_invalid_trailing_cluster(word: &str) -> bool {
    if word.is_empty() {
        return false;
    }

    let chars: Vec<char> = word.chars().collect();
    let mut seen_vowel = false;
    let mut trailing_consonants = String::new();

    for &c in &chars {
        if crate::unicode::is_vietnamese_vowel(c) {
            seen_vowel = true;
            trailing_consonants.clear();
        } else if seen_vowel && c.is_alphabetic() {
            trailing_consonants.push(c.to_ascii_lowercase());
        }
    }

    if !trailing_consonants.is_empty() {
        if trailing_consonants.chars().count() == 1 {
            let single = trailing_consonants.chars().next().unwrap();
            // Single consonants that NEVER end a Vietnamese syllable:
            if matches!(
                single,
                'b' | 'd' | 'đ' | 'f' | 'j' | 'k' | 'l' | 'r' | 's' | 'v' | 'w' | 'x' | 'z'
            ) {
                return true;
            }
        } else if trailing_consonants.chars().count() >= 2
            && !matches!(trailing_consonants.as_str(), "ch" | "ng" | "nh")
        {
            return true;
        }
    }

    false
}
