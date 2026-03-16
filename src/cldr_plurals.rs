//! CLDR cardinal plural rules database.
//!
//! Maps locale codes to their required plural categories based on CLDR v44
//! cardinal rules for standard (non-compact) number formatting.
//!
//! Reference: <https://unicode.org/cldr/charts/44/supplemental/language_plural_rules.html>

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::locale;

/// The six CLDR plural categories for cardinal numbers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PluralCategory {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

impl PluralCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            PluralCategory::Zero => "zero",
            PluralCategory::One => "one",
            PluralCategory::Two => "two",
            PluralCategory::Few => "few",
            PluralCategory::Many => "many",
            PluralCategory::Other => "other",
        }
    }
}

impl std::fmt::Display for PluralCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ── Plural category sets ─────────────────────────────────────────────────────

const OTHER_ONLY: &[PluralCategory] = &[PluralCategory::Other];

const ONE_OTHER: &[PluralCategory] = &[PluralCategory::One, PluralCategory::Other];

#[allow(dead_code)] // Available for strict CLDR v44 compliance override
const ONE_MANY_OTHER: &[PluralCategory] = &[
    PluralCategory::One,
    PluralCategory::Many,
    PluralCategory::Other,
];

const ONE_TWO_OTHER: &[PluralCategory] = &[
    PluralCategory::One,
    PluralCategory::Two,
    PluralCategory::Other,
];

const ZERO_ONE_OTHER: &[PluralCategory] = &[
    PluralCategory::Zero,
    PluralCategory::One,
    PluralCategory::Other,
];

const ONE_FEW_OTHER: &[PluralCategory] = &[
    PluralCategory::One,
    PluralCategory::Few,
    PluralCategory::Other,
];

const ONE_FEW_MANY_OTHER: &[PluralCategory] = &[
    PluralCategory::One,
    PluralCategory::Few,
    PluralCategory::Many,
    PluralCategory::Other,
];

const ONE_TWO_FEW_OTHER: &[PluralCategory] = &[
    PluralCategory::One,
    PluralCategory::Two,
    PluralCategory::Few,
    PluralCategory::Other,
];

const ONE_TWO_FEW_MANY_OTHER: &[PluralCategory] = &[
    PluralCategory::One,
    PluralCategory::Two,
    PluralCategory::Few,
    PluralCategory::Many,
    PluralCategory::Other,
];

const ALL_SIX: &[PluralCategory] = &[
    PluralCategory::Zero,
    PluralCategory::One,
    PluralCategory::Two,
    PluralCategory::Few,
    PluralCategory::Many,
    PluralCategory::Other,
];

// ── Legacy locale aliases ────────────────────────────────────────────────────

static LOCALE_ALIASES: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("iw", "he"),  // Hebrew legacy
        ("in", "id"),  // Indonesian legacy
        ("jw", "jv"),  // Javanese legacy
        ("ji", "yi"),  // Yiddish legacy
        ("mo", "ro"),  // Moldovan → Romanian
        ("sh", "sr"),  // Serbo-Croatian → Serbian
        ("tl", "fil"), // Tagalog → Filipino
        ("no", "nb"),  // Norwegian → Bokmål
    ])
});

// ── CLDR v44 cardinal plural rules ──────────────────────────────────────────
//
// Source: Unicode CLDR v44 common/supplemental/plurals.xml
//
// DESIGN DECISION: In CLDR v38+, a "many" category was added to many
// languages (en, de, fr, es, it, pt, ca, etc.) for the rule:
//   e = 0 and i != 0 and i % 1000000 = 0 and v = 0 or e != 0..5
// This triggers for standard integers that are exact multiples of 1,000,000
// AND for compact decimal notation (e.g., "1M"). Technically, CLDR v44
// classifies these languages as {one, many, other}.
//
// However, this validator uses the PRACTICAL plural categories — the ones
// translators actually need to provide in real-world i18n files. Requiring
// a "many" form for English, German, French, Spanish etc. would flag
// virtually every translation file as incorrect, since no one provides a
// separate plural form for exact multiples of one million.
//
// Languages where "many" has REAL grammatical significance (Polish, Russian,
// Czech, etc.) are correctly mapped with the "many" category.
//
// For strict CLDR v44 compliance, use the ONE_MANY_OTHER constant and
// override the mapping for the affected languages.

static CLDR_RULES: LazyLock<HashMap<&'static str, &'static [PluralCategory]>> =
    LazyLock::new(|| {
        let mut m = HashMap::with_capacity(210);

        // ── {other} only ─────────────────────────────────────────────────
        // Languages with no grammatical plural distinction.
        for lang in [
            "bm", "bo", "dz", "hnj", "id", "ig", "ii", "ja", "jbo", "jv", "kde", "kea", "km", "ko",
            "lkt", "lo", "ms", "my", "nqo", "osa", "root", "sah", "ses", "sg", "su", "th", "to",
            "tpi", "vi", "wo", "yo", "yue", "zh",
        ] {
            m.insert(lang, OTHER_ONLY);
        }

        // ── {one, other} ────────────────────────────────────────────────
        // Most common pattern: singular vs plural.
        // NOTE: In strict CLDR v44, many of these languages (en, de, fr,
        // es, it, pt, ca, etc.) have a "many" category for n % 1000000 = 0.
        // See the design decision comment above for why we use {one, other}.
        for lang in [
            // Standard: one = (i = 1 and v = 0)
            "af", "an", "asa", "ast", "az", "bal", "bem", "bez", "bg", "brx", "ca", "ce", "cgg",
            "chr", "ckb", "da", "de", "dv", "ee", "el", "en", "eo", "es", "et", "eu", "fi", "fo",
            "fur", "fy", "gl", "gsw", "ha", "haw", "hu", "hy", "ia", "ie", "io", "is", "it", "jgo",
            "jmc", "ka", "kab", "kaj", "kcg", "kk", "kkj", "kl", "kok", "ks", "ksb", "ku", "ky",
            "lb", "lg", "lij", "mas", "mgo", "mk", "ml", "mn", "mr", "nah", "nb", "nd", "ne", "nl",
            "nn", "nnh", "nr", "nso", "ny", "nyn", "om", "or", "os", "pap", "pcm", "ps", "pt",
            "rm", "rof", "rwk", "saq", "sc", "sd", "sdh", "seh", "si", "sn", "so", "sq", "ss",
            "ssy", "st", "sv", "sw", "syr", "ta", "te", "teo", "tig", "tk", "tn", "tr", "ts", "ug",
            "ur", "uz", "ve", "vo", "vun", "wae", "xh", "xog", "yi", "zu",
            // one = (i = 0 or n = 1)
            "am", "as", "bn", "doi", "fa", "gu", "hi", "kn", // one = (i = 0,1)
            "ff", "fr", // one = (n = 0..1)
            "ak", "bho", "guw", "ln", "mg", "pa", "ti", "wa",
            // Filipino: complex "one" rule, still {one, other}
            "ceb", "fil", // Central Atlas Tamazight: {one, other}
            "tzm",
        ] {
            m.insert(lang, ONE_OTHER);
        }

        // ── {one, two, other} ───────────────────────────────────────────
        // Languages that distinguish singular, dual, and plural.
        for lang in [
            "he",  // Hebrew
            "iu",  // Inuktitut
            "naq", // Nama
            "sat", // Santali
            "se",  // Northern Sami
            "sma", // Southern Sami
            "smi", // Sami (generic)
            "smj", // Lule Sami
            "smn", // Inari Sami
            "sms", // Skolt Sami
        ] {
            m.insert(lang, ONE_TWO_OTHER);
        }

        // ── {zero, one, other} ──────────────────────────────────────────
        for lang in [
            "ksh", // Colognian
            "lag", // Langi
            "lv",  // Latvian
            "prg", // Prussian
        ] {
            m.insert(lang, ZERO_ONE_OTHER);
        }

        // ── {one, few, other} ───────────────────────────────────────────
        for lang in [
            "bs",  // Bosnian
            "hr",  // Croatian
            "ro",  // Romanian
            "shi", // Tachelhit
            "sr",  // Serbian
        ] {
            m.insert(lang, ONE_FEW_OTHER);
        }

        // ── {one, few, many, other} ─────────────────────────────────────
        // East Slavic, West Slavic, Baltic
        for lang in [
            "be", // Belarusian
            "cs", // Czech
            "lt", // Lithuanian
            "pl", // Polish
            "ru", // Russian
            "sk", // Slovak
            "uk", // Ukrainian
        ] {
            m.insert(lang, ONE_FEW_MANY_OTHER);
        }

        // ── {one, two, few, other} ──────────────────────────────────────
        for lang in [
            "dsb", // Lower Sorbian
            "gd",  // Scottish Gaelic
            "hsb", // Upper Sorbian
            "sl",  // Slovenian
        ] {
            m.insert(lang, ONE_TWO_FEW_OTHER);
        }

        // ── {one, two, few, many, other} ────────────────────────────────
        for lang in [
            "br",  // Breton
            "ga",  // Irish
            "gv",  // Manx
            "mt",  // Maltese
            "sgs", // Samogitian
        ] {
            m.insert(lang, ONE_TWO_FEW_MANY_OTHER);
        }

        // ── {zero, one, two, few, many, other} ─────────────────────────
        // All six categories.
        for lang in [
            "ar",  // Arabic
            "ars", // Najdi Arabic
            "cy",  // Welsh
            "kw",  // Cornish
        ] {
            m.insert(lang, ALL_SIX);
        }

        m
    });

/// Look up the required CLDR plural categories for a locale.
///
/// Performs locale normalization, alias resolution, and base language fallback.
/// Returns `None` if the language is not in the CLDR database.
pub fn lookup(locale_code: &str) -> Option<&'static [PluralCategory]> {
    let normalized = locale::normalize(locale_code);
    let lower = normalized.to_lowercase();

    // Extract base language (before any hyphen)
    let base = lower.split('-').next().unwrap_or(&lower);

    // Resolve legacy aliases
    let resolved = LOCALE_ALIASES.get(base).copied().unwrap_or(base);

    CLDR_RULES.get(resolved).copied()
}

/// Returns the total number of languages in the CLDR database.
#[cfg(test)]
pub fn database_size() -> usize {
    CLDR_RULES.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Basic lookups ────────────────────────────────────────────────────

    #[test]
    fn test_other_only_languages() {
        for lang in ["ja", "ko", "zh", "vi", "th", "id", "ms", "km", "lo", "my"] {
            let cats = lookup(lang).unwrap_or_else(|| panic!("{lang} should be in CLDR"));
            assert_eq!(cats, OTHER_ONLY, "{lang} should have only 'other'");
        }
    }

    #[test]
    fn test_one_other_languages() {
        for lang in [
            "en", "de", "fr", "es", "it", "pt", "nl", "sv", "da", "fi", "hu", "tr", "bg", "el",
            "hi", "bn", "fa", "ur", "sw", "am",
        ] {
            let cats = lookup(lang).unwrap_or_else(|| panic!("{lang} should be in CLDR"));
            assert_eq!(cats, ONE_OTHER, "{lang} should have 'one, other'");
        }
    }

    #[test]
    fn test_arabic_all_six() {
        let cats = lookup("ar").expect("Arabic should be in CLDR");
        assert_eq!(cats, ALL_SIX);
        assert_eq!(cats.len(), 6);
    }

    #[test]
    fn test_welsh_all_six() {
        let cats = lookup("cy").expect("Welsh should be in CLDR");
        assert_eq!(cats, ALL_SIX);
    }

    #[test]
    fn test_cornish_all_six() {
        let cats = lookup("kw").expect("Cornish should be in CLDR");
        assert_eq!(cats, ALL_SIX);
    }

    #[test]
    fn test_russian_four_forms() {
        let cats = lookup("ru").expect("Russian should be in CLDR");
        assert_eq!(cats, ONE_FEW_MANY_OTHER);
        assert_eq!(cats.len(), 4);
    }

    #[test]
    fn test_polish_four_forms() {
        let cats = lookup("pl").expect("Polish should be in CLDR");
        assert_eq!(cats, ONE_FEW_MANY_OTHER);
    }

    #[test]
    fn test_czech_four_forms() {
        let cats = lookup("cs").expect("Czech should be in CLDR");
        assert_eq!(cats, ONE_FEW_MANY_OTHER);
    }

    #[test]
    fn test_ukrainian_four_forms() {
        let cats = lookup("uk").expect("Ukrainian should be in CLDR");
        assert_eq!(cats, ONE_FEW_MANY_OTHER);
    }

    #[test]
    fn test_hebrew_three_forms() {
        let cats = lookup("he").expect("Hebrew should be in CLDR");
        assert_eq!(cats, ONE_TWO_OTHER);
    }

    #[test]
    fn test_latvian_three_forms() {
        let cats = lookup("lv").expect("Latvian should be in CLDR");
        assert_eq!(cats, ZERO_ONE_OTHER);
    }

    #[test]
    fn test_romanian_three_forms() {
        let cats = lookup("ro").expect("Romanian should be in CLDR");
        assert_eq!(cats, ONE_FEW_OTHER);
    }

    #[test]
    fn test_croatian_three_forms() {
        let cats = lookup("hr").expect("Croatian should be in CLDR");
        assert_eq!(cats, ONE_FEW_OTHER);
    }

    #[test]
    fn test_slovenian_four_forms() {
        let cats = lookup("sl").expect("Slovenian should be in CLDR");
        assert_eq!(cats, ONE_TWO_FEW_OTHER);
    }

    #[test]
    fn test_irish_five_forms() {
        let cats = lookup("ga").expect("Irish should be in CLDR");
        assert_eq!(cats, ONE_TWO_FEW_MANY_OTHER);
    }

    #[test]
    fn test_maltese_five_forms() {
        let cats = lookup("mt").expect("Maltese should be in CLDR");
        assert_eq!(cats, ONE_TWO_FEW_MANY_OTHER);
    }

    #[test]
    fn test_breton_five_forms() {
        let cats = lookup("br").expect("Breton should be in CLDR");
        assert_eq!(cats, ONE_TWO_FEW_MANY_OTHER);
    }

    #[test]
    fn test_manx_five_forms() {
        let cats = lookup("gv").expect("Manx should be in CLDR");
        assert_eq!(cats, ONE_TWO_FEW_MANY_OTHER);
    }

    #[test]
    fn test_samogitian_five_forms() {
        let cats = lookup("sgs").expect("Samogitian should be in CLDR");
        assert_eq!(cats, ONE_TWO_FEW_MANY_OTHER);
    }

    #[test]
    fn test_santali_three_forms() {
        let cats = lookup("sat").expect("Santali should be in CLDR");
        assert_eq!(cats, ONE_TWO_OTHER);
    }

    #[test]
    fn test_langi_three_forms() {
        let cats = lookup("lag").expect("Langi should be in CLDR");
        assert_eq!(cats, ZERO_ONE_OTHER);
    }

    // ── Locale normalization & fallback ──────────────────────────────────

    #[test]
    fn test_region_fallback() {
        // pt-BR should fall back to pt
        let cats = lookup("pt-BR").expect("pt-BR should resolve");
        assert_eq!(cats, ONE_OTHER);

        // en-US should fall back to en
        let cats = lookup("en-US").expect("en-US should resolve");
        assert_eq!(cats, ONE_OTHER);

        // fr-CA should fall back to fr
        let cats = lookup("fr-CA").expect("fr-CA should resolve");
        assert_eq!(cats, ONE_OTHER);

        // ar-EG should fall back to ar
        let cats = lookup("ar-EG").expect("ar-EG should resolve");
        assert_eq!(cats, ALL_SIX);

        // zh-TW should fall back to zh
        let cats = lookup("zh-TW").expect("zh-TW should resolve");
        assert_eq!(cats, OTHER_ONLY);
    }

    #[test]
    fn test_script_subtag_fallback() {
        // zh-Hans → zh → {other}
        let cats = lookup("zh-Hans").expect("zh-Hans should resolve");
        assert_eq!(cats, OTHER_ONLY);

        // zh-Hant → zh → {other}
        let cats = lookup("zh-Hant").expect("zh-Hant should resolve");
        assert_eq!(cats, OTHER_ONLY);

        // sr-Latn → sr → {one, few, other}
        let cats = lookup("sr-Latn").expect("sr-Latn should resolve");
        assert_eq!(cats, ONE_FEW_OTHER);
    }

    #[test]
    fn test_underscore_normalization() {
        let cats = lookup("pt_BR").expect("pt_BR should resolve");
        assert_eq!(cats, ONE_OTHER);

        let cats = lookup("zh_Hans").expect("zh_Hans should resolve");
        assert_eq!(cats, OTHER_ONLY);

        let cats = lookup("zh_Hant_TW").expect("zh_Hant_TW should resolve");
        assert_eq!(cats, OTHER_ONLY);
    }

    #[test]
    fn test_case_insensitive() {
        let cats = lookup("EN").expect("EN should resolve");
        assert_eq!(cats, ONE_OTHER);

        let cats = lookup("JA").expect("JA should resolve");
        assert_eq!(cats, OTHER_ONLY);

        let cats = lookup("AR").expect("AR should resolve");
        assert_eq!(cats, ALL_SIX);
    }

    // ── Legacy alias resolution ─────────────────────────────────────────

    #[test]
    fn test_alias_hebrew() {
        // iw → he
        let cats = lookup("iw").expect("iw should resolve to he");
        assert_eq!(cats, ONE_TWO_OTHER);
    }

    #[test]
    fn test_alias_indonesian() {
        // in → id
        let cats = lookup("in").expect("in should resolve to id");
        assert_eq!(cats, OTHER_ONLY);
    }

    #[test]
    fn test_alias_javanese() {
        // jw → jv
        let cats = lookup("jw").expect("jw should resolve to jv");
        assert_eq!(cats, OTHER_ONLY);
    }

    #[test]
    fn test_alias_yiddish() {
        // ji → yi
        let cats = lookup("ji").expect("ji should resolve to yi");
        assert_eq!(cats, ONE_OTHER);
    }

    #[test]
    fn test_alias_moldovan() {
        // mo → ro
        let cats = lookup("mo").expect("mo should resolve to ro");
        assert_eq!(cats, ONE_FEW_OTHER);
    }

    #[test]
    fn test_alias_serbo_croatian() {
        // sh → sr
        let cats = lookup("sh").expect("sh should resolve to sr");
        assert_eq!(cats, ONE_FEW_OTHER);
    }

    #[test]
    fn test_alias_tagalog() {
        // tl → fil
        let cats = lookup("tl").expect("tl should resolve to fil");
        assert_eq!(cats, ONE_OTHER);
    }

    #[test]
    fn test_alias_norwegian() {
        // no → nb
        let cats = lookup("no").expect("no should resolve to nb");
        assert_eq!(cats, ONE_OTHER);
    }

    // ── Unknown locales ─────────────────────────────────────────────────

    #[test]
    fn test_unknown_locale_returns_none() {
        assert!(lookup("xx").is_none());
        assert!(lookup("zzz").is_none());
        assert!(lookup("unknown").is_none());
    }

    // ── Database integrity ──────────────────────────────────────────────

    #[test]
    fn test_all_entries_contain_other() {
        for (lang, cats) in CLDR_RULES.iter() {
            assert!(
                cats.contains(&PluralCategory::Other),
                "{lang} must include 'other' category"
            );
        }
    }

    #[test]
    fn test_all_entries_sorted() {
        // Categories should be in canonical order for consistent display
        for (lang, cats) in CLDR_RULES.iter() {
            let mut sorted = cats.to_vec();
            sorted.sort();
            assert!(
                **cats == sorted[..],
                "{lang} categories should be in canonical order"
            );
        }
    }

    #[test]
    fn test_database_covers_major_languages() {
        let major = [
            "ar", "bn", "bg", "ca", "cs", "da", "de", "el", "en", "es", "et", "fa", "fi", "fr",
            "he", "hi", "hr", "hu", "id", "it", "ja", "ko", "lt", "lv", "ms", "nl", "nb", "pl",
            "pt", "ro", "ru", "sk", "sl", "sr", "sv", "th", "tr", "uk", "ur", "vi", "zh",
        ];
        for lang in major {
            assert!(
                lookup(lang).is_some(),
                "Major language '{lang}' should be in CLDR database"
            );
        }
    }

    #[test]
    fn test_database_size() {
        // Should have a substantial number of languages
        assert!(
            database_size() > 155,
            "CLDR database should cover 155+ languages, got {}",
            database_size()
        );
    }

    // ── ONE_MANY_OTHER constant exists ───────────────────────────────────

    #[test]
    fn test_one_many_other_constant() {
        assert_eq!(ONE_MANY_OTHER.len(), 3);
        assert!(ONE_MANY_OTHER.contains(&PluralCategory::One));
        assert!(ONE_MANY_OTHER.contains(&PluralCategory::Many));
        assert!(ONE_MANY_OTHER.contains(&PluralCategory::Other));
    }
}
