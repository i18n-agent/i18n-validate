use std::collections::HashMap;

use i18n_convert::ir::{EntryValue, I18nEntry, I18nResource, PluralSet, ResourceMetadata};
use indexmap::IndexMap;

use i18n_validate::config::ResolvedConfig;
use i18n_validate::diagnostic::{CheckId, Severity};
use i18n_validate::discovery::ValidationContext;
use i18n_validate::layout::Layout;
use i18n_validate::validate;

// ── Test helpers ────────────────────────────────────────────────────────────

fn make_plural(
    zero: Option<&str>,
    one: Option<&str>,
    two: Option<&str>,
    few: Option<&str>,
    many: Option<&str>,
    other: &str,
) -> PluralSet {
    PluralSet {
        zero: zero.map(|s| s.to_string()),
        one: one.map(|s| s.to_string()),
        two: two.map(|s| s.to_string()),
        few: few.map(|s| s.to_string()),
        many: many.map(|s| s.to_string()),
        other: other.to_string(),
        ..Default::default()
    }
}

fn make_config(ref_lang: &str) -> ResolvedConfig {
    ResolvedConfig {
        ref_lang: ref_lang.to_string(),
        expected_languages: Vec::new(),
        layout: None,
        include: Vec::new(),
        exclude: Vec::new(),
        no_warnings: false,
        strict: false,
        skip_checks: Vec::new(),
        check_severity: HashMap::new(),
        language_configs: HashMap::new(),
    }
}

/// Build a ValidationContext with plural entries for testing.
///
/// `ref_plurals`: Vec of (key, PluralSet) for the reference language.
/// `translations`: Vec of (language_code, Vec of (key, PluralSet)) for translations.
fn build_plural_context(
    ref_lang: &str,
    ref_plurals: Vec<(&str, PluralSet)>,
    translations: Vec<(&str, Vec<(&str, PluralSet)>)>,
) -> ValidationContext {
    let file_name = "messages.json".to_string();

    // Build reference resource
    let mut ref_entries = IndexMap::new();
    for (key, plural) in ref_plurals {
        ref_entries.insert(
            key.to_string(),
            I18nEntry {
                key: key.to_string(),
                value: EntryValue::Plural(plural),
                ..Default::default()
            },
        );
    }
    let ref_resource = I18nResource {
        metadata: ResourceMetadata::default(),
        entries: ref_entries,
    };

    let mut ref_resources = HashMap::new();
    ref_resources.insert(file_name.clone(), ref_resource);

    // Build translation resources
    let mut lang_resources = HashMap::new();
    let mut discovered_languages = Vec::new();

    for (lang, plurals) in translations {
        let mut entries = IndexMap::new();
        for (key, plural) in plurals {
            entries.insert(
                key.to_string(),
                I18nEntry {
                    key: key.to_string(),
                    value: EntryValue::Plural(plural),
                    ..Default::default()
                },
            );
        }
        let resource = I18nResource {
            metadata: ResourceMetadata::default(),
            entries,
        };

        let mut files = HashMap::new();
        files.insert(file_name.clone(), resource);
        lang_resources.insert(lang.to_string(), files);
        discovered_languages.push(lang.to_string());
    }

    ValidationContext {
        ref_lang: ref_lang.to_string(),
        ref_resources,
        lang_resources,
        parse_failures: Vec::new(),
        discovered_languages,
        discovered_formats: Vec::new(),
        config: make_config(ref_lang),
        layout: Layout::Directory,
        ref_file_names: vec![file_name],
    }
}

/// Run all validators and filter for PluralRequirements diagnostics.
fn run_plural_requirements(ctx: &ValidationContext) -> Vec<i18n_validate::diagnostic::Diagnostic> {
    let all = validate::run_all(ctx);
    all.into_iter()
        .filter(|d| d.check == CheckId::PluralRequirements)
        .collect()
}

/// Filter diagnostics by language.
fn for_lang<'a>(
    diags: &'a [i18n_validate::diagnostic::Diagnostic],
    lang: &str,
) -> Vec<&'a i18n_validate::diagnostic::Diagnostic> {
    diags.iter().filter(|d| d.language == lang).collect()
}

// ── English reference: correct {one, other} ─────────────────────────────────

#[test]
fn english_with_one_other_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![],
    );
    let diags = run_plural_requirements(&ctx);
    let en = for_lang(&diags, "en");
    assert!(
        en.is_empty(),
        "English {{one, other}} should have no issues"
    );
}

// ── Japanese: {other} only ──────────────────────────────────────────────────

#[test]
fn japanese_with_other_only_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "ja",
            vec![("items", make_plural(None, None, None, None, None, "#個"))],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let ja = for_lang(&diags, "ja");
    assert!(ja.is_empty(), "Japanese {{other}} should have no issues");
}

#[test]
fn japanese_with_extra_one_gets_warning() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "ja",
            vec![(
                "items",
                make_plural(None, Some("#個"), None, None, None, "#個"),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let ja = for_lang(&diags, "ja");

    assert_eq!(ja.len(), 1, "Should have one warning for extra 'one'");
    assert_eq!(ja[0].severity, Severity::Warning);
    assert!(
        ja[0].message.contains("one"),
        "Should mention 'one' as unnecessary"
    );
}

// ── Chinese: {other} only ───────────────────────────────────────────────────

#[test]
fn chinese_with_other_only_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "zh",
            vec![("items", make_plural(None, None, None, None, None, "#个"))],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let zh = for_lang(&diags, "zh");
    assert!(zh.is_empty(), "Chinese {{other}} should have no issues");
}

#[test]
fn chinese_hans_with_other_only_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "zh-Hans",
            vec![("items", make_plural(None, None, None, None, None, "#个"))],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let zh_hans = for_lang(&diags, "zh-Hans");
    assert!(
        zh_hans.is_empty(),
        "zh-Hans {{other}} should have no issues"
    );
}

// ── Korean: {other} only ────────────────────────────────────────────────────

#[test]
fn korean_with_extra_one_gets_warning() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "ko",
            vec![(
                "items",
                make_plural(None, Some("# 항목"), None, None, None, "# 항목"),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let ko = for_lang(&diags, "ko");
    assert_eq!(ko.len(), 1);
    assert_eq!(ko[0].severity, Severity::Warning);
}

// ── Arabic: all 6 forms required ────────────────────────────────────────────

#[test]
fn arabic_with_all_six_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "ar",
            vec![(
                "items",
                make_plural(
                    Some("لا عناصر"),
                    Some("عنصر واحد"),
                    Some("عنصران"),
                    Some("# عناصر"),
                    Some("# عنصرًا"),
                    "# عنصر",
                ),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let ar = for_lang(&diags, "ar");
    assert!(
        ar.is_empty(),
        "Arabic with all 6 forms should have no issues"
    );
}

#[test]
fn arabic_with_only_one_other_missing_four() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "ar",
            vec![(
                "items",
                make_plural(None, Some("عنصر"), None, None, None, "عناصر"),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let ar = for_lang(&diags, "ar");

    let errors: Vec<_> = ar
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(errors.len(), 1, "Should have one error for missing forms");

    let msg = &errors[0].message;
    assert!(msg.contains("zero"), "Should mention missing 'zero'");
    assert!(msg.contains("two"), "Should mention missing 'two'");
    assert!(msg.contains("few"), "Should mention missing 'few'");
    assert!(msg.contains("many"), "Should mention missing 'many'");
}

#[test]
fn arabic_with_only_other_missing_five() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "ar",
            vec![("items", make_plural(None, None, None, None, None, "عناصر"))],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let ar_errors: Vec<_> = for_lang(&diags, "ar")
        .into_iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();

    assert_eq!(ar_errors.len(), 1);
    let msg = &ar_errors[0].message;
    assert!(msg.contains("zero"));
    assert!(msg.contains("one"));
    assert!(msg.contains("two"));
    assert!(msg.contains("few"));
    assert!(msg.contains("many"));
}

// ── Russian: {one, few, many, other} ────────────────────────────────────────

#[test]
fn russian_with_all_four_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "ru",
            vec![(
                "items",
                make_plural(
                    None,
                    Some("# предмет"),
                    None,
                    Some("# предмета"),
                    Some("# предметов"),
                    "# предметов",
                ),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let ru = for_lang(&diags, "ru");
    assert!(
        ru.is_empty(),
        "Russian with {{one, few, many, other}} should have no issues"
    );
}

#[test]
fn russian_with_only_one_other_missing_two() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "ru",
            vec![(
                "items",
                make_plural(None, Some("# предмет"), None, None, None, "# предметов"),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let ru_errors: Vec<_> = for_lang(&diags, "ru")
        .into_iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();

    assert_eq!(ru_errors.len(), 1);
    let msg = &ru_errors[0].message;
    assert!(msg.contains("few"), "Should mention missing 'few'");
    assert!(msg.contains("many"), "Should mention missing 'many'");
}

#[test]
fn russian_with_extra_zero_gets_warning() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "ru",
            vec![(
                "items",
                make_plural(
                    Some("нет предметов"),
                    Some("# предмет"),
                    None,
                    Some("# предмета"),
                    Some("# предметов"),
                    "# предметов",
                ),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let ru = for_lang(&diags, "ru");

    let warnings: Vec<_> = ru
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .collect();
    assert_eq!(warnings.len(), 1, "Should warn about extra 'zero'");
    assert!(warnings[0].message.contains("zero"));

    let errors: Vec<_> = ru
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "Should have no errors when all required forms present"
    );
}

// ── Polish: {one, few, many, other} ─────────────────────────────────────────

#[test]
fn polish_complete_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "pl",
            vec![(
                "items",
                make_plural(
                    None,
                    Some("# element"),
                    None,
                    Some("# elementy"),
                    Some("# elementów"),
                    "# elementów",
                ),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let pl = for_lang(&diags, "pl");
    assert!(pl.is_empty());
}

// ── Czech: {one, few, many, other} ──────────────────────────────────────────

#[test]
fn czech_complete_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "cs",
            vec![(
                "items",
                make_plural(
                    None,
                    Some("# položka"),
                    None,
                    Some("# položky"),
                    Some("# položek"),
                    "# položek",
                ),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    assert!(for_lang(&diags, "cs").is_empty());
}

// ── Ukrainian: {one, few, many, other} ──────────────────────────────────────

#[test]
fn ukrainian_missing_few_and_many() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "uk",
            vec![(
                "items",
                make_plural(None, Some("# елемент"), None, None, None, "# елементів"),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let uk_errors: Vec<_> = for_lang(&diags, "uk")
        .into_iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(uk_errors.len(), 1);
    assert!(uk_errors[0].message.contains("few"));
    assert!(uk_errors[0].message.contains("many"));
}

// ── Hebrew: {one, two, other} ───────────────────────────────────────────────

#[test]
fn hebrew_complete_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "he",
            vec![(
                "items",
                make_plural(
                    None,
                    Some("פריט #"),
                    Some("# פריטים"),
                    None,
                    None,
                    "# פריטים",
                ),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    assert!(for_lang(&diags, "he").is_empty());
}

#[test]
fn hebrew_legacy_iw_resolves() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "iw",
            vec![(
                "items",
                make_plural(None, Some("פריט"), None, None, None, "פריטים"),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let iw_errors: Vec<_> = for_lang(&diags, "iw")
        .into_iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    // iw → he → {one, two, other} — missing "two"
    assert_eq!(iw_errors.len(), 1);
    assert!(iw_errors[0].message.contains("two"));
}

// ── Welsh: all 6 forms ──────────────────────────────────────────────────────

#[test]
fn welsh_complete_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "cy",
            vec![(
                "items",
                make_plural(
                    Some("dim eitemau"),
                    Some("# eitem"),
                    Some("# eitem"),
                    Some("# eitem"),
                    Some("# eitem"),
                    "# eitem",
                ),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    assert!(for_lang(&diags, "cy").is_empty());
}

// ── Latvian: {zero, one, other} ─────────────────────────────────────────────

#[test]
fn latvian_complete_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "lv",
            vec![(
                "items",
                make_plural(
                    Some("nav vienumu"),
                    Some("# vienums"),
                    None,
                    None,
                    None,
                    "# vienumi",
                ),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    assert!(for_lang(&diags, "lv").is_empty());
}

#[test]
fn latvian_missing_zero() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "lv",
            vec![(
                "items",
                make_plural(None, Some("# vienums"), None, None, None, "# vienumi"),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let lv_errors: Vec<_> = for_lang(&diags, "lv")
        .into_iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(lv_errors.len(), 1);
    assert!(lv_errors[0].message.contains("zero"));
}

// ── Romanian: {one, few, other} ─────────────────────────────────────────────

#[test]
fn romanian_complete_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "ro",
            vec![(
                "items",
                make_plural(
                    None,
                    Some("# articol"),
                    None,
                    Some("# articole"),
                    None,
                    "# de articole",
                ),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    assert!(for_lang(&diags, "ro").is_empty());
}

// ── Croatian: {one, few, other} ─────────────────────────────────────────────

#[test]
fn croatian_complete_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "hr",
            vec![(
                "items",
                make_plural(
                    None,
                    Some("# stavka"),
                    None,
                    Some("# stavke"),
                    None,
                    "# stavki",
                ),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    assert!(for_lang(&diags, "hr").is_empty());
}

// ── Slovenian: {one, two, few, other} ───────────────────────────────────────

#[test]
fn slovenian_complete_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "sl",
            vec![(
                "items",
                make_plural(
                    None,
                    Some("# predmet"),
                    Some("# predmeta"),
                    Some("# predmeti"),
                    None,
                    "# predmetov",
                ),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    assert!(for_lang(&diags, "sl").is_empty());
}

#[test]
fn slovenian_missing_two_and_few() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "sl",
            vec![(
                "items",
                make_plural(None, Some("# predmet"), None, None, None, "# predmetov"),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let sl_errors: Vec<_> = for_lang(&diags, "sl")
        .into_iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(sl_errors.len(), 1);
    assert!(sl_errors[0].message.contains("two"));
    assert!(sl_errors[0].message.contains("few"));
}

// ── Irish: {one, two, few, many, other} ─────────────────────────────────────

#[test]
fn irish_complete_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "ga",
            vec![(
                "items",
                make_plural(
                    None,
                    Some("# rud"),
                    Some("# rud"),
                    Some("# rud"),
                    Some("# rud"),
                    "# rud",
                ),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    assert!(for_lang(&diags, "ga").is_empty());
}

// ── Maltese: {one, two, few, many, other} ───────────────────────────────────

#[test]
fn maltese_missing_all_optional() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "mt",
            vec![(
                "items",
                make_plural(None, None, None, None, None, "# oġġetti"),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let mt_errors: Vec<_> = for_lang(&diags, "mt")
        .into_iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(mt_errors.len(), 1);
    assert!(mt_errors[0].message.contains("one"));
    assert!(mt_errors[0].message.contains("two"));
    assert!(mt_errors[0].message.contains("few"));
    assert!(mt_errors[0].message.contains("many"));
}

// ── Scottish Gaelic: {one, two, few, other} ─────────────────────────────────

#[test]
fn scottish_gaelic_complete_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "gd",
            vec![(
                "items",
                make_plural(
                    None,
                    Some("# rud"),
                    Some("# rud"),
                    Some("# rudan"),
                    None,
                    "# rud",
                ),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    assert!(for_lang(&diags, "gd").is_empty());
}

// ── Lithuanian: {one, few, many, other} ─────────────────────────────────────

#[test]
fn lithuanian_complete_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "lt",
            vec![(
                "items",
                make_plural(
                    None,
                    Some("# elementas"),
                    None,
                    Some("# elementai"),
                    Some("# elementų"),
                    "# elementų",
                ),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    assert!(for_lang(&diags, "lt").is_empty());
}

// ── Unknown language: skipped ───────────────────────────────────────────────

#[test]
fn unknown_language_is_skipped() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "xx",
            vec![(
                "items",
                make_plural(None, Some("# item"), None, None, None, "# items"),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let xx = for_lang(&diags, "xx");
    assert!(xx.is_empty(), "Unknown language should be skipped");
}

// ── Multiple keys ───────────────────────────────────────────────────────────

#[test]
fn multiple_plural_keys_checked_independently() {
    let ctx = build_plural_context(
        "en",
        vec![
            (
                "items",
                make_plural(None, Some("# item"), None, None, None, "# items"),
            ),
            (
                "messages",
                make_plural(None, Some("# message"), None, None, None, "# messages"),
            ),
        ],
        vec![(
            "ar",
            vec![
                // items: complete (all 6)
                (
                    "items",
                    make_plural(
                        Some("z"),
                        Some("o"),
                        Some("t"),
                        Some("f"),
                        Some("m"),
                        "other",
                    ),
                ),
                // messages: incomplete (only one, other)
                (
                    "messages",
                    make_plural(None, Some("رسالة"), None, None, None, "رسائل"),
                ),
            ],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let ar = for_lang(&diags, "ar");

    // items should have no errors
    let items_diags: Vec<_> = ar
        .iter()
        .filter(|d| d.key.as_deref() == Some("items"))
        .collect();
    assert!(
        items_diags.is_empty(),
        "Complete items should have no issues"
    );

    // messages should have errors for missing forms
    let msg_errors: Vec<_> = ar
        .iter()
        .filter(|d| d.key.as_deref() == Some("messages") && d.severity == Severity::Error)
        .collect();
    assert_eq!(msg_errors.len(), 1);
    assert!(msg_errors[0].message.contains("zero"));
    assert!(msg_errors[0].message.contains("two"));
    assert!(msg_errors[0].message.contains("few"));
    assert!(msg_errors[0].message.contains("many"));
}

// ── Multiple languages ──────────────────────────────────────────────────────

#[test]
fn multiple_languages_checked() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![
            // Japanese: correct with {other} only
            (
                "ja",
                vec![("items", make_plural(None, None, None, None, None, "#個"))],
            ),
            // Arabic: incomplete with {one, other}
            (
                "ar",
                vec![(
                    "items",
                    make_plural(None, Some("عنصر"), None, None, None, "عناصر"),
                )],
            ),
            // German: correct with {one, other}
            (
                "de",
                vec![(
                    "items",
                    make_plural(None, Some("# Artikel"), None, None, None, "# Artikel"),
                )],
            ),
            // Russian: incomplete with {one, other}
            (
                "ru",
                vec![(
                    "items",
                    make_plural(None, Some("элемент"), None, None, None, "элементы"),
                )],
            ),
        ],
    );
    let diags = run_plural_requirements(&ctx);

    assert!(for_lang(&diags, "ja").is_empty(), "Japanese should be fine");
    assert!(for_lang(&diags, "de").is_empty(), "German should be fine");

    let ar_errors: Vec<_> = for_lang(&diags, "ar")
        .into_iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(ar_errors.len(), 1, "Arabic should have errors");

    let ru_errors: Vec<_> = for_lang(&diags, "ru")
        .into_iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(ru_errors.len(), 1, "Russian should have errors");
}

// ── Expected/found metadata ─────────────────────────────────────────────────

#[test]
fn diagnostics_include_expected_and_found() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "ar",
            vec![(
                "items",
                make_plural(None, Some("عنصر"), None, None, None, "عناصر"),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let ar_errors: Vec<_> = for_lang(&diags, "ar")
        .into_iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();

    assert_eq!(ar_errors.len(), 1);
    let diag = &ar_errors[0];

    // Should have expected = CLDR categories
    assert!(diag.expected.is_some());
    let expected = diag.expected.as_ref().unwrap();
    assert!(expected.contains("zero"));
    assert!(expected.contains("one"));
    assert!(expected.contains("two"));
    assert!(expected.contains("few"));
    assert!(expected.contains("many"));
    assert!(expected.contains("other"));

    // Should have found = present categories
    assert!(diag.found.is_some());
    let found = diag.found.as_ref().unwrap();
    assert!(found.contains("one"));
    assert!(found.contains("other"));

    // Should have file and key
    assert_eq!(diag.file.as_deref(), Some("messages.json"));
    assert_eq!(diag.key.as_deref(), Some("items"));
}

// ── Region/alias variants ───────────────────────────────────────────────────

#[test]
fn pt_br_uses_portuguese_rules() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "pt-BR",
            vec![(
                "items",
                make_plural(None, Some("# item"), None, None, None, "# itens"),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let pt_br = for_lang(&diags, "pt-BR");
    assert!(pt_br.is_empty(), "pt-BR {{one, other}} should be correct");
}

#[test]
fn sr_latn_uses_serbian_rules() {
    // Serbian needs {one, few, other}
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "sr-Latn",
            vec![(
                "items",
                make_plural(None, Some("# stavka"), None, None, None, "# stavki"),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let sr_latn_errors: Vec<_> = for_lang(&diags, "sr-Latn")
        .into_iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(sr_latn_errors.len(), 1, "sr-Latn should need 'few' form");
    assert!(sr_latn_errors[0].message.contains("few"));
}

// ── Reference language with wrong forms ─────────────────────────────────────

#[test]
fn reference_language_also_checked() {
    // If reference language is Arabic but only has {one, other}, flag it
    let ctx = build_plural_context(
        "ar",
        vec![(
            "items",
            make_plural(None, Some("عنصر"), None, None, None, "عناصر"),
        )],
        vec![],
    );
    let diags = run_plural_requirements(&ctx);
    let ar = for_lang(&diags, "ar");
    let errors: Vec<_> = ar
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert_eq!(errors.len(), 1, "Arabic reference should also be checked");
}

// ── Breton: {one, two, few, many, other} ────────────────────────────────────

#[test]
fn breton_complete_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "br",
            vec![(
                "items",
                make_plural(
                    None,
                    Some("# tra"),
                    Some("# dra"),
                    Some("# zra"),
                    Some("# a draoù"),
                    "# tra",
                ),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    assert!(for_lang(&diags, "br").is_empty());
}

// ── Belarusian: {one, few, many, other} ─────────────────────────────────────

#[test]
fn belarusian_complete_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "be",
            vec![(
                "items",
                make_plural(
                    None,
                    Some("# рэч"),
                    None,
                    Some("# рэчы"),
                    Some("# рэчаў"),
                    "# рэчаў",
                ),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    assert!(for_lang(&diags, "be").is_empty());
}

// ── Both errors and warnings for the same language ──────────────────────────

#[test]
fn mixed_errors_and_warnings() {
    // Slovenian needs {one, two, few, other}
    // Give it {zero, one, other} → missing two, few (errors) + extra zero (warning)
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "sl",
            vec![(
                "items",
                make_plural(
                    Some("nič"),
                    Some("# predmet"),
                    None,
                    None,
                    None,
                    "# predmetov",
                ),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let sl = for_lang(&diags, "sl");

    let errors: Vec<_> = sl
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    let warnings: Vec<_> = sl
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .collect();

    assert_eq!(errors.len(), 1, "Should have 1 error for missing forms");
    assert!(errors[0].message.contains("two"));
    assert!(errors[0].message.contains("few"));

    assert_eq!(warnings.len(), 1, "Should have 1 warning for extra 'zero'");
    assert!(warnings[0].message.contains("zero"));
}

// ── Skip check via config ───────────────────────────────────────────────────

#[test]
fn skipped_when_configured() {
    let mut ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "ar",
            vec![(
                "items",
                make_plural(None, Some("عنصر"), None, None, None, "عناصر"),
            )],
        )],
    );
    ctx.config.skip_checks = vec![CheckId::PluralRequirements];
    let diags = run_plural_requirements(&ctx);
    assert!(
        diags.is_empty(),
        "Should produce no diagnostics when skipped"
    );
}

// ── Indonesian: {other} only ────────────────────────────────────────────────

#[test]
fn indonesian_legacy_code_resolves() {
    // "in" is legacy code for Indonesian → should resolve to {other}
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "in",
            vec![(
                "items",
                make_plural(None, Some("# barang"), None, None, None, "# barang"),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    let id_warnings: Vec<_> = for_lang(&diags, "in")
        .into_iter()
        .filter(|d| d.severity == Severity::Warning)
        .collect();
    assert_eq!(
        id_warnings.len(),
        1,
        "'in' → id → {{other}}, so 'one' is extra"
    );
    assert!(id_warnings[0].message.contains("one"));
}

// ── Thai: {other} only ──────────────────────────────────────────────────────

#[test]
fn thai_with_other_only_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "th",
            vec![(
                "items",
                make_plural(None, None, None, None, None, "# รายการ"),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    assert!(for_lang(&diags, "th").is_empty());
}

// ── Vietnamese: {other} only ────────────────────────────────────────────────

#[test]
fn vietnamese_with_other_only_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "vi",
            vec![("items", make_plural(None, None, None, None, None, "# mục"))],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    assert!(for_lang(&diags, "vi").is_empty());
}

// ── Turkish: {one, other} ───────────────────────────────────────────────────

#[test]
fn turkish_with_one_other_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "tr",
            vec![(
                "items",
                make_plural(None, Some("# öğe"), None, None, None, "# öğe"),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    assert!(for_lang(&diags, "tr").is_empty());
}

// ── French: {one, other} ───────────────────────────────────────────────────

#[test]
fn french_with_one_other_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "fr",
            vec![(
                "items",
                make_plural(None, Some("# élément"), None, None, None, "# éléments"),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    assert!(for_lang(&diags, "fr").is_empty());
}

// ── German: {one, other} ───────────────────────────────────────────────────

#[test]
fn german_with_one_other_is_correct() {
    let ctx = build_plural_context(
        "en",
        vec![(
            "items",
            make_plural(None, Some("# item"), None, None, None, "# items"),
        )],
        vec![(
            "de",
            vec![(
                "items",
                make_plural(None, Some("# Element"), None, None, None, "# Elemente"),
            )],
        )],
    );
    let diags = run_plural_requirements(&ctx);
    assert!(for_lang(&diags, "de").is_empty());
}
