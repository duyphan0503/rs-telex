#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tone {
    #[default]
    None,
    Sac,   // 's'
    Huyen, // 'f'
    Hoi,   // 'r'
    Nga,   // 'x'
    Nang,  // 'j'
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VowelMark {
    None,
    Hat,   // Circumflex: â, ê, ô
    Breve, // Breve: ă
    Horn,  // Horn: ơ, ư
}

pub fn remove_tone(c: char) -> char {
    let lower = c.to_lowercase().next().unwrap_or(c);
    let is_upper = c.is_uppercase();

    let res = match lower {
        'á' | 'à' | 'ả' | 'ã' | 'ạ' => 'a',
        'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' => 'ă',
        'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' => 'â',
        'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ' => 'e',
        'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' => 'ê',
        'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị' => 'i',
        'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ' => 'o',
        'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' => 'ô',
        'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' => 'ơ',
        'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ' => 'u',
        'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' => 'ư',
        'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ' => 'y',
        _ => lower,
    };

    if is_upper {
        res.to_uppercase().next().unwrap_or(res)
    } else {
        res
    }
}

pub fn get_tone(c: char) -> Tone {
    let lower = c.to_lowercase().next().unwrap_or(c);
    match lower {
        'á' | 'ắ' | 'ấ' | 'é' | 'ế' | 'í' | 'ó' | 'ố' | 'ớ' | 'ú' | 'ứ' | 'ý' => {
            Tone::Sac
        }
        'à' | 'ằ' | 'ầ' | 'è' | 'ề' | 'ì' | 'ò' | 'ồ' | 'ờ' | 'ù' | 'ừ' | 'ỳ' => {
            Tone::Huyen
        }
        'ả' | 'ẳ' | 'ẩ' | 'ẻ' | 'ể' | 'ỉ' | 'ỏ' | 'ổ' | 'ở' | 'ủ' | 'ử' | 'ỷ' => {
            Tone::Hoi
        }
        'ã' | 'ẵ' | 'ẫ' | 'ẽ' | 'ễ' | 'ĩ' | 'õ' | 'ỗ' | 'ỡ' | 'ũ' | 'ữ' | 'ỹ' => {
            Tone::Nga
        }
        'ạ' | 'ặ' | 'ậ' | 'ẹ' | 'ệ' | 'ị' | 'ọ' | 'ộ' | 'ợ' | 'ụ' | 'ự' | 'ỵ' => {
            Tone::Nang
        }
        _ => Tone::None,
    }
}

pub fn apply_tone(base: char, tone: Tone) -> char {
    let lower = remove_tone(base).to_lowercase().next().unwrap_or(base);
    let is_upper = base.is_uppercase();

    let res = match (lower, tone) {
        ('a', Tone::None) => 'a',
        ('a', Tone::Sac) => 'á',
        ('a', Tone::Huyen) => 'à',
        ('a', Tone::Hoi) => 'ả',
        ('a', Tone::Nga) => 'ã',
        ('a', Tone::Nang) => 'ạ',

        ('ă', Tone::None) => 'ă',
        ('ă', Tone::Sac) => 'ắ',
        ('ă', Tone::Huyen) => 'ằ',
        ('ă', Tone::Hoi) => 'ẳ',
        ('ă', Tone::Nga) => 'ẵ',
        ('ă', Tone::Nang) => 'ặ',

        ('â', Tone::None) => 'â',
        ('â', Tone::Sac) => 'ấ',
        ('â', Tone::Huyen) => 'ầ',
        ('â', Tone::Hoi) => 'ẩ',
        ('â', Tone::Nga) => 'ẫ',
        ('â', Tone::Nang) => 'ậ',

        ('e', Tone::None) => 'e',
        ('e', Tone::Sac) => 'é',
        ('e', Tone::Huyen) => 'è',
        ('e', Tone::Hoi) => 'ẻ',
        ('e', Tone::Nga) => 'ẽ',
        ('e', Tone::Nang) => 'ẹ',

        ('ê', Tone::None) => 'ê',
        ('ê', Tone::Sac) => 'ế',
        ('ê', Tone::Huyen) => 'ề',
        ('ê', Tone::Hoi) => 'ể',
        ('ê', Tone::Nga) => 'ễ',
        ('ê', Tone::Nang) => 'ệ',

        ('i', Tone::None) => 'i',
        ('i', Tone::Sac) => 'í',
        ('i', Tone::Huyen) => 'ì',
        ('i', Tone::Hoi) => 'ỉ',
        ('i', Tone::Nga) => 'ĩ',
        ('i', Tone::Nang) => 'ị',

        ('o', Tone::None) => 'o',
        ('o', Tone::Sac) => 'ó',
        ('o', Tone::Huyen) => 'ò',
        ('o', Tone::Hoi) => 'ỏ',
        ('o', Tone::Nga) => 'õ',
        ('o', Tone::Nang) => 'ọ',

        ('ô', Tone::None) => 'ô',
        ('ô', Tone::Sac) => 'ố',
        ('ô', Tone::Huyen) => 'ồ',
        ('ô', Tone::Hoi) => 'ổ',
        ('ô', Tone::Nga) => 'ỗ',
        ('ô', Tone::Nang) => 'ộ',

        ('ơ', Tone::None) => 'ơ',
        ('ơ', Tone::Sac) => 'ớ',
        ('ơ', Tone::Huyen) => 'ờ',
        ('ơ', Tone::Hoi) => 'ở',
        ('ơ', Tone::Nga) => 'ỡ',
        ('ơ', Tone::Nang) => 'ợ',

        ('u', Tone::None) => 'u',
        ('u', Tone::Sac) => 'ú',
        ('u', Tone::Huyen) => 'ù',
        ('u', Tone::Hoi) => 'ủ',
        ('u', Tone::Nga) => 'ũ',
        ('u', Tone::Nang) => 'ụ',

        ('ư', Tone::None) => 'ư',
        ('ư', Tone::Sac) => 'ứ',
        ('ư', Tone::Huyen) => 'ừ',
        ('ư', Tone::Hoi) => 'ử',
        ('ư', Tone::Nga) => 'ữ',
        ('ư', Tone::Nang) => 'ự',

        ('y', Tone::None) => 'y',
        ('y', Tone::Sac) => 'ý',
        ('y', Tone::Huyen) => 'ỳ',
        ('y', Tone::Hoi) => 'ỷ',
        ('y', Tone::Nga) => 'ỹ',
        ('y', Tone::Nang) => 'ỵ',

        _ => lower,
    };

    if is_upper {
        res.to_uppercase().next().unwrap_or(res)
    } else {
        res
    }
}

pub fn is_vietnamese_vowel(c: char) -> bool {
    let lower = c.to_lowercase().next().unwrap_or(c);
    matches!(
        lower,
        'a' | 'ă'
            | 'â'
            | 'e'
            | 'ê'
            | 'i'
            | 'o'
            | 'ô'
            | 'ơ'
            | 'u'
            | 'ư'
            | 'y'
            | 'á'
            | 'à'
            | 'ả'
            | 'ã'
            | 'ạ'
            | 'ắ'
            | 'ằ'
            | 'ẳ'
            | 'ẵ'
            | 'ặ'
            | 'ấ'
            | 'ầ'
            | 'ẩ'
            | 'ẫ'
            | 'ậ'
            | 'é'
            | 'è'
            | 'ẻ'
            | 'ẽ'
            | 'ẹ'
            | 'ế'
            | 'ề'
            | 'ể'
            | 'ễ'
            | 'ệ'
            | 'í'
            | 'ì'
            | 'ỉ'
            | 'ĩ'
            | 'ị'
            | 'ó'
            | 'ò'
            | 'ỏ'
            | 'õ'
            | 'ọ'
            | 'ố'
            | 'ồ'
            | 'ổ'
            | 'ỗ'
            | 'ộ'
            | 'ớ'
            | 'ờ'
            | 'ở'
            | 'ỡ'
            | 'ợ'
            | 'ú'
            | 'ù'
            | 'ủ'
            | 'ũ'
            | 'ụ'
            | 'ứ'
            | 'ừ'
            | 'ử'
            | 'ữ'
            | 'ự'
            | 'ý'
            | 'ỳ'
            | 'ỷ'
            | 'ỹ'
            | 'ỵ'
    )
}

pub fn to_base_vowel(c: char) -> char {
    let lower = remove_tone(c).to_lowercase().next().unwrap_or(c);
    let is_upper = c.is_uppercase();

    let res = match lower {
        'â' | 'ă' | 'a' => 'a',
        'ê' | 'e' => 'e',
        'ô' | 'ơ' | 'o' => 'o',
        'ư' | 'u' => 'u',
        'i' => 'i',
        'y' => 'y',
        _ => lower,
    };

    if is_upper {
        res.to_uppercase().next().unwrap_or(res)
    } else {
        res
    }
}

pub fn to_hat(c: char) -> Option<char> {
    let tone = get_tone(c);
    let base = to_base_vowel(c).to_lowercase().next().unwrap_or(c);
    let is_upper = c.is_uppercase();

    let hat_base = match base {
        'a' => 'â',
        'e' => 'ê',
        'o' => 'ô',
        _ => return None,
    };

    let mut res = apply_tone(hat_base, tone);
    if is_upper {
        res = res.to_uppercase().next().unwrap_or(res);
    }
    Some(res)
}

pub fn to_breve(c: char) -> Option<char> {
    let tone = get_tone(c);
    let base = to_base_vowel(c).to_lowercase().next().unwrap_or(c);
    let is_upper = c.is_uppercase();

    let breve_base = match base {
        'a' => 'ă',
        _ => return None,
    };

    let mut res = apply_tone(breve_base, tone);
    if is_upper {
        res = res.to_uppercase().next().unwrap_or(res);
    }
    Some(res)
}

pub fn to_horn(c: char) -> Option<char> {
    let tone = get_tone(c);
    let base = to_base_vowel(c).to_lowercase().next().unwrap_or(c);
    let is_upper = c.is_uppercase();

    let horn_base = match base {
        'u' => 'ư',
        'o' => 'ơ',
        _ => return None,
    };

    let mut res = apply_tone(horn_base, tone);
    if is_upper {
        res = res.to_uppercase().next().unwrap_or(res);
    }
    Some(res)
}

pub fn has_hat(c: char) -> bool {
    let lower = remove_tone(c).to_lowercase().next().unwrap_or(c);
    matches!(lower, 'â' | 'ê' | 'ô')
}

pub fn has_breve(c: char) -> bool {
    let lower = remove_tone(c).to_lowercase().next().unwrap_or(c);
    matches!(lower, 'ă')
}

pub fn has_horn(c: char) -> bool {
    let lower = remove_tone(c).to_lowercase().next().unwrap_or(c);
    matches!(lower, 'ư' | 'ơ')
}
