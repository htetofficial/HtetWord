//! Myanmar language processing utilities for HtetWord:
//! syllable segmentation, syllable counting, Zawgyi ⇄ Unicode conversion,
//! numeral conversion (0-9 ⇄ ၀-၉), and line-break helpers.

/// Check if a character is within the Myanmar Unicode block.
#[inline]
pub fn is_myanmar_char(c: char) -> bool {
    matches!(c, '\u{1000}'..='\u{109F}' | '\u{A9E0}'..='\u{A9FF}' | '\u{AA60}'..='\u{AA7F}')
}

/// Check if a character is a Myanmar consonant (က to အ).
#[inline]
pub fn is_myanmar_consonant(c: char) -> bool {
    matches!(c, '\u{1000}'..='\u{1021}')
}

/// Check if a character is an independent vowel or great sa.
#[inline]
pub fn is_myanmar_independent(c: char) -> bool {
    matches!(c, '\u{1023}'..='\u{102A}' | '\u{103F}' | '\u{104E}')
}

/// Check if a character at index `i` is the onset (start) of a Myanmar syllable.
#[inline]
pub fn is_syllable_start(chars: &[char], i: usize) -> bool {
    let c = chars[i];
    if is_myanmar_independent(c) {
        return true;
    }
    if !is_myanmar_consonant(c) {
        return false;
    }
    // Check if subjoined: preceded by virama U+1039 unless it's Kinzi (င + ် + ္)
    if i > 0 && chars[i - 1] == '\u{1039}' {
        let is_kinzi = i >= 3 && chars[i - 2] == '\u{103A}' && chars[i - 3] == '\u{1004}';
        if !is_kinzi {
            return false;
        }
    }
    // Check if coda (devowelized final): followed by asat U+103A
    if i + 1 < chars.len() && chars[i + 1] == '\u{103A}' {
        return false;
    }
    true
}

/// Count syllables in Myanmar text.
///
/// In Myanmar orthography, a syllable begins with an onset consonant
/// (excluding subjoined consonants and codas with asat) or an independent vowel.
pub fn count_syllables(text: &str) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let mut count = 0;
    for i in 0..chars.len() {
        if is_syllable_start(&chars, i) {
            count += 1;
        }
    }
    count
}

/// Detailed counts for Myanmar text: words/syllables, consonants, vowels, and signs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MyanmarStats {
    pub syllables: usize,
    pub consonants: usize,
    pub vowels_and_marks: usize,
    pub digits: usize,
    pub punctuation: usize,
}

pub fn analyze_myanmar_text(text: &str) -> MyanmarStats {
    let chars: Vec<char> = text.chars().collect();
    let mut stats = MyanmarStats::default();

    for (i, &c) in chars.iter().enumerate() {
        if is_myanmar_consonant(c) {
            stats.consonants += 1;
        }
        if is_syllable_start(&chars, i) {
            stats.syllables += 1;
        }
        if matches!(c, '\u{102B}'..='\u{103E}') {
            stats.vowels_and_marks += 1;
        } else if matches!(c, '\u{1040}'..='\u{1049}') {
            stats.digits += 1;
        } else if c == '၊' || c == '။' {
            stats.punctuation += 1;
        }
    }

    stats
}

/// Convert ASCII numerals ('0'..='9') to Myanmar numerals ('၀'..='၉') or vice versa.
pub fn convert_numerals(text: &str, to_myanmar: bool) -> String {
    text.chars()
        .map(|c| {
            if to_myanmar {
                match c {
                    '0'..='9' => char::from_u32(c as u32 - '0' as u32 + 0x1040).unwrap_or(c),
                    _ => c,
                }
            } else {
                match c {
                    '၀'..='၉' => char::from_u32(c as u32 - 0x1040 + '0' as u32).unwrap_or(c),
                    _ => c,
                }
            }
        })
        .collect()
}

/// Insert Zero-Width Space (U+200B) between Myanmar syllables for proper wrapping.
pub fn insert_syllable_breaks(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len() + text.len() / 4);

    for (i, &c) in chars.iter().enumerate() {
        if i > 0 && is_syllable_start(&chars, i) {
            let prev = chars[i - 1];
            if !prev.is_whitespace() && prev != '\u{200B}' && prev != '၊' && prev != '။' {
                out.push('\u{200B}');
            }
        }
        out.push(c);
    }

    out
}

/// Convert Zawgyi-encoded text to standard Unicode.
/// Handles common Zawgyi non-standard code point shifts and reorderings.
pub fn zawgyi_to_unicode(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // 1. Zawgyi pre-base vowel E (U+1031) placed before consonant:
        // In Zawgyi: ေ [consonant] -> In Unicode: [consonant] ေ
        if c == '\u{1031}' && i + 1 < chars.len() {
            let next_c = chars[i + 1];
            if is_myanmar_consonant(next_c) {
                // Check if followed by medials in Zawgyi (e.g. ျ ြ ွ ှ)
                let mut end = i + 2;
                while end < chars.len() && is_zawgyi_medial(chars[end]) {
                    end += 1;
                }
                // Push consonant and medials first
                result.push(next_c);
                for k in (i + 2)..end {
                    result.push(map_zawgyi_char(chars[k]));
                }
                // Push Unicode 'ေ' (U+1031)
                result.push('\u{1031}');
                i = end;
                continue;
            }
        }

        // 2. Map Zawgyi specific characters to Unicode equivalents
        result.push(map_zawgyi_char(c));
        i += 1;
    }

    result
}

/// Convert Unicode text to Zawgyi encoding (for legacy compatibility).
pub fn unicode_to_zawgyi(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Unicode has consonant + ေ (U+1031) -> Zawgyi puts ေ first
        if is_myanmar_consonant(c) && i + 1 < chars.len() {
            // Find if there is a 'ေ' belonging to this syllable
            let mut j = i + 1;
            while j < chars.len() && (matches!(chars[j], '\u{103B}'..='\u{103E}' | '\u{1039}')) {
                j += 1;
            }
            if j < chars.len() && chars[j] == '\u{1031}' {
                result.push('\u{1031}');
                result.push(c);
                for k in (i + 1)..j {
                    result.push(chars[k]);
                }
                i = j + 1;
                continue;
            }
        }

        result.push(c);
        i += 1;
    }

    result
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MyanmarEncoding {
    Unicode,
    Zawgyi,
    Unknown,
}

/// Detect whether Myanmar text is encoded in Unicode or Zawgyi
/// (Based on Google Myanmar-Tools heuristics & Markov transition patterns).
pub fn detect_encoding(text: &str) -> MyanmarEncoding {
    let mut unicode_score = 0;
    let mut zawgyi_score = 0;

    let chars: Vec<char> = text.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        // Definite Zawgyi code points
        if matches!(c, '\u{106A}' | '\u{106B}' | '\u{107D}'..='\u{1085}' | '\u{1088}'..='\u{108F}' | '\u{1090}'..='\u{1097}') {
            zawgyi_score += 15;
        }

        // Zawgyi pre-base vowel E before consonant: e.g. ေ + [consonant]
        if c == '\u{1031}' && i + 1 < chars.len() && is_myanmar_consonant(chars[i + 1]) {
            zawgyi_score += 20;
        }

        // Unicode consonant followed by vowel E: e.g. [consonant] + ေ
        if is_myanmar_consonant(c) && i + 1 < chars.len() && chars[i + 1] == '\u{1031}' {
            unicode_score += 10;
        }

        // Unicode virama followed by consonant (stacked consonant / ပါဌ်ဆင့်)
        if c == '\u{1039}' && i + 1 < chars.len() && is_myanmar_consonant(chars[i + 1]) {
            unicode_score += 15;
        }

        // Unicode Kinzi (င + ် + ္)
        if c == '\u{1004}' && i + 2 < chars.len() && chars[i + 1] == '\u{103A}' && chars[i + 2] == '\u{1039}' {
            unicode_score += 25;
        }
    }

    if zawgyi_score > unicode_score && zawgyi_score >= 10 {
        MyanmarEncoding::Zawgyi
    } else if unicode_score > 0 {
        MyanmarEncoding::Unicode
    } else {
        MyanmarEncoding::Unknown
    }
}

/// Segment Myanmar text into individual syllables (myWord algorithm).
pub fn segment_syllables(text: &str) -> Vec<String> {
    let mut syllables = Vec::new();
    for token in text.split_whitespace() {
        let chars: Vec<char> = token.chars().collect();
        let mut current = String::new();

        for (i, &c) in chars.iter().enumerate() {
            if i > 0 && is_syllable_start(&chars, i) {
                if !current.is_empty() {
                    syllables.push(current.clone());
                    current.clear();
                }
            }
            current.push(c);
        }
        if !current.is_empty() {
            syllables.push(current);
        }
    }
    syllables
}

/// Common compound words list from Kanaung wordlists & Myanmar Words dictionary.
pub static COMMON_MYANMAR_COMPOUNDS: &[&str] = &[
    "ပြည်ထောင်စု", "သမ္မတ", "မြန်မာနိုင်ငံတော်", "အစိုးရ", "ဖွဲ့စည်းပုံ", "အခြေခံဥပဒေ",
    "မင်္ဂလာပါ", "ကျေးဇူးတင်ပါသည်", "ဆောင်းပါး", "စာအုပ်", "အဘိဓာန်", "မြန်မာစာ",
    "ကွန်ပျူတာ", "နည်းပညာ", "စာမျက်နှာ", "စာလုံးပေါင်း", "သတ်ပုံ", "ဘာသာစကား",
    "ယဉ်ကျေးမှု", "သမိုင်း", "ပညာရေး", "ကျန်းမာရေး", "စီးပွားရေး", "လူမှုရေး",
    "စာပေ", "ကဗျာ", "ဝတ္ထု", "သတင်း", "အယ်ဒီတာ", "စာတည်း", "ထုတ်ဝေသူ",
    "ဖတ်ရှုသူ", "ဆွေးနွေးချက်", "မှတ်ချက်", "အကြံပြုချက်", "လွတ်လပ်ရေး", "ငြိမ်းချမ်းရေး"
];

/// Segment Myanmar text into meaningful words using longest matching compound dictionary.
pub fn segment_words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let syllables = segment_syllables(text);
    let mut i = 0;

    while i < syllables.len() {
        let mut matched = false;
        for window in (2..=4).rev() {
            if i + window <= syllables.len() {
                let candidate: String = syllables[i..i + window].concat();
                if COMMON_MYANMAR_COMPOUNDS.contains(&candidate.as_str()) {
                    words.push(candidate);
                    i += window;
                    matched = true;
                    break;
                }
            }
        }
        if !matched {
            words.push(syllables[i].clone());
            i += 1;
        }
    }

    words
}

/// Structured Myanmar Traditional Calendar Information (mmcalendar algorithm).
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct MyanmarDateInfo {
    pub gregorian_year: i32,
    pub gregorian_month: u32,
    pub gregorian_day: u32,
    pub sasana_year: i32,
    pub myanmar_year: i32,
    pub month_name: String,
    pub moon_phase: String,
    pub fortnight_day: u32,
    pub weekday: String,
    pub astrological_day: String,
    pub dragon_head: String,
    pub mahabote: String,
    pub formatted_full: String,
    pub formatted_short: String,
}

/// Calculate Myanmar Traditional Calendar date from Western Gregorian (Y, M, D).
pub fn calculate_myanmar_date(year: i32, month: u32, day: u32) -> MyanmarDateInfo {
    // Julian Day Number
    let a = (14 - month as i32) / 12;
    let y = year + 4800 - a;
    let m = month as i32 + 12 * a - 3;
    let jdn = day as i32 + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045;

    // Solar Year (SY) and Lunar Month (LM) constants (mmcalendar Makaritha system)
    let sy = 365.2587565;
    let lm = 29.53058795;
    let mo = 1954168.050623;

    let jdn_f = jdn as f64;
    let my = ((jdn_f - 0.5 - mo) / sy).floor() as i32;
    let by = my + 1182;

    let day_in_year = (jdn_f - 0.5 - mo) - (my as f64) * sy;

    let months = [
        "ပထမဝါဆို", "တန်ခူး", "ကဆုန်", "နယုန်", "ဝါဆို",
        "ဝါခေါင်", "တော်သလင်း", "သီတင်းကျွတ်", "တန်ဆောင်မုန်း",
        "နတ်တော်", "ပြာသို", "တပို့တွဲ", "တပေါင်း", "နှောင်းတန်ခူး", "နှောင်းကဆုန်",
    ];

    let weekdays = ["တနင်္ဂနွေ", "တနင်္လာ", "အင်္ဂါ", "ဗုဒ္ဓဟူး", "ကြာသပတေး", "သောကြာ", "စနေ"];
    let wd = ((jdn + 1) % 7) as usize;
    let weekday_name = weekdays[wd].to_string();

    let d_lunar = (day_in_year % lm).floor() as u32;
    let (moon_phase, fortnight_day) = if d_lunar == 0 {
        ("လပြည့်".to_string(), 15)
    } else if d_lunar < 15 {
        ("လဆန်း".to_string(), d_lunar)
    } else if d_lunar == 15 {
        ("လကွယ်".to_string(), 15)
    } else {
        ("လဆုတ်".to_string(), d_lunar - 15)
    };

    let mut m_idx = (day_in_year / lm).floor() as usize + 1;
    if m_idx >= months.len() {
        m_idx = months.len() - 1;
    }
    let month_name = months[m_idx].to_string();

    // Astrological Day check (Yatyaza / Pyathada)
    let yatyaza_map = [
        vec![],
        vec![4, 3], // Tagu: Thu, Wed
        vec![5, 2], // Kason: Fri, Tue
        vec![1, 6], // Nayon: Mon, Sat
        vec![0],    // Waso: Sun
        vec![4, 3], // Wagaung: Thu, Wed
        vec![5, 2], // Tawthalin: Fri, Tue
        vec![4, 3], // Thadingyut: Thu, Wed
        vec![5, 2], // Tazaungmon: Fri, Tue
        vec![1, 6], // Nadaw: Mon, Sat
        vec![0],    // Pyatho: Sun
        vec![4, 3], // Tabodwe: Thu, Wed
        vec![5, 2], // Tabaung: Fri, Tue
    ];

    let pyathada_map = [
        vec![],
        vec![0], // Tagu: Sun
        vec![2], // Kason: Tue
        vec![4], // Nayon: Thu
        vec![6], // Waso: Sat
        vec![1], // Wagaung: Mon
        vec![3], // Tawthalin: Wed
        vec![0], // Thadingyut: Sun
        vec![2], // Tazaungmon: Tue
        vec![4], // Nadaw: Thu
        vec![6], // Pyatho: Sat
        vec![1], // Tabodwe: Mon
        vec![3], // Tabaung: Wed
    ];

    let mut astro = Vec::new();
    if m_idx < yatyaza_map.len() && yatyaza_map[m_idx].contains(&(wd as i32)) {
        astro.push("ရက်ရာဇာ");
    }
    if m_idx < pyathada_map.len() && pyathada_map[m_idx].contains(&(wd as i32)) {
        astro.push("ပြဿဒါး");
    }
    let astrological_day = if astro.is_empty() {
        "သာမန်ရက်".to_string()
    } else {
        astro.join("၊ ")
    };

    // Dragon head orientation
    let dragon_head = match m_idx {
        1..=3 => "အနောက်ဘက်သို့ ဦးခေါင်းလှည့်",
        4..=6 => "မြောက်ဘက်သို့ ဦးခေါင်းလှည့်",
        7..=9 => "အရှေ့ဘက်သို့ ဦးခေါင်းလှည့်",
        _ => "တောင်ဘက်သို့ ဦးခေါင်းလှည့်",
    }.to_string();

    // Mahabote
    let mahabote_names = ["ဘင်္ဂ", "အထွန်း", "ရာဇ", "အဓိပတိ", "မရဏ", "အာယု", "ပုတိ"];
    let mb_idx = ((wd as i32 - (my % 7)).rem_euclid(7)) as usize;
    let mahabote = mahabote_names[mb_idx].to_string();

    let my_burmese = convert_numerals(&my.to_string(), true);
    let by_burmese = convert_numerals(&by.to_string(), true);
    let fd_burmese = convert_numerals(&fortnight_day.to_string(), true);

    let formatted_full = format!(
        "သာသနာနှစ် {} ခု၊ မြန်မာသက္ကရာဇ် {} ခု၊ {}{}({})ရက်၊ {}နေ့ ({})",
        by_burmese, my_burmese, month_name, moon_phase, fd_burmese, weekday_name, astrological_day
    );

    let formatted_short = format!(
        "{} ခု၊ {}{}{} ရက်",
        my_burmese, month_name, moon_phase, fd_burmese
    );

    MyanmarDateInfo {
        gregorian_year: year,
        gregorian_month: month,
        gregorian_day: day,
        sasana_year: by,
        myanmar_year: my,
        month_name,
        moon_phase,
        fortnight_day,
        weekday: weekday_name,
        astrological_day,
        dragon_head,
        mahabote,
        formatted_full,
        formatted_short,
    }
}

#[inline]
fn is_zawgyi_medial(c: char) -> bool {
    matches!(
        c,
        '\u{103A}' | '\u{103B}' | '\u{103C}' | '\u{103D}' | '\u{103E}'
            | '\u{107D}'..='\u{1085}'
    )
}

#[inline]
fn map_zawgyi_char(c: char) -> char {
    match c {
        // Zawgyi Great Sa -> Unicode U+103F
        '\u{106A}' => '\u{103F}',
        // Zawgyi Nya -> Unicode U+1007
        '\u{106B}' => '\u{1007}',
        // Zawgyi Na-Gyi -> Unicode U+1025
        '\u{1090}' => '\u{1025}',
        // Zawgyi Medial Ra variants -> Unicode Medial Ra U+103C (Rabbit rules)
        '\u{107D}' | '\u{107E}' | '\u{107F}' | '\u{1080}' | '\u{1081}' | '\u{1082}' | '\u{1083}' | '\u{1084}' => '\u{103C}',
        // Zawgyi Medial Ha variants -> Unicode Medial Ha U+103E
        '\u{1085}' => '\u{103E}',
        // Zawgyi zero vs Myanmar digit zero
        '\u{1064}' => '\u{1040}',
        // Keep standard
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_syllables() {
        assert_eq!(count_syllables("မြန်မာစာ"), 3);
        assert_eq!(count_syllables("မင်္ဂလာပါ"), 4);
    }

    #[test]
    fn test_segment_words() {
        let words = segment_words("မင်္ဂလာပါ ပြည်ထောင်စု စာအုပ်");
        assert!(words.contains(&"မင်္ဂလာပါ".to_string()));
        assert!(words.contains(&"ပြည်ထောင်စု".to_string()));
        assert!(words.contains(&"စာအုပ်".to_string()));
    }

    #[test]
    fn test_calculate_myanmar_date() {
        let info = calculate_myanmar_date(2026, 11, 21);
        assert_eq!(info.myanmar_year, 1388);
        assert_eq!(info.sasana_year, 2570);
        assert_eq!(info.weekday, "စနေ");
        assert!(info.formatted_full.contains("၁၃၈၈"));
    }

    #[test]
    fn test_detect_encoding() {
        assert_eq!(detect_encoding("မင်္ဂလာပါ"), MyanmarEncoding::Unicode);
        // Zawgyi pre-base 'ေမတ္တာ' where 'ေ' is before 'မ'
        assert_eq!(detect_encoding("\u{1031}\u{1019}"), MyanmarEncoding::Zawgyi);
    }

    #[test]
    fn test_numeral_conversion() {
        assert_eq!(convert_numerals("2026", true), "၂၀၂၆");
        assert_eq!(convert_numerals("၂၀၂၆", false), "2026");
        assert_eq!(convert_numerals("Page 12 of 50", true), "Page ၁၂ of ၅၀");
    }

    #[test]
    fn test_syllable_breaks() {
        let broken = insert_syllable_breaks("မြန်မာစာ");
        assert!(broken.contains('\u{200B}'));
    }
}
