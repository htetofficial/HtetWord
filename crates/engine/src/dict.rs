//! Ornagai English-Myanmar Dictionary module for HtetWord.
//!
//! Fast StarDict reader (.idx + .dict) containing 111,232 headwords,
//! 127,930 senses, and 249,103 example sentences.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Global singleton dictionary instance for sub-millisecond lookups.
static DICTIONARY: OnceLock<Option<OrnagaiDict>> = OnceLock::new();

/// A parsed dictionary entry with structured fields.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct DictItem {
    pub word: String,
    pub pos: String,
    pub pronunciation: String,
    pub definition: String,
    pub examples: Vec<String>,
    pub raw_html: String,
    pub plain_text: String,
}

/// In-memory StarDict index and file reader.
pub struct OrnagaiDict {
    dict_path: PathBuf,
    entries: Vec<(String, u32, u32)>,
    lookup: HashMap<String, (u32, u32)>,
    reverse_entries: Vec<(String, Vec<u32>)>,
    reverse_lookup: HashMap<String, Vec<u32>>,
}

impl OrnagaiDict {
    /// Locate dictionary files across standard application paths.
    pub fn find_dict_files() -> Option<(PathBuf, PathBuf)> {
        // 1. Environment variable override
        if let Ok(dir) = std::env::var("HTETWORD_DICT_DIR") {
            let p = PathBuf::from(dir);
            let idx = p.join("ornagai-en-my.idx");
            let dict = p.join("ornagai-en-my.dict");
            if idx.exists() && dict.exists() {
                return Some((idx, dict));
            }
        }

        // 2. Path relative to executable
        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                let candidates = [
                    exe_dir.join("assets/dict"),
                    exe_dir.join("../assets/dict"),
                    exe_dir.join("../Resources/assets/dict"),
                    exe_dir.join("../../assets/dict"),
                    exe_dir.join("../../../assets/dict"),
                ];
                for base in candidates {
                    let idx = base.join("ornagai-en-my.idx");
                    let dict = base.join("ornagai-en-my.dict");
                    if idx.exists() && dict.exists() {
                        return Some((idx, dict));
                    }
                }
            }
        }

        // 3. Current working directory
        let local_base = Path::new("assets/dict");
        let local_idx = local_base.join("ornagai-en-my.idx");
        let local_dict = local_base.join("ornagai-en-my.dict");
        if local_idx.exists() && local_dict.exists() {
            return Some((local_idx, local_dict));
        }

        // 4. Source root fallback
        let manifest_base = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets/dict"));
        let manifest_idx = manifest_base.join("ornagai-en-my.idx");
        let manifest_dict = manifest_base.join("ornagai-en-my.dict");
        if manifest_idx.exists() && manifest_dict.exists() {
            return Some((manifest_idx, manifest_dict));
        }

        None
    }

    /// Load and index the dictionary from disk.
    pub fn load() -> Option<Self> {
        let (idx_path, dict_path) = Self::find_dict_files()?;
        let idx_bytes = std::fs::read(&idx_path).ok()?;
        
        let mut entries = Vec::with_capacity(112_000);
        let mut lookup = HashMap::with_capacity(112_000);

        let mut pos = 0;
        let len = idx_bytes.len();
        while pos < len {
            let null_idx = idx_bytes[pos..].iter().position(|&b| b == 0)?;
            let word_bytes = &idx_bytes[pos..pos + null_idx];
            let word = String::from_utf8_lossy(word_bytes).to_string();
            pos += null_idx + 1;

            if pos + 8 > len {
                break;
            }

            let offset = u32::from_be_bytes([idx_bytes[pos], idx_bytes[pos + 1], idx_bytes[pos + 2], idx_bytes[pos + 3]]);
            let size = u32::from_be_bytes([idx_bytes[pos + 4], idx_bytes[pos + 5], idx_bytes[pos + 6], idx_bytes[pos + 7]]);
            pos += 8;

            lookup.insert(word.to_ascii_lowercase(), (offset, size));
            entries.push((word, offset, size));
        }

        let mut reverse_entries = Vec::new();
        let mut reverse_lookup = HashMap::new();

        let rev_idx_path = idx_path.with_file_name("ornagai-my-en.idx");
        if let Ok(rev_bytes) = std::fs::read(&rev_idx_path) {
            if rev_bytes.len() >= 16 && &rev_bytes[0..8] == b"HTETMYEN" {
                let _version = u32::from_be_bytes([rev_bytes[8], rev_bytes[9], rev_bytes[10], rev_bytes[11]]);
                let count = u32::from_be_bytes([rev_bytes[12], rev_bytes[13], rev_bytes[14], rev_bytes[15]]) as usize;
                reverse_entries.reserve(count.min(250_000));
                reverse_lookup.reserve(count.min(250_000));

                let mut p = 16;
                let blen = rev_bytes.len();
                while p + 3 <= blen {
                    let term_len = u16::from_be_bytes([rev_bytes[p], rev_bytes[p + 1]]) as usize;
                    let num_ids = rev_bytes[p + 2] as usize;
                    p += 3;
                    if p + term_len + num_ids * 4 > blen {
                        break;
                    }
                    if let Ok(term) = std::str::from_utf8(&rev_bytes[p..p + term_len]) {
                        let mut ids = Vec::with_capacity(num_ids);
                        let mut id_pos = p + term_len;
                        for _ in 0..num_ids {
                            let id = u32::from_be_bytes([
                                rev_bytes[id_pos],
                                rev_bytes[id_pos + 1],
                                rev_bytes[id_pos + 2],
                                rev_bytes[id_pos + 3],
                            ]);
                            ids.push(id);
                            id_pos += 4;
                        }
                        reverse_lookup.insert(term.to_string(), ids.clone());
                        reverse_entries.push((term.to_string(), ids));
                    }
                    p += term_len + num_ids * 4;
                }
            }
        }

        Some(OrnagaiDict {
            dict_path,
            entries,
            lookup,
            reverse_entries,
            reverse_lookup,
        })
    }

    /// Look up an English word and parse its Myanmar definition.
    pub fn lookup(&self, word: &str) -> Option<DictItem> {
        let clean = word.trim().to_ascii_lowercase();
        if clean.is_empty() {
            return None;
        }

        let &(offset, size) = self.lookup.get(&clean)?;
        let mut file = File::open(&self.dict_path).ok()?;
        file.seek(SeekFrom::Start(offset as u64)).ok()?;

        let mut buffer = vec![0u8; size as usize];
        file.read_exact(&mut buffer).ok()?;

        let raw_html = String::from_utf8_lossy(&buffer).to_string();
        Some(Self::parse_html(word, &raw_html))
    }

    /// Autocomplete suggestions for a search prefix.
    pub fn suggest(&self, prefix: &str, limit: usize) -> Vec<String> {
        let clean = prefix.trim().to_ascii_lowercase();
        if clean.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();
        // Priority 1: Exact matches or prefix matches
        for (word, _, _) in &self.entries {
            if word.to_ascii_lowercase().starts_with(&clean) {
                results.push(word.clone());
                if results.len() >= limit {
                    return results;
                }
            }
        }

        // Priority 2: Substring matches if prefix matches are few
        if results.len() < limit && clean.len() >= 3 {
            for (word, _, _) in &self.entries {
                if !results.contains(word) && word.to_ascii_lowercase().contains(&clean) {
                    results.push(word.clone());
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        results
    }

    /// Total number of indexed words.
    pub fn total_words(&self) -> usize {
        self.entries.len()
    }

    /// Get parsed dictionary item by internal index.
    pub fn get_by_index(&self, index: usize) -> Option<DictItem> {
        let (word, offset, size) = self.entries.get(index)?;
        let mut file = File::open(&self.dict_path).ok()?;
        file.seek(SeekFrom::Start(*offset as u64)).ok()?;

        let mut buffer = vec![0u8; *size as usize];
        file.read_exact(&mut buffer).ok()?;

        let raw_html = String::from_utf8_lossy(&buffer).to_string();
        Some(Self::parse_html(word, &raw_html))
    }

    /// Reverse lookup: Search English words given a Myanmar term or phrase.
    /// Returns a list of (English headword, DictItem) matching the query.
    pub fn reverse_search(&self, query: &str, limit: usize) -> Vec<(String, DictItem)> {
        let clean = query.trim();
        if clean.is_empty() {
            return Vec::new();
        }

        let mut matched_indices = Vec::new();

        // 1. Exact match in reverse index
        if let Some(ids) = self.reverse_lookup.get(clean) {
            for &id in ids {
                if !matched_indices.contains(&id) {
                    matched_indices.push(id);
                }
                if matched_indices.len() >= limit {
                    break;
                }
            }
        }

        // 2. Prefix / containment match in reverse index
        if matched_indices.len() < limit {
            for (term, ids) in &self.reverse_entries {
                if term.starts_with(clean) || clean.starts_with(term) {
                    for &id in ids {
                        if !matched_indices.contains(&id) {
                            matched_indices.push(id);
                        }
                        if matched_indices.len() >= limit {
                            break;
                        }
                    }
                }
                if matched_indices.len() >= limit {
                    break;
                }
            }
        }

        // 3. Substring match if still below limit and query has at least 2 chars
        if matched_indices.len() < limit && clean.chars().count() >= 2 {
            for (term, ids) in &self.reverse_entries {
                if term.contains(clean) {
                    for &id in ids {
                        if !matched_indices.contains(&id) {
                            matched_indices.push(id);
                        }
                        if matched_indices.len() >= limit {
                            break;
                        }
                    }
                }
                if matched_indices.len() >= limit {
                    break;
                }
            }
        }

        // Map matched indices to DictItem
        let mut results = Vec::with_capacity(matched_indices.len());
        for id in matched_indices {
            if let Some(item) = self.get_by_index(id as usize) {
                results.push((item.word.clone(), item));
            }
        }

        results
    }

    /// Parse StarDict HTML into a structured `DictItem`.
    fn parse_html(original_word: &str, html: &str) -> DictItem {
        let mut pos = String::new();
        let mut definition = String::new();
        let mut examples = Vec::new();

        // Extract <b>...</b> for part of speech
        if let Some(start) = html.find("<b>") {
            if let Some(end) = html[start + 3..].find("</b>") {
                pos = html[start + 3..start + 3 + end].trim().to_string();
            }
        }

        // Extract pronunciation [ ... ] or <i>/.../</i>
        let mut pron_parts = Vec::new();
        if let Some(start) = html.find("<i>/") {
            if let Some(end) = html[start + 3..].find("/</i>") {
                pron_parts.push(html[start + 3..start + 3 + end + 1].trim().to_string());
            }
        }
        if let Some(start) = html.find('[') {
            if let Some(end) = html[start..].find(']') {
                pron_parts.push(html[start..=start + end].trim().to_string());
            }
        }
        let pronunciation = pron_parts.join(" ");

        // Process line by line separated by <br> or newline
        let lines: Vec<&str> = html.split("<br>").collect();
        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Check if this is an example sentence (starts with <i>• or contains bullet)
            if trimmed.contains('•') || (trimmed.starts_with("<i>") && trimmed.contains('.')) {
                let clean_ex = strip_tags(trimmed);
                if !clean_ex.is_empty() {
                    examples.push(clean_ex);
                }
            } else if !trimmed.starts_with("<b>") && !trimmed.starts_with("<i>/") {
                // Main definition line
                let clean_def = strip_tags(trimmed);
                if !clean_def.is_empty() && !clean_def.starts_with('[') {
                    if !definition.is_empty() {
                        definition.push('\n');
                    }
                    definition.push_str(&clean_def);
                }
            }
        }

        // Generate clean plain text format for easy insertion
        let mut plain_text = format!("{original_word}");
        if !pos.is_empty() {
            plain_text.push_str(&format!(" ({pos})"));
        }
        if !pronunciation.is_empty() {
            plain_text.push_str(&format!(" {pronunciation}"));
        }
        if !definition.is_empty() {
            plain_text.push_str(&format!(" : {definition}"));
        }

        DictItem {
            word: original_word.to_string(),
            pos,
            pronunciation,
            definition,
            examples,
            raw_html: html.to_string(),
            plain_text,
        }
    }
}

/// Strip basic HTML tags from dictionary string.
fn strip_tags(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_tag = false;
    for c in input.chars() {
        if c == '<' {
            in_tag = true;
        } else if c == '>' {
            in_tag = false;
        } else if !in_tag {
            output.push(c);
        }
    }
    output
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .trim()
        .to_string()
}

/// Check if query string contains Myanmar characters.
#[inline]
pub fn is_myanmar_text(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(c, '\u{1000}'..='\u{109F}' | '\u{A9E0}'..='\u{A9FF}' | '\u{AA60}'..='\u{AA7F}')
    })
}

/// Play speech audio for an English word using system TTS.
pub fn speak(word: &str) {
    let clean: String = word
        .chars()
        .filter(|c| c.is_alphabetic() || *c == ' ' || *c == '-')
        .collect();
    let trimmed = clean.trim();
    if trimmed.is_empty() {
        return;
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("say")
            .arg(trimmed)
            .spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let script = format!("(New-Object -ComObject SAPI.SpVoice).Speak('{}')", trimmed.replace('\'', ""));
        let _ = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("spd-say")
            .arg(trimmed)
            .spawn()
            .or_else(|_| std::process::Command::new("espeak").arg(trimmed).spawn());
    }
    #[cfg(target_os = "android")]
    {
        // Android TTS invocation stub
    }
}

/// Global lookup helper function.
pub fn lookup(word: &str) -> Option<DictItem> {
    let dict = DICTIONARY.get_or_init(OrnagaiDict::load);
    dict.as_ref()?.lookup(word)
}

/// Global reverse search helper function (Myanmar -> English).
pub fn reverse_search(query: &str, limit: usize) -> Vec<(String, DictItem)> {
    let dict = DICTIONARY.get_or_init(OrnagaiDict::load);
    match dict.as_ref() {
        Some(d) => d.reverse_search(query, limit),
        None => Vec::new(),
    }
}

/// Bidirectional lookup: auto-detects Myanmar or English query.
pub fn lookup_auto(query: &str, limit: usize) -> (bool, Option<DictItem>, Vec<(String, DictItem)>) {
    let q = query.trim();
    if is_myanmar_text(q) {
        let matches = reverse_search(q, limit);
        let first_item = matches.first().map(|(_, item)| item.clone());
        (true, first_item, matches)
    } else {
        let item = lookup(q);
        (false, item, Vec::new())
    }
}

/// Global suggest helper function.
pub fn suggest(prefix: &str, limit: usize) -> Vec<String> {
    let dict = DICTIONARY.get_or_init(OrnagaiDict::load);
    match dict.as_ref() {
        Some(d) => d.suggest(prefix, limit),
        None => Vec::new(),
    }
}

/// Check if the dictionary is loaded and ready.
pub fn is_available() -> bool {
    let dict = DICTIONARY.get_or_init(OrnagaiDict::load);
    dict.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dict_lookup() {
        if !is_available() {
            println!("Dictionary files not found in standard paths for test.");
            return;
        }
        let res = lookup("love");
        assert!(res.is_some(), "Word 'love' should be found");
        let item = res.unwrap();
        assert_eq!(item.word.to_lowercase(), "love");
        assert!(!item.definition.is_empty(), "Definition should not be empty");

        let suggestions = suggest("appl", 5);
        assert!(!suggestions.is_empty(), "Should suggest words for 'appl'");
        assert!(suggestions.iter().any(|s| s.starts_with("appl")));

        // Test reverse lookup
        let rev = reverse_search("စာအုပ်", 5);
        assert!(!rev.is_empty(), "Reverse search for 'စာအုပ်' should return results");
        assert!(rev.iter().any(|(w, _)| w == "book"), "Should match 'book' for 'စာအုပ်'");

        let rev_love = reverse_search("မေတ္တာ", 5);
        assert!(!rev_love.is_empty(), "Reverse search for 'မေတ္တာ' should return results");
        assert!(rev_love.iter().any(|(w, _)| w == "love"), "Should match 'love' for 'မေတ္တာ'");

        assert!(is_myanmar_text("စာအုပ်"));
        assert!(!is_myanmar_text("book"));
    }
}

