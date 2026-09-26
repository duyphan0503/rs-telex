use crate::syllable::{
    has_invalid_trailing_cluster, is_programming_or_english, is_valid_vietnamese_initial_cluster,
};
use crate::unicode::{
    Tone, apply_tone, get_tone, has_breve, has_hat, has_horn, is_vietnamese_vowel, remove_tone,
    to_base_vowel, to_breve, to_hat, to_horn,
};

#[derive(Debug, PartialEq, Eq)]
pub enum TelexAction {
    /// Commit the character directly without changing buffer
    PassThrough(char),
    /// Replace the last N committed chars with new_text
    Replace { backspaces: usize, new_text: String },
}

#[derive(Default, Debug, Clone)]
pub struct TelexBuffer {
    pub raw: String,
    pub composed: String,
    pub current_tone: Tone,
    pub is_english: bool,
}

impl TelexBuffer {
    pub fn new() -> Self {
        Self {
            raw: String::new(),
            composed: String::new(),
            current_tone: Tone::None,
            is_english: false,
        }
    }

    pub fn reset(&mut self) {
        self.raw.clear();
        self.composed.clear();
        self.current_tone = Tone::None;
        self.is_english = false;
    }

    pub fn is_empty(&self) -> bool {
        self.composed.is_empty()
    }

    pub fn pop_char(&mut self) -> Option<char> {
        self.raw.pop();
        let popped = self.composed.pop();
        if self.composed.is_empty() {
            self.reset();
        } else {
            // Re-detect current tone
            self.current_tone = Tone::None;
            for c in self.composed.chars() {
                let t = get_tone(c);
                if t != Tone::None {
                    self.current_tone = t;
                    break;
                }
            }
        }
        popped
    }

    /// Process a newly pressed character
    pub fn process_char(&mut self, c: char) -> TelexAction {
        // Non-alphabetic chars (space, punctuation, digits) commit current word and reset
        if !c.is_alphabetic() && c != '_' {
            self.reset();
            return TelexAction::PassThrough(c);
        }

        if self.is_english {
            self.raw.push(c);
            self.composed.push(c);
            return TelexAction::PassThrough(c);
        }

        let lower = c.to_ascii_lowercase();

        // If word starts with 'w'/'W' followed by another vowel or consonant (e.g., "web", "wifi", "windows")
        if (self.composed.starts_with('w') || self.composed.starts_with('W'))
            && self.composed.len() == 1
            && lower != 's'
            && lower != 'f'
            && lower != 'r'
            && lower != 'x'
            && lower != 'j'
            && lower != 'w'
        {
            self.raw.push(c);
            self.composed.push(c);
            self.is_english = true;
            return TelexAction::PassThrough(c);
        }

        // 1. Try Vowel / Consonant Modifier keys: 'a', 'e', 'o', 'w', 'd'
        if matches!(lower, 'a' | 'e' | 'o' | 'w' | 'd')
            && let Some(action) = self.try_modify_vowel_or_d(c)
        {
            return action;
        }

        // 2. Try Tone keys: 's', 'f', 'r', 'x', 'j'
        let has_vowel = self.composed.chars().any(is_vietnamese_vowel);
        if has_vowel {
            let target_tone = match lower {
                's' => Some(Tone::Sac),
                'f' => Some(Tone::Huyen),
                'r' => Some(Tone::Hoi),
                'x' => Some(Tone::Nga),
                'j' => Some(Tone::Nang),
                _ => None,
            };

            if let Some(new_tone) = target_tone
                && let Some(action) = self.try_apply_or_undo_tone(c, new_tone)
            {
                return action;
            }
        } else {
            // If no vowel yet and user types 'f' or 'x' after a non-empty valid consonant cluster (e.g. "chf", "nghx"), swallow it
            if matches!(lower, 'f' | 'x')
                && !self.composed.is_empty()
                && is_valid_vietnamese_initial_cluster(&self.composed)
            {
                return TelexAction::PassThrough('\0');
            }
        }

        // 3. Try Tone Removal key: 'z'
        if lower == 'z'
            && let Some(action) = self.try_remove_tone(c)
        {
            return action;
        }

        // 4. Fallback: Regular letter typed
        // Check if adding this character creates an invalid Vietnamese word or trailing cluster (e.g. "test", "export", "async", "software")
        let mut candidate_composed = self.composed.clone();
        candidate_composed.push(c);

        let mut candidate_raw = self.raw.clone();
        candidate_raw.push(c);

        let is_code_word = is_programming_or_english(&candidate_composed)
            || is_exact_code_keyword(&candidate_raw)
            || has_invalid_trailing_cluster(&candidate_composed);

        if is_code_word {
            // Special case for typing rust: if raw is "russt" -> "rust"
            if candidate_composed.eq_ignore_ascii_case("russt")
                || candidate_raw.eq_ignore_ascii_case("russt")
            {
                let old_len = self.composed.chars().count();
                let restored = if candidate_composed
                    .chars()
                    .next()
                    .unwrap_or('r')
                    .is_uppercase()
                {
                    "Rust".to_string()
                } else {
                    "rust".to_string()
                };
                self.composed = restored.clone();
                self.raw = restored.clone();
                self.current_tone = Tone::None;
                self.is_english = true;
                return TelexAction::Replace {
                    backspaces: old_len,
                    new_text: restored,
                };
            }

            // Auto-restore raw text if tone/modification was applied (e.g. "té" + "t" -> "test", "ẻ" + "p" -> "exp", "á" + "y" -> "asy")
            if self.current_tone != Tone::None || self.composed != self.raw {
                let old_len = self.composed.chars().count();
                let mut restored = self.raw.clone();
                restored.push(c);
                self.composed = restored.clone();
                self.raw = restored.clone();
                self.current_tone = Tone::None;
                self.is_english = true;
                return TelexAction::Replace {
                    backspaces: old_len,
                    new_text: restored,
                };
            }
            self.is_english = true;
            self.raw.push(c);
            self.composed.push(c);
            return TelexAction::PassThrough(c);
        }

        self.raw.push(c);
        self.composed.push(c);
        TelexAction::PassThrough(c)
    }

    /// Tries to apply vowel modifications (a->â, e->ê, o->ô, w->ư/ơ/ă/ươ/ưu, d->đ) or undo them.
    fn try_modify_vowel_or_d(&mut self, c: char) -> Option<TelexAction> {
        let lower = c.to_ascii_lowercase();

        if self.composed.is_empty() {
            return None;
        }

        let old_len = self.composed.chars().count();
        let chars: Vec<char> = self.composed.chars().collect();

        match lower {
            'w' => {
                // Special case: "w" at start of word -> "ư" when followed by another 'w'
                if chars.len() == 1 && (chars[0] == 'w' || chars[0] == 'W') {
                    let u_char = if chars[0].is_uppercase() { 'Ư' } else { 'ư' };
                    self.composed = u_char.to_string();
                    self.raw = self.composed.clone();
                    return Some(TelexAction::Replace {
                        backspaces: 1,
                        new_text: self.composed.clone(),
                    });
                }

                // Case 1: "uo" -> "ươ" (e.g., "đuoc" + "w" -> "đươc", "muon" + "w" -> "mươn")
                for i in 0..chars.len().saturating_sub(1) {
                    let base_u = to_base_vowel(chars[i]).to_ascii_lowercase();
                    let base_o = to_base_vowel(chars[i + 1]).to_ascii_lowercase();
                    if base_u == 'u'
                        && base_o == 'o'
                        && !has_horn(chars[i])
                        && !has_horn(chars[i + 1])
                    {
                        let mut new_chars = chars.clone();
                        let tone_u = get_tone(chars[i]);
                        let tone_o = get_tone(chars[i + 1]);
                        let u_horn = if chars[i].is_uppercase() { 'Ư' } else { 'ư' };
                        let o_horn = if chars[i + 1].is_uppercase() {
                            'Ơ'
                        } else {
                            'ơ'
                        };
                        new_chars[i] = apply_tone(u_horn, tone_u);
                        new_chars[i + 1] = apply_tone(o_horn, tone_o);

                        let new_composed: String = new_chars.into_iter().collect();
                        self.composed = new_composed.clone();
                        self.raw.push(c);
                        return Some(TelexAction::Replace {
                            backspaces: old_len,
                            new_text: new_composed,
                        });
                    }
                }

                // Case 2: "uu" -> "ưu" (e.g., "luu" + "w" -> "lưu", "huu" + "w" -> "hưu", "muu" + "w" -> "mưu")
                for i in 0..chars.len().saturating_sub(1) {
                    let base_u1 = to_base_vowel(chars[i]).to_ascii_lowercase();
                    let base_u2 = to_base_vowel(chars[i + 1]).to_ascii_lowercase();
                    if base_u1 == 'u'
                        && base_u2 == 'u'
                        && !has_horn(chars[i])
                        && !has_horn(chars[i + 1])
                    {
                        let mut new_chars = chars.clone();
                        let tone_u1 = get_tone(chars[i]);
                        let u_horn = if chars[i].is_uppercase() { 'Ư' } else { 'ư' };
                        new_chars[i] = apply_tone(u_horn, tone_u1);
                        // chars[i+1] stays 'u'

                        let new_composed: String = new_chars.into_iter().collect();
                        self.composed = new_composed.clone();
                        self.raw.push(c);
                        return Some(TelexAction::Replace {
                            backspaces: old_len,
                            new_text: new_composed,
                        });
                    }
                }

                // Case 3: 'u' -> 'ư'
                for i in (0..chars.len()).rev() {
                    let base = to_base_vowel(chars[i]).to_ascii_lowercase();
                    if base == 'u'
                        && !has_horn(chars[i])
                        && let Some(horned) = to_horn(chars[i])
                    {
                        let mut new_chars = chars.clone();
                        new_chars[i] = horned;
                        let new_composed: String = new_chars.into_iter().collect();
                        self.composed = new_composed.clone();
                        self.raw.push(c);
                        return Some(TelexAction::Replace {
                            backspaces: old_len,
                            new_text: new_composed,
                        });
                    }
                }

                // Case 4: 'o' -> 'ơ'
                for i in (0..chars.len()).rev() {
                    let base = to_base_vowel(chars[i]).to_ascii_lowercase();
                    if base == 'o'
                        && !has_horn(chars[i])
                        && !has_hat(chars[i])
                        && let Some(horned) = to_horn(chars[i])
                    {
                        let mut new_chars = chars.clone();
                        new_chars[i] = horned;
                        let new_composed: String = new_chars.into_iter().collect();
                        self.composed = new_composed.clone();
                        self.raw.push(c);
                        return Some(TelexAction::Replace {
                            backspaces: old_len,
                            new_text: new_composed,
                        });
                    }
                }

                // Case 5: 'a' -> 'ă'
                for i in (0..chars.len()).rev() {
                    let base = to_base_vowel(chars[i]).to_ascii_lowercase();
                    if base == 'a'
                        && !has_breve(chars[i])
                        && !has_hat(chars[i])
                        && let Some(breved) = to_breve(chars[i])
                    {
                        let mut new_chars = chars.clone();
                        new_chars[i] = breved;
                        let new_composed: String = new_chars.into_iter().collect();
                        self.composed = new_composed.clone();
                        self.raw.push(c);
                        return Some(TelexAction::Replace {
                            backspaces: old_len,
                            new_text: new_composed,
                        });
                    }
                }

                // Case 6: Undo 'w' modification (e.g. "tư" + "w" -> "tuw", "căn" + "w" -> "canw")
                let has_any_w_mod = chars.iter().any(|&ch| has_horn(ch) || has_breve(ch));
                if has_any_w_mod {
                    let mut unmod = String::new();
                    for &ch in &chars {
                        let tone = get_tone(ch);
                        let base = to_base_vowel(ch);
                        unmod.push(apply_tone(base, tone));
                    }
                    unmod.push(c);
                    self.composed = unmod.clone();
                    self.raw = unmod.clone();
                    return Some(TelexAction::Replace {
                        backspaces: old_len,
                        new_text: unmod,
                    });
                }
            }

            'a' => {
                // Find 'a' without hat in current syllable -> 'â' (e.g., "van" + "a" -> "vân", "aa" -> "â")
                if let Some(idx) = find_vowel_target_in_current_syllable(&chars, 'a')
                    && !has_hat(chars[idx])
                    && !has_breve(chars[idx])
                    && let Some(hatted) = to_hat(chars[idx])
                {
                    let mut new_chars = chars.clone();
                    new_chars[idx] = hatted;
                    let new_composed: String = new_chars.into_iter().collect();
                    self.composed = new_composed.clone();
                    self.raw.push(c);
                    return Some(TelexAction::Replace {
                        backspaces: old_len,
                        new_text: new_composed,
                    });
                }

                // Undo 'a' modification if 'â' already exists (e.g., "vân" + "a" -> "vana")
                let has_a_hat = chars
                    .iter()
                    .any(|&ch| has_hat(ch) && to_base_vowel(ch).eq_ignore_ascii_case(&'a'));
                if has_a_hat {
                    let mut unmod = String::new();
                    for &ch in &chars {
                        if has_hat(ch) && to_base_vowel(ch).eq_ignore_ascii_case(&'a') {
                            let tone = get_tone(ch);
                            unmod.push(apply_tone('a', tone));
                        } else {
                            unmod.push(ch);
                        }
                    }
                    unmod.push(c);
                    self.composed = unmod.clone();
                    self.raw = unmod.clone();
                    return Some(TelexAction::Replace {
                        backspaces: old_len,
                        new_text: unmod,
                    });
                }
            }

            'e' => {
                // Find 'e' without hat in current syllable -> 'ê' (e.g., "ee" -> "ê", "men" + "e" -> "mên")
                if let Some(idx) = find_vowel_target_in_current_syllable(&chars, 'e')
                    && !has_hat(chars[idx])
                    && let Some(hatted) = to_hat(chars[idx])
                {
                    let mut new_chars = chars.clone();
                    new_chars[idx] = hatted;
                    let new_composed: String = new_chars.into_iter().collect();
                    self.composed = new_composed.clone();
                    self.raw.push(c);
                    return Some(TelexAction::Replace {
                        backspaces: old_len,
                        new_text: new_composed,
                    });
                }

                // Undo 'e' modification if 'ê' already exists
                let has_e_hat = chars
                    .iter()
                    .any(|&ch| has_hat(ch) && to_base_vowel(ch).eq_ignore_ascii_case(&'e'));
                if has_e_hat {
                    let mut unmod = String::new();
                    for &ch in &chars {
                        if has_hat(ch) && to_base_vowel(ch).eq_ignore_ascii_case(&'e') {
                            let tone = get_tone(ch);
                            unmod.push(apply_tone('e', tone));
                        } else {
                            unmod.push(ch);
                        }
                    }
                    unmod.push(c);
                    self.composed = unmod.clone();
                    self.raw = unmod.clone();
                    return Some(TelexAction::Replace {
                        backspaces: old_len,
                        new_text: unmod,
                    });
                }
            }

            'o' => {
                // Find 'o' without hat/horn in current syllable -> 'ô' (e.g., "khong" + "o" -> "không", "loi" + "o" -> "lôi")
                if let Some(idx) = find_vowel_target_in_current_syllable(&chars, 'o')
                    && !has_hat(chars[idx])
                    && !has_horn(chars[idx])
                    && let Some(hatted) = to_hat(chars[idx])
                {
                    let mut new_chars = chars.clone();
                    new_chars[idx] = hatted;
                    let new_composed: String = new_chars.into_iter().collect();
                    self.composed = new_composed.clone();
                    self.raw.push(c);
                    return Some(TelexAction::Replace {
                        backspaces: old_len,
                        new_text: new_composed,
                    });
                }

                // Undo 'o' modification if 'ô' already exists (e.g., "không" + "o" -> "khongo")
                let has_o_hat = chars
                    .iter()
                    .any(|&ch| has_hat(ch) && to_base_vowel(ch).eq_ignore_ascii_case(&'o'));
                if has_o_hat {
                    let mut unmod = String::new();
                    for &ch in &chars {
                        if has_hat(ch) && to_base_vowel(ch).eq_ignore_ascii_case(&'o') {
                            let tone = get_tone(ch);
                            unmod.push(apply_tone('o', tone));
                        } else {
                            unmod.push(ch);
                        }
                    }
                    unmod.push(c);
                    self.composed = unmod.clone();
                    self.raw = unmod.clone();
                    return Some(TelexAction::Replace {
                        backspaces: old_len,
                        new_text: unmod,
                    });
                }
            }

            'd' => {
                // First char is 'd' or 'D' -> 'đ' or 'Đ' (e.g. "dd" -> "đ", "da" + "d" -> "đa")
                let first = chars[0];
                let is_first_upper = first.is_uppercase();
                let lower_first = first.to_ascii_lowercase();

                if lower_first == 'd' {
                    let mut new_chars = chars.clone();
                    new_chars[0] = if is_first_upper { 'Đ' } else { 'đ' };
                    let new_composed: String = new_chars.into_iter().collect();
                    self.composed = new_composed.clone();
                    self.raw.push(c);
                    return Some(TelexAction::Replace {
                        backspaces: old_len,
                        new_text: new_composed,
                    });
                } else if first == 'đ' || first == 'Đ' {
                    // Undo 'đ' -> 'dd'
                    let mut unmod = String::new();
                    unmod.push(if is_first_upper { 'D' } else { 'd' });
                    unmod.push(c);
                    for &ch in &chars[1..] {
                        unmod.push(ch);
                    }
                    self.composed = unmod.clone();
                    self.raw = unmod.clone();
                    return Some(TelexAction::Replace {
                        backspaces: old_len,
                        new_text: unmod,
                    });
                }
            }

            _ => {}
        }

        None
    }

    /// Tries to apply a tone or toggle/undo an existing tone.
    fn try_apply_or_undo_tone(
        &mut self,
        original_char: char,
        new_tone: Tone,
    ) -> Option<TelexAction> {
        if self.composed.is_empty() {
            return None;
        }

        let has_vowel = self.composed.chars().any(is_vietnamese_vowel);
        if !has_vowel {
            return None;
        }

        // Do not apply tone if word is English/programming or ends with invalid trailing clusters
        if is_programming_or_english(&self.composed) || has_invalid_trailing_cluster(&self.composed)
        {
            return None;
        }

        let old_len = self.composed.chars().count();

        // If the same tone is pressed again -> undo tone and append the raw key (e.g. "má" + "s" -> "mas")
        if self.current_tone == new_tone {
            let mut unaccented = String::new();
            for ch in self.composed.chars() {
                unaccented.push(remove_tone(ch));
            }
            unaccented.push(original_char);
            self.composed = unaccented.clone();
            self.raw = unaccented.clone();
            self.current_tone = Tone::None;

            return Some(TelexAction::Replace {
                backspaces: old_len,
                new_text: unaccented,
            });
        }

        // Find standard tone target vowel index
        let vowel_idx = self.find_tone_target_index();
        if let Some(idx) = vowel_idx {
            let chars: Vec<char> = self.composed.chars().collect();
            let mut new_chars = Vec::new();

            for (i, &ch) in chars.iter().enumerate() {
                if i == idx {
                    new_chars.push(apply_tone(ch, new_tone));
                } else {
                    new_chars.push(remove_tone(ch));
                }
            }

            let new_composed: String = new_chars.into_iter().collect();
            self.current_tone = new_tone;
            self.raw.push(original_char);
            self.composed = new_composed.clone();

            return Some(TelexAction::Replace {
                backspaces: old_len,
                new_text: new_composed,
            });
        }

        None
    }

    /// Tries to remove tone (keeps hats/horns) via 'z' key.
    fn try_remove_tone(&mut self, _original_char: char) -> Option<TelexAction> {
        if self.composed.is_empty() || self.current_tone == Tone::None {
            return None;
        }

        let old_len = self.composed.chars().count();
        let mut new_composed = String::new();
        for ch in self.composed.chars() {
            new_composed.push(remove_tone(ch));
        }

        if new_composed != self.composed {
            self.current_tone = Tone::None;
            self.composed = new_composed.clone();
            self.raw = new_composed.clone();
            return Some(TelexAction::Replace {
                backspaces: old_len,
                new_text: new_composed,
            });
        }

        None
    }

    /// Finds vowel position using standard Vietnamese placement rule (Unikey default - "hòa", "được", "lỗi", "quà", "giá", "triều")
    fn find_tone_target_index(&self) -> Option<usize> {
        let chars: Vec<char> = self.composed.chars().collect();
        let mut vowels: Vec<(usize, char)> = chars
            .iter()
            .enumerate()
            .filter_map(|(i, &c)| {
                if is_vietnamese_vowel(c) {
                    Some((i, c))
                } else {
                    None
                }
            })
            .collect();

        if vowels.is_empty() {
            return None;
        }

        // Special handling for initial "qu" and "gi" clusters:
        // In "qu", 'u' acts as a semivowel consonant modifier (e.g. "quà", "quán", "quê")
        let lower_composed: String = chars.iter().map(|c| c.to_ascii_lowercase()).collect();
        if lower_composed.starts_with("qu") && vowels.len() >= 2 && vowels[0].0 == 1 {
            vowels.remove(0); // Ignore 'u' in "qu"
        } else if lower_composed.starts_with("gi") && vowels.len() >= 2 && vowels[0].0 == 1 {
            vowels.remove(0); // Ignore 'i' in "gi" when followed by another vowel
        }

        if vowels.len() == 1 {
            return Some(vowels[0].0);
        }

        // Special case for "ươ": tone ALWAYS goes on 'ơ' (e.g. "được", "người", "rượu", "nước", "cười")
        for i in 0..vowels.len().saturating_sub(1) {
            let v1 = to_base_vowel(vowels[i].1).to_ascii_lowercase();
            let v2 = to_base_vowel(vowels[i + 1].1).to_ascii_lowercase();
            if v1 == 'u' && v2 == 'o' && has_horn(vowels[i].1) && has_horn(vowels[i + 1].1) {
                return Some(vowels[i + 1].0);
            }
        }

        // Priority rule for vowels with hats/horns (ê, ô, ơ, â, ă, ư):
        // In Vietnamese diphthongs like "iê", "uô", "yê", "uâ", tone ALWAYS goes to the vowel with mark!
        // E.g., "triều", "tiến", "cuộn", "yến", "xuân"
        for &(idx, ch) in vowels.iter().rev() {
            if has_hat(ch) || has_horn(ch) || has_breve(ch) {
                return Some(idx);
            }
        }

        // Check if there is a trailing consonant after the last vowel
        let last_vowel_idx = vowels.last()?.0;
        let has_ending_consonant = last_vowel_idx < chars.len() - 1;

        if vowels.len() == 2 {
            if has_ending_consonant {
                // When there is an ending consonant (e.g. "hoán", "luận"):
                // Tone always goes to the second vowel!
                return Some(vowels[1].0);
            } else {
                // No ending consonant:
                // Standard Unikey placement puts tone on first vowel for "oa", "oe", "uy" ("hòa", "hòe", "thủy")
                // and for all other plain diphthongs ("ua", "ia", "oi", "ai", "ay", "au", "ao", "eo", "iu", "ui")
                return Some(vowels[0].0);
            }
        }

        if vowels.len() >= 3 {
            if has_ending_consonant {
                // In triphthongs with ending consonant (e.g. "uyên" in "nguyễn", "khuyên", "tuyệt", "chuyến"):
                // Tone ALWAYS goes to the third vowel (e/ê)!
                return Some(vowels[2].0);
            } else {
                // In triphthongs without ending consonant (e.g., "ngoài", "khuỷu", "rượu", "chuối"):
                // Tone goes to the middle vowel (second vowel)!
                return Some(vowels[1].0);
            }
        }

        Some(vowels[0].0)
    }
}

/// Checks if a word matches exact programming/tech keywords
fn is_exact_code_keyword(word: &str) -> bool {
    let lower = word.to_ascii_lowercase();
    matches!(
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
            | "system"
            | "service"
            | "socket"
            | "window"
            | "buffer"
            | "process"
            | "commit"
            | "cargo"
            | "error"
    )
}

/// Scans backwards from the end of `chars` for a vowel matching `target_base`.
/// Strictly confines the search within the current syllable's vowel cluster.
/// Allows diphthongs (e.g. 'oi' in 'loi' + 'o' -> 'lôi'), but stops immediately
/// if a consonant separates the current vowel from a previous vowel (e.g. 'e...ng...i...n' in 'engin').
fn find_vowel_target_in_current_syllable(chars: &[char], target_base: char) -> Option<usize> {
    if chars.is_empty() {
        return None;
    }
    let mut i = chars.len();
    let mut trailing_consonants = String::new();
    let mut in_vowel_cluster = false;

    while i > 0 {
        i -= 1;
        let ch = chars[i];
        if is_vietnamese_vowel(ch) {
            in_vowel_cluster = true;
            let base = to_base_vowel(ch).to_ascii_lowercase();
            if base == target_base {
                return Some(i);
            }
            // If it's a different vowel within the same contiguous vowel cluster (e.g. 'i' in "loi" when target is 'o'),
            // continue looking backwards within the contiguous vowel cluster.
        } else {
            if in_vowel_cluster {
                // We were in the vowel cluster and have now hit a preceding consonant (e.g., 'ng' in "engin" before 'i').
                // Stop immediately: cannot cross into a preceding syllable!
                return None;
            }
            trailing_consonants.insert(0, ch.to_ascii_lowercase());
            // Valid single or ending consonant endings in Vietnamese:
            // "c", "ch", "m", "n", "ng", "nh", "p", "t", and transient prefixes during typing ("g", "h")
            if !matches!(
                trailing_consonants.as_str(),
                "c" | "ch" | "m" | "n" | "ng" | "nh" | "p" | "t" | "g" | "h"
            ) {
                return None;
            }
        }
    }
    None
}
