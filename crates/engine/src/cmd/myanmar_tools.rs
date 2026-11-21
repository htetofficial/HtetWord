//! Myanmar language tools commands: Zawgyi ⇄ Unicode, numerals, syllable breaks and counting.

use chrono::Datelike;
use serde_json::{Value, json};
use crate::myanmar;
use crate::{CmdResult, CommandSpec, Session};
use super::{sel_result, type_text};

/// Get the text in the selection or the active paragraph if selection is collapsed.
fn get_target_text(s: &Session) -> Option<String> {
    if !s.sel.is_collapsed() {
        let (a, b) = s.sel.ordered();
        let fragment = s.doc.copy_range(&a, &b);
        Some(fragment.clipboard_text())
    } else {
        // Active paragraph text
        let at = &s.sel.focus;
        s.doc.para_at(at).map(|p| p.text.clone())
    }
}

/// Convert Zawgyi to Unicode in the current selection or paragraph.
fn zawgyi_to_unicode_cmd(s: &mut Session, _v: &Value) -> CmdResult {
    if let Some(target) = get_target_text(s) {
        if !target.is_empty() {
            let converted = myanmar::zawgyi_to_unicode(&target);
            if converted != target {
                if s.sel.is_collapsed() {
                    // Select current paragraph and replace
                    let at = s.sel.focus.clone();
                    if let Some(p) = s.doc.para_at(&at) {
                        let a = Pos { story: at.story, path: at.path.clone(), off: 0 };
                        let b = Pos { story: at.story, path: at.path.clone(), off: p.len() };
                        s.sel = crate::Selection { anchor: a, focus: b };
                    }
                }
                type_text(s, &converted)?;
            }
        }
    }
    sel_result(s)
}

/// Convert Unicode to Zawgyi in the current selection or paragraph.
fn unicode_to_zawgyi_cmd(s: &mut Session, _v: &Value) -> CmdResult {
    if let Some(target) = get_target_text(s) {
        if !target.is_empty() {
            let converted = myanmar::unicode_to_zawgyi(&target);
            if converted != target {
                if s.sel.is_collapsed() {
                    let at = s.sel.focus.clone();
                    if let Some(p) = s.doc.para_at(&at) {
                        let a = Pos { story: at.story, path: at.path.clone(), off: 0 };
                        let b = Pos { story: at.story, path: at.path.clone(), off: p.len() };
                        s.sel = crate::Selection { anchor: a, focus: b };
                    }
                }
                type_text(s, &converted)?;
            }
        }
    }
    sel_result(s)
}

/// Convert digits in selection or paragraph to Myanmar numerals (၀-၉).
fn to_burmese_numerals_cmd(s: &mut Session, _v: &Value) -> CmdResult {
    if let Some(target) = get_target_text(s) {
        let converted = myanmar::convert_numerals(&target, true);
        if converted != target {
            if s.sel.is_collapsed() {
                let at = s.sel.focus.clone();
                if let Some(p) = s.doc.para_at(&at) {
                    let a = Pos { story: at.story, path: at.path.clone(), off: 0 };
                    let b = Pos { story: at.story, path: at.path.clone(), off: p.len() };
                    s.sel = crate::Selection { anchor: a, focus: b };
                }
            }
            type_text(s, &converted)?;
        }
    }
    sel_result(s)
}

/// Convert digits in selection or paragraph to Western numerals (0-9).
fn to_western_numerals_cmd(s: &mut Session, _v: &Value) -> CmdResult {
    if let Some(target) = get_target_text(s) {
        let converted = myanmar::convert_numerals(&target, false);
        if converted != target {
            if s.sel.is_collapsed() {
                let at = s.sel.focus.clone();
                if let Some(p) = s.doc.para_at(&at) {
                    let a = Pos { story: at.story, path: at.path.clone(), off: 0 };
                    let b = Pos { story: at.story, path: at.path.clone(), off: p.len() };
                    s.sel = crate::Selection { anchor: a, focus: b };
                }
            }
            type_text(s, &converted)?;
        }
    }
    sel_result(s)
}

/// Insert syllable breaks (ZWSP) for smooth justification and wrapping.
fn syllable_break_cmd(s: &mut Session, _v: &Value) -> CmdResult {
    if let Some(target) = get_target_text(s) {
        let converted = myanmar::insert_syllable_breaks(&target);
        if converted != target {
            if s.sel.is_collapsed() {
                let at = s.sel.focus.clone();
                if let Some(p) = s.doc.para_at(&at) {
                    let a = Pos { story: at.story, path: at.path.clone(), off: 0 };
                    let b = Pos { story: at.story, path: at.path.clone(), off: p.len() };
                    s.sel = crate::Selection { anchor: a, focus: b };
                }
            }
            type_text(s, &converted)?;
        }
    }
    sel_result(s)
}

/// Analyze Myanmar syllable and character counts.
fn syllable_count_cmd(s: &mut Session, _v: &Value) -> CmdResult {
    let text = get_target_text(s).unwrap_or_default();
    let stats = myanmar::analyze_myanmar_text(&text);
    Ok(json!({
        "syllables": stats.syllables,
        "consonants": stats.consonants,
        "vowelsAndMarks": stats.vowels_and_marks,
        "digits": stats.digits,
        "punctuation": stats.punctuation,
    }))
}

/// Look up a word in the Ornagai English-Myanmar dictionary.
fn dict_lookup_cmd(_s: &mut Session, v: &Value) -> CmdResult {
    let word = crate::p::req_str(v, "word")?;
    if let Some(item) = crate::dict::lookup(word) {
        Ok(json!({
            "found": true,
            "word": item.word,
            "pos": item.pos,
            "pronunciation": item.pronunciation,
            "definition": item.definition,
            "examples": item.examples,
            "plainText": item.plain_text,
        }))
    } else {
        let suggestions = crate::dict::suggest(word, 8);
        Ok(json!({
            "found": false,
            "word": word,
            "suggestions": suggestions,
        }))
    }
}

/// Autocomplete suggestions from the dictionary.
fn dict_suggest_cmd(_s: &mut Session, v: &Value) -> CmdResult {
    let prefix = crate::p::req_str(v, "prefix")?;
    let limit = crate::p::u64(v, "limit").unwrap_or(15) as usize;
    let suggestions = crate::dict::suggest(prefix, limit);
    Ok(json!({ "suggestions": suggestions }))
}

/// Insert dictionary definition at the current caret.
fn dict_insert_cmd(s: &mut Session, v: &Value) -> CmdResult {
    let text = if let Some(t) = crate::p::str(v, "text") {
        t.to_string()
    } else if let Some(w) = crate::p::str(v, "word") {
        if let Some(item) = crate::dict::lookup(w) {
            item.plain_text
        } else {
            w.to_string()
        }
    } else {
        return Err(crate::CmdError::Params("`word` or `text` required".into()));
    };
    type_text(s, &text)?;
    sel_result(s)
}

/// Insert current Myanmar Date (mmcalendar traditional calendar calculation).
fn insert_myanmar_date_cmd(s: &mut Session, v: &Value) -> CmdResult {
    let now = chrono::Local::now();
    let info = myanmar::calculate_myanmar_date(now.year(), now.month(), now.day());
    let full = crate::p::bool(v, "full").unwrap_or(true);
    let date_text = if full {
        info.formatted_full
    } else {
        info.formatted_short
    };
    type_text(s, &date_text)?;
    sel_result(s)
}

/// Get detailed Myanmar traditional calendar metadata for the current or given date.
fn myanmar_date_info_cmd(_s: &mut Session, v: &Value) -> CmdResult {
    let now = chrono::Local::now();
    let y = crate::p::u64(v, "year").unwrap_or(now.year() as u64) as i32;
    let m = crate::p::u64(v, "month").unwrap_or(now.month() as u64) as u32;
    let d = crate::p::u64(v, "day").unwrap_or(now.day() as u64) as u32;
    let info = myanmar::calculate_myanmar_date(y, m, d);
    Ok(json!(info))
}

/// Reverse dictionary lookup: Find English words for Myanmar term.
fn dict_reverse_lookup_cmd(_s: &mut Session, v: &Value) -> CmdResult {
    let query = crate::p::req_str(v, "query")?;
    let limit = crate::p::u64(v, "limit").unwrap_or(10) as usize;
    let results = crate::dict::reverse_search(query, limit);
    let items: Vec<Value> = results.into_iter().map(|(w, item)| {
        json!({
            "word": w,
            "pos": item.pos,
            "pronunciation": item.pronunciation,
            "definition": item.definition,
            "plainText": item.plain_text,
        })
    }).collect();
    Ok(json!({ "query": query, "results": items }))
}

/// Pronounce word via speech synthesis.
fn dict_speak_cmd(_s: &mut Session, v: &Value) -> CmdResult {
    let word = crate::p::req_str(v, "word")?;
    crate::dict::speak(word);
    Ok(json!({ "spoken": word }))
}

/// Detect whether target text is encoded in Unicode or Zawgyi.
fn myanmar_detect_encoding_cmd(s: &mut Session, _v: &Value) -> CmdResult {
    let text = get_target_text(s).unwrap_or_default();
    let encoding = myanmar::detect_encoding(&text);
    let enc_str = match encoding {
        myanmar::MyanmarEncoding::Unicode => "Unicode",
        myanmar::MyanmarEncoding::Zawgyi => "Zawgyi",
        myanmar::MyanmarEncoding::Unknown => "Unknown",
    };
    Ok(json!({ "encoding": enc_str }))
}

/// Clean excess zero-width spaces and duplicate spaces.
fn clean_myanmar_text_cmd(s: &mut Session, _v: &Value) -> CmdResult {
    if let Some(target) = get_target_text(s) {
        let cleaned = target
            .replace("\u{200B}\u{200B}", "\u{200B}")
            .replace("  ", " ");
        if cleaned != target {
            if s.sel.is_collapsed() {
                let at = s.sel.focus.clone();
                if let Some(p) = s.doc.para_at(&at) {
                    let a = Pos { story: at.story, path: at.path.clone(), off: 0 };
                    let b = Pos { story: at.story, path: at.path.clone(), off: p.len() };
                    s.sel = crate::Selection { anchor: a, focus: b };
                }
            }
            type_text(s, &cleaned)?;
        }
    }
    sel_result(s)
}

/// Convert integer into Myanmar spoken/written words.
pub fn myanmar_num_to_words(n: u64) -> String {
    if n == 0 {
        return "သုည".to_string();
    }
    let digits = ["", "တစ်", "နှစ်", "သုံး", "လေး", "ငါး", "ခြောက်", "ခုနစ်", "ရှစ်", "ကိုး"];
    let places = ["", "ဆယ်", "ရာ", "ထောင်", "သောင်း", "သိန်း", "သန်း", "ကုဋေ"];

    let s = n.to_string();
    let len = s.len();
    let chars: Vec<char> = s.chars().collect();
    let mut parts = Vec::new();

    for (i, &c) in chars.iter().enumerate() {
        let d = c.to_digit(10).unwrap_or(0) as usize;
        let place_idx = len - 1 - i;
        if d > 0 {
            if place_idx < places.len() {
                let place_name = places[place_idx];
                if place_name.is_empty() {
                    parts.push(digits[d].to_string());
                } else {
                    parts.push(format!("{}{}", digits[d], place_name));
                }
            } else {
                parts.push(digits[d].to_string());
            }
        }
    }

    if parts.is_empty() {
        "သုည".to_string()
    } else {
        parts.join(" ")
    }
}

/// Format number with thousand separators and Myanmar Kyat currency suffix.
pub fn format_currency_str(input: &str, burmese_digits: bool) -> String {
    let clean_digits: String = input
        .chars()
        .filter(|c| c.is_ascii_digit() || ('\u{1040}'..='\u{1049}').contains(c))
        .map(|c| match c {
            '\u{1040}'..='\u{1049}' => ((c as u32 - 0x1040) as u8 + b'0') as char,
            _ => c,
        })
        .collect();
    if clean_digits.is_empty() {
        return input.to_string();
    }
    let mut result = String::new();
    let chars: Vec<char> = clean_digits.chars().rev().collect();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }
    let formatted: String = result.chars().rev().collect();
    let final_num = if burmese_digits {
        myanmar::convert_numerals(&formatted, true)
    } else {
        formatted
    };
    format!("{final_num} ကျပ်")
}

/// Convert number in selection or paragraph to Myanmar words.
fn number_to_words_cmd(s: &mut Session, _v: &Value) -> CmdResult {
    if let Some(target) = get_target_text(s) {
        let clean_num: String = target.chars().filter_map(|c| {
            if c.is_ascii_digit() {
                Some(c)
            } else if ('\u{1040}'..='\u{1049}').contains(&c) {
                Some(((c as u32 - 0x1040) as u8 + b'0') as char)
            } else {
                None
            }
        }).collect();
        if let Ok(n) = clean_num.parse::<u64>() {
            let words = myanmar_num_to_words(n);
            if s.sel.is_collapsed() {
                let at = s.sel.focus.clone();
                if let Some(p) = s.doc.para_at(&at) {
                    let a = Pos { story: at.story, path: at.path.clone(), off: 0 };
                    let b = Pos { story: at.story, path: at.path.clone(), off: p.len() };
                    s.sel = crate::Selection { anchor: a, focus: b };
                }
            }
            type_text(s, &words)?;
        }
    }
    sel_result(s)
}

/// Format selection or paragraph as Myanmar currency (ကျပ်).
fn format_currency_cmd(s: &mut Session, v: &Value) -> CmdResult {
    let burmese = crate::p::bool(v, "burmese").unwrap_or(true);
    if let Some(target) = get_target_text(s) {
        let formatted = format_currency_str(&target, burmese);
        if formatted != target {
            if s.sel.is_collapsed() {
                let at = s.sel.focus.clone();
                if let Some(p) = s.doc.para_at(&at) {
                    let a = Pos { story: at.story, path: at.path.clone(), off: 0 };
                    let b = Pos { story: at.story, path: at.path.clone(), off: p.len() };
                    s.sel = crate::Selection { anchor: a, focus: b };
                }
            }
            type_text(s, &formatted)?;
        }
    }
    sel_result(s)
}

use wordcraft_doc::Pos;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        CommandSpec::new(
            "myanmar.zawgyiToUnicode",
            "Convert Zawgyi to Unicode (ဇော်ဂျီ မှ ယူနီကုဒ်)",
            "Review › Myanmar",
            zawgyi_to_unicode_cmd,
        ),
        CommandSpec::new(
            "myanmar.unicodeToZawgyi",
            "Convert Unicode to Zawgyi (ယူနီကုဒ် မှ ဇော်ဂျီ)",
            "Review › Myanmar",
            unicode_to_zawgyi_cmd,
        ),
        CommandSpec::new(
            "myanmar.toBurmeseNumerals",
            "Convert to Myanmar Numerals (မြန်မာဂဏန်း ၀-၉)",
            "Review › Myanmar",
            to_burmese_numerals_cmd,
        ),
        CommandSpec::new(
            "myanmar.toWesternNumerals",
            "Convert to Western Numerals (အင်္ဂလိပ်ဂဏန်း 0-9)",
            "Review › Myanmar",
            to_western_numerals_cmd,
        ),
        CommandSpec::new(
            "myanmar.syllableBreak",
            "Insert Syllable Break (စာလုံးခွဲ နေရာလွတ်)",
            "Review › Myanmar",
            syllable_break_cmd,
        ),
        CommandSpec::new(
            "myanmar.syllableCount",
            "Myanmar Syllable & Word Count",
            "Review › Myanmar",
            syllable_count_cmd,
        )
        .pure(),
        CommandSpec::new(
            "dict.lookup",
            "English-Myanmar Dictionary Lookup (အင်္ဂလိပ်-မြန်မာ အဘိဓာန် ရှာဖွေမှု)",
            "Review › Dictionary",
            dict_lookup_cmd,
        )
        .pure(),
        CommandSpec::new(
            "dict.suggest",
            "Dictionary Autocomplete Suggestions",
            "Review › Dictionary",
            dict_suggest_cmd,
        )
        .pure(),
        CommandSpec::new(
            "dict.insert",
            "Insert Dictionary Definition (အဘိဓာန် အဓိပ္ပာယ်ထည့်သွင်းမည်)",
            "Insert › Dictionary",
            dict_insert_cmd,
        ),
        CommandSpec::new(
            "myanmar.insertDate",
            "Insert Myanmar Date (ယနေ့ မြန်မာရက်စွဲ ထည့်မည်)",
            "Insert › Date",
            insert_myanmar_date_cmd,
        ),
        CommandSpec::new(
            "myanmar.cleanText",
            "Clean Myanmar Text & Spacing (စာသား သန့်စင်ပြင်ဆင်မည်)",
            "Review › Myanmar",
            clean_myanmar_text_cmd,
        ),
        CommandSpec::new(
            "myanmar.numberToWords",
            "Convert Numbers to Myanmar Words (ဂဏန်းမှ စာသားပြောင်းမည်)",
            "Review › Myanmar",
            number_to_words_cmd,
        ),
        CommandSpec::new(
            "myanmar.formatCurrency",
            "Format Myanmar Currency (ကျပ်ငွေ ပမာဏ ဖော်မတ်)",
            "Review › Myanmar",
            format_currency_cmd,
        ),
        CommandSpec::new(
            "myanmar.dateInfo",
            "Myanmar Calendar Information (မြန်မာပြက္ခဒိန် အချက်အလက်များ)",
            "Review › Myanmar",
            myanmar_date_info_cmd,
        )
        .pure(),
        CommandSpec::new(
            "myanmar.detectEncoding",
            "Detect Zawgyi vs Unicode Encoding (ဖောင့်စနစ် စစ်ဆေးမည်)",
            "Review › Myanmar",
            myanmar_detect_encoding_cmd,
        )
        .pure(),
        CommandSpec::new(
            "dict.reverseLookup",
            "Myanmar to English Reverse Dictionary Lookup (မြန်မာမှ အင်္ဂလိပ်သို့ အဘိဓာန် ရှာဖွေမှု)",
            "Review › Dictionary",
            dict_reverse_lookup_cmd,
        )
        .pure(),
        CommandSpec::new(
            "dict.speak",
            "Speak Word Pronunciation (စကားလုံး အသံထွက်ပြမည်)",
            "Review › Dictionary",
            dict_speak_cmd,
        )
        .pure(),
    ]
}
