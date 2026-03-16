use regex::Regex;
use std::sync::LazyLock;

static LOCALE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z]{2,3}(?:[_-][a-zA-Z]{4})?(?:[_-](?:[a-zA-Z]{2}|\d{3}))?$").unwrap()
});

/// Known ISO 639-1 two-letter language codes (subset for validation).
const ISO_639_1: &[&str] = &[
    "aa", "ab", "af", "ak", "am", "an", "ar", "as", "av", "ay", "az", "ba", "be", "bg", "bh", "bi",
    "bm", "bn", "bo", "br", "bs", "ca", "ce", "ch", "co", "cr", "cs", "cu", "cv", "cy", "da", "de",
    "dv", "dz", "ee", "el", "en", "eo", "es", "et", "eu", "fa", "ff", "fi", "fj", "fo", "fr", "fy",
    "ga", "gd", "gl", "gn", "gu", "gv", "ha", "he", "hi", "ho", "hr", "ht", "hu", "hy", "hz", "ia",
    "id", "ie", "ig", "ii", "ik", "in", "io", "is", "it", "iu", "iw", "ja", "ji", "jv", "jw", "ka",
    "kg", "ki", "kj", "kk", "kl", "km", "kn", "ko", "kr", "ks", "ku", "kv", "kw", "ky", "la", "lb",
    "lg", "li", "ln", "lo", "lt", "lu", "lv", "mg", "mh", "mi", "mk", "ml", "mn", "mo", "mr", "ms",
    "mt", "my", "na", "nb", "nd", "ne", "ng", "nl", "nn", "no", "nr", "nv", "ny", "oc", "oj", "om",
    "or", "os", "pa", "pi", "pl", "ps", "pt", "qu", "rm", "rn", "ro", "ru", "rw", "sa", "sc", "sd",
    "se", "sg", "sh", "si", "sk", "sl", "sm", "sn", "so", "sq", "sr", "ss", "st", "su", "sv", "sw",
    "ta", "te", "tg", "th", "ti", "tk", "tl", "tn", "to", "tr", "ts", "tt", "tw", "ty", "ug", "uk",
    "ur", "uz", "ve", "vi", "vo", "wa", "wo", "xh", "yi", "yo", "za", "zh", "zu",
];

/// BCP 47 normalization:
/// - underscore -> hyphen
/// - language part lowercase
/// - script part titlecase (4 chars)
/// - region part uppercase (2 chars) or numeric (3 digits)
pub fn normalize(code: &str) -> String {
    let code = code.replace('_', "-");
    let parts: Vec<&str> = code.split('-').collect();
    if parts.is_empty() {
        return code;
    }

    let mut result = parts[0].to_lowercase();

    for part in &parts[1..] {
        result.push('-');
        if part.len() == 4 {
            // Script: titlecase
            let mut chars = part.chars();
            if let Some(first) = chars.next() {
                result.push(first.to_uppercase().next().unwrap_or(first));
                for c in chars {
                    result.push(c.to_lowercase().next().unwrap_or(c));
                }
            }
        } else if part.len() == 2 && part.chars().all(|c| c.is_ascii_alphabetic()) {
            // Region: uppercase
            result.push_str(&part.to_uppercase());
        } else {
            // Numeric region or other
            result.push_str(part);
        }
    }

    result
}

/// Extract locale from a directory name.
/// Strips `values-` prefix (Android) and `.lproj` suffix (iOS).
/// Android uses `r` prefix for regions (e.g., `values-pt-rBR` -> `pt-BR`).
pub fn extract_from_path(dirname: &str) -> Option<String> {
    let mut s = dirname;

    // Strip Android values- prefix
    if let Some(rest) = s.strip_prefix("values-") {
        s = rest;
    }

    // Strip iOS .lproj suffix
    if let Some(rest) = s.strip_suffix(".lproj") {
        s = rest;
    }

    if s.is_empty() {
        return None;
    }

    // Handle Android region prefix: pt-rBR -> pt-BR
    let s = strip_android_region_prefix(s);
    let normalized = normalize(&s);

    if is_locale_code(&normalized) {
        Some(normalized)
    } else {
        None
    }
}

/// Strip Android `r` region prefix: `pt-rBR` -> `pt-BR`, `zh-rTW` -> `zh-TW`
fn strip_android_region_prefix(s: &str) -> String {
    let re = Regex::new(r"^([a-zA-Z]{2,3})[_-]r([A-Z]{2})$").unwrap();
    if let Some(caps) = re.captures(s) {
        format!("{}-{}", &caps[1], &caps[2])
    } else {
        s.to_string()
    }
}

/// Extract locale from a filename like `en.json`, `messages_pt_BR.properties`.
/// Returns None for non-locale filenames like `translation.json`.
pub fn extract_from_filename(filename: &str) -> Option<String> {
    // Strip extension
    let stem = filename
        .rsplit_once('.')
        .map(|(s, _)| s)
        .unwrap_or(filename);

    // Try the full stem as a locale code
    let normalized = normalize(stem);
    if is_locale_code(&normalized) {
        return Some(normalized);
    }

    // Try extracting locale after common prefixes: messages_en, strings_en, etc.
    for prefix in &["messages_", "strings_", "lang_", "app_"] {
        if let Some(rest) = stem.strip_prefix(prefix) {
            let normalized = normalize(rest);
            if is_locale_code(&normalized) {
                return Some(normalized);
            }
        }
    }

    // Try the last underscore/hyphen separated part
    if let Some(pos) = stem.rfind(['_', '-']) {
        let suffix = &stem[pos + 1..];
        let normalized = normalize(suffix);
        if is_locale_code(&normalized) {
            return Some(normalized);
        }

        // Try from the last two parts (for compound locales like pt_BR)
        let before = &stem[..pos];
        if let Some(pos2) = before.rfind(['_', '-']) {
            let compound = &stem[pos2 + 1..];
            let normalized = normalize(compound);
            if is_locale_code(&normalized) {
                return Some(normalized);
            }
        }
    }

    None
}

/// Normalize both codes and compare for fuzzy equality.
pub fn fuzzy_eq(a: &str, b: &str) -> bool {
    normalize(a) == normalize(b)
}

/// Returns true if the string looks like a valid locale code.
/// Valid: 2-3 letter language (must be known ISO 639-1 code for 2-letter,
/// or have script/region for 3-letter), optional 4-letter script, optional 2-letter region.
pub fn is_locale_code(s: &str) -> bool {
    if !LOCALE_RE.is_match(s) {
        return false;
    }

    // Extract the language part (before first separator)
    let normalized = s.replace('_', "-");
    let lang_part = normalized.split('-').next().unwrap_or("").to_lowercase();

    if lang_part.len() == 2 {
        // 2-letter codes: must be a known ISO 639-1 code
        ISO_639_1.contains(&lang_part.as_str())
    } else if lang_part.len() == 3 {
        // 3-letter codes: accept if they have a script or region qualifier,
        // or if the code itself is plausible (not a common English word).
        // For standalone 3-letter codes, check against a blocklist of
        // common directory/file names that are not locales.
        let has_qualifier = normalized.contains('-');
        if has_qualifier {
            true
        } else {
            // Known 3-letter locale codes (ISO 639-2/3 commonly used in i18n)
            const KNOWN_3_LETTER: &[&str] = &[
                "aar", "abk", "afr", "aka", "amh", "ara", "asm", "aze", "bak", "bel", "ben", "bod",
                "bos", "bre", "bul", "cat", "ces", "cmn", "cor", "cym", "dan", "deu", "div", "ell",
                "eng", "epo", "est", "eus", "fas", "fil", "fin", "fra", "fry", "gle", "glg", "grn",
                "guj", "hat", "hau", "heb", "hin", "hrv", "hun", "hye", "ibo", "ind", "isl", "ita",
                "jam", "jav", "jpn", "kan", "kat", "kaz", "khm", "kin", "kor", "kur", "lao", "lat",
                "lav", "lin", "lit", "ltz", "mal", "mar", "mkd", "mlg", "mlt", "mon", "mri", "msa",
                "mya", "nep", "nld", "nno", "nob", "nor", "nya", "oci", "ori", "orm", "pan", "pol",
                "por", "pus", "que", "roh", "ron", "run", "rus", "sin", "slk", "slv", "smo", "sna",
                "som", "sot", "spa", "sqi", "srp", "sun", "swa", "swe", "tam", "tat", "tel", "tgk",
                "tgl", "tha", "tir", "ton", "tsn", "tur", "ukr", "urd", "uzb", "vie", "wol", "xho",
                "yid", "yor", "zho", "zul",
            ];
            KNOWN_3_LETTER.contains(&lang_part.as_str())
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize() {
        assert_eq!(normalize("en"), "en");
        assert_eq!(normalize("EN"), "en");
        assert_eq!(normalize("pt_BR"), "pt-BR");
        assert_eq!(normalize("zh_Hans"), "zh-Hans");
        assert_eq!(normalize("zh_HANS"), "zh-Hans");
        assert_eq!(normalize("zh-Hant-TW"), "zh-Hant-TW");
    }

    #[test]
    fn test_extract_from_path() {
        assert_eq!(extract_from_path("en"), Some("en".to_string()));
        assert_eq!(extract_from_path("de"), Some("de".to_string()));
        assert_eq!(extract_from_path("values-en"), Some("en".to_string()));
        assert_eq!(
            extract_from_path("values-pt-rBR"),
            Some("pt-BR".to_string())
        );
        assert_eq!(extract_from_path("en.lproj"), Some("en".to_string()));
        assert_eq!(extract_from_path("src"), None);
        assert_eq!(extract_from_path("components"), None);
    }

    #[test]
    fn test_extract_from_filename() {
        assert_eq!(extract_from_filename("en.json"), Some("en".to_string()));
        assert_eq!(extract_from_filename("de.json"), Some("de".to_string()));
        assert_eq!(
            extract_from_filename("messages_pt_BR.properties"),
            Some("pt-BR".to_string())
        );
        assert_eq!(extract_from_filename("translation.json"), None);
        assert_eq!(extract_from_filename("index.js"), None);
    }

    #[test]
    fn test_fuzzy_eq() {
        assert!(fuzzy_eq("pt_BR", "pt-BR"));
        assert!(fuzzy_eq("zh_hans", "zh-Hans"));
        assert!(!fuzzy_eq("en", "de"));
    }

    #[test]
    fn test_is_locale_code() {
        assert!(is_locale_code("en"));
        assert!(is_locale_code("de"));
        assert!(is_locale_code("pt-BR"));
        assert!(is_locale_code("zh-Hans"));
        assert!(is_locale_code("zh-Hant-TW"));
        assert!(!is_locale_code("translation"));
        assert!(!is_locale_code("src"));
        assert!(!is_locale_code(""));
    }
}
