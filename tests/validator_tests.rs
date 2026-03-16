use std::collections::HashMap;
use std::path::PathBuf;

use i18n_validate::config::ResolvedConfig;
use i18n_validate::diagnostic::{CheckId, Severity};
use i18n_validate::discovery::ValidationContext;
use i18n_validate::{discovery, layout, validate};

/// Return the absolute path to the project's `tests/fixtures` directory.
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Build a ValidationContext from the i18next fixtures with the given expected languages.
fn build_i18next_context(expected: &[&str]) -> ValidationContext {
    let path = fixtures_dir().join("i18next");
    let cfg = make_config(expected, &[]);
    let detected_layout = layout::detect(&path, cfg.layout).expect("layout detection should work");
    discovery::discover(&path, &detected_layout, &cfg).expect("discovery should succeed")
}

/// Build a ValidationContext from the flat-json fixtures with the given expected languages.
fn build_flat_context(expected: &[&str]) -> ValidationContext {
    let path = fixtures_dir().join("flat-json");
    let cfg = make_config(expected, &[]);
    let detected_layout = layout::detect(&path, cfg.layout).expect("layout detection should work");
    discovery::discover(&path, &detected_layout, &cfg).expect("discovery should succeed")
}

/// Build a ResolvedConfig with the given expected languages and skip checks.
fn make_config(expected: &[&str], skip: &[CheckId]) -> ResolvedConfig {
    ResolvedConfig {
        ref_lang: "en".to_string(),
        expected_languages: expected.iter().map(|s| s.to_string()).collect(),
        layout: None,
        include: Vec::new(),
        exclude: Vec::new(),
        no_warnings: false,
        strict: false,
        skip_checks: skip.to_vec(),
        check_severity: HashMap::new(),
        language_configs: HashMap::new(),
    }
}

/// Build a context and run all validators, returning only diagnostics for the given check.
fn run_check(
    ctx: &ValidationContext,
    check: CheckId,
) -> Vec<i18n_validate::diagnostic::Diagnostic> {
    let all = validate::run_all(ctx);
    all.into_iter().filter(|d| d.check == check).collect()
}

// ─── Missing Keys ────────────────────────────────────────────────────────────

#[test]
fn missing_keys_detects_absent_keys() {
    let ctx = build_i18next_context(&["en", "de", "ja"]);
    let diags = run_check(&ctx, CheckId::MissingKeys);

    // ja is missing: farewell, pricing.items, short
    let ja_diags: Vec<_> = diags.iter().filter(|d| d.language == "ja").collect();
    assert!(
        ja_diags.len() >= 3,
        "Expected at least 3 missing keys for ja, got {}",
        ja_diags.len()
    );

    let keys: Vec<&str> = ja_diags.iter().filter_map(|d| d.key.as_deref()).collect();
    assert!(
        keys.contains(&"farewell"),
        "Should detect missing 'farewell'"
    );
    assert!(
        keys.contains(&"pricing.items"),
        "Should detect missing 'pricing.items'"
    );
    assert!(keys.contains(&"short"), "Should detect missing 'short'");

    // All should be errors
    for d in &ja_diags {
        assert_eq!(d.severity, Severity::Error);
    }
}

#[test]
fn missing_keys_no_false_positives_when_complete() {
    let ctx = build_i18next_context(&["en", "de", "ja"]);
    let diags = run_check(&ctx, CheckId::MissingKeys);

    // de has all keys from en — no missing-keys diagnostics for de
    let de_diags: Vec<_> = diags.iter().filter(|d| d.language == "de").collect();
    assert!(
        de_diags.is_empty(),
        "de should have no missing keys, got {:?}",
        de_diags
            .iter()
            .filter_map(|d| d.key.as_deref())
            .collect::<Vec<_>>()
    );
}

// ─── Extra Keys ──────────────────────────────────────────────────────────────

#[test]
fn extra_keys_detects_surplus_keys() {
    // Create a fixture with an extra key by using a tempdir
    let tmp = tempfile::tempdir().expect("create tempdir");
    let en_dir = tmp.path().join("en");
    let fr_dir = tmp.path().join("fr");
    std::fs::create_dir_all(&en_dir).unwrap();
    std::fs::create_dir_all(&fr_dir).unwrap();

    // en has only "hello"
    std::fs::write(en_dir.join("messages.json"), r#"{"hello":"Hello"}"#).unwrap();
    // fr has "hello" + "bonus" (extra key)
    std::fs::write(
        fr_dir.join("messages.json"),
        r#"{"hello":"Bonjour","bonus":"Extra"}"#,
    )
    .unwrap();

    let cfg = make_config(&["en", "fr"], &[]);
    let detected_layout = layout::detect(tmp.path(), cfg.layout).expect("layout detection");
    let ctx = discovery::discover(tmp.path(), &detected_layout, &cfg).expect("discovery");

    let diags = run_check(&ctx, CheckId::ExtraKeys);
    assert!(!diags.is_empty(), "Should detect extra key 'bonus' in fr");
    let keys: Vec<&str> = diags.iter().filter_map(|d| d.key.as_deref()).collect();
    assert!(
        keys.contains(&"bonus"),
        "Should detect 'bonus' as extra key"
    );
}

// ─── Placeholders ────────────────────────────────────────────────────────────

#[test]
fn placeholders_detects_mismatch() {
    let ctx = build_i18next_context(&["en", "de", "ja"]);
    let diags = run_check(&ctx, CheckId::Placeholders);

    // ja's pricing.total has {price} but is missing {currency}
    let ja_diags: Vec<_> = diags.iter().filter(|d| d.language == "ja").collect();
    assert!(
        !ja_diags.is_empty(),
        "Should detect placeholder mismatch in ja"
    );

    let total_diag = ja_diags
        .iter()
        .find(|d| d.key.as_deref() == Some("pricing.total"));
    assert!(
        total_diag.is_some(),
        "Should detect mismatch for pricing.total"
    );
    let total_diag = total_diag.unwrap();
    assert_eq!(total_diag.severity, Severity::Error);
    assert!(
        total_diag.message.contains("currency"),
        "Message should mention missing placeholder 'currency'"
    );

    // de should have no placeholder mismatches
    let de_diags: Vec<_> = diags.iter().filter(|d| d.language == "de").collect();
    assert!(
        de_diags.is_empty(),
        "de should have no placeholder mismatches"
    );
}

// ─── Empty Values ────────────────────────────────────────────────────────────

#[test]
fn empty_values_detects_empty_strings() {
    let ctx = build_i18next_context(&["en", "de", "ja"]);
    let diags = run_check(&ctx, CheckId::EmptyValues);

    // ja has empty_test = ""
    let ja_diags: Vec<_> = diags.iter().filter(|d| d.language == "ja").collect();
    assert!(!ja_diags.is_empty(), "Should detect empty value in ja");

    let empty_diag = ja_diags
        .iter()
        .find(|d| d.key.as_deref() == Some("empty_test"));
    assert!(
        empty_diag.is_some(),
        "Should detect empty value for 'empty_test'"
    );
    assert_eq!(
        empty_diag.unwrap().severity,
        Severity::Warning,
        "Empty values should be warnings"
    );

    // de should have no empty values
    let de_diags: Vec<_> = diags.iter().filter(|d| d.language == "de").collect();
    assert!(
        de_diags.is_empty(),
        "de should have no empty value warnings"
    );
}

// ─── Untranslated ────────────────────────────────────────────────────────────

#[test]
fn untranslated_detects_identical_values() {
    let ctx = build_i18next_context(&["en", "de", "ja"]);
    let diags = run_check(&ctx, CheckId::Untranslated);

    // ja.greeting = "Hello" is identical to en.greeting = "Hello"
    let ja_diags: Vec<_> = diags.iter().filter(|d| d.language == "ja").collect();
    assert!(
        !ja_diags.is_empty(),
        "Should detect untranslated value in ja"
    );

    let greeting_diag = ja_diags
        .iter()
        .find(|d| d.key.as_deref() == Some("greeting"));
    assert!(
        greeting_diag.is_some(),
        "Should detect untranslated 'greeting' in ja"
    );
    assert_eq!(
        greeting_diag.unwrap().severity,
        Severity::Warning,
        "Untranslated should be a warning"
    );

    // de.greeting = "Hallo" != "Hello" => no untranslated
    let de_diags: Vec<_> = diags.iter().filter(|d| d.language == "de").collect();
    assert!(
        de_diags.is_empty(),
        "de should have no untranslated warnings"
    );
}

#[test]
fn untranslated_skips_short_strings() {
    // en and de both have "short": "OK" (2 chars) — should NOT be flagged
    // because the untranslated validator skips strings with len < 3.
    let ctx = build_i18next_context(&["en", "de", "ja"]);
    let diags = run_check(&ctx, CheckId::Untranslated);

    let short_diags: Vec<_> = diags
        .iter()
        .filter(|d| d.key.as_deref() == Some("short"))
        .collect();
    assert!(
        short_diags.is_empty(),
        "Short strings (< 3 chars) should not be flagged as untranslated"
    );
}

// ─── Parse Errors ────────────────────────────────────────────────────────────

#[test]
fn parse_errors_returns_failures() {
    // Create a fixture with invalid JSON to trigger a parse error.
    let tmp = tempfile::tempdir().expect("create tempdir");
    let en_dir = tmp.path().join("en");
    let fr_dir = tmp.path().join("fr");
    std::fs::create_dir_all(&en_dir).unwrap();
    std::fs::create_dir_all(&fr_dir).unwrap();

    std::fs::write(en_dir.join("messages.json"), r#"{"hello":"Hello"}"#).unwrap();
    // Invalid JSON in fr
    std::fs::write(fr_dir.join("messages.json"), r#"{"broken": }"#).unwrap();

    let cfg = make_config(&["en", "fr"], &[]);
    let detected_layout = layout::detect(tmp.path(), cfg.layout).expect("layout detection");
    let ctx = discovery::discover(tmp.path(), &detected_layout, &cfg).expect("discovery");

    let diags = run_check(&ctx, CheckId::ParseErrors);
    assert!(
        !diags.is_empty(),
        "Should detect parse error in fr/messages.json"
    );
    assert_eq!(diags[0].severity, Severity::Error);
}

// ─── Skip Checks ─────────────────────────────────────────────────────────────

#[test]
fn skip_checks_are_respected() {
    let path = fixtures_dir().join("i18next");
    let cfg = make_config(
        &["en", "de", "ja"],
        &[CheckId::MissingKeys, CheckId::Placeholders],
    );
    let detected_layout = layout::detect(&path, cfg.layout).expect("layout detection");
    let mut ctx = discovery::discover(&path, &detected_layout, &cfg).expect("discovery");
    // Apply the config with skip_checks to the context
    ctx.config = cfg;
    let all = validate::run_all(&ctx);

    let missing = all
        .iter()
        .filter(|d| d.check == CheckId::MissingKeys)
        .count();
    let placeholders = all
        .iter()
        .filter(|d| d.check == CheckId::Placeholders)
        .count();

    assert_eq!(
        missing, 0,
        "Skipped missing-keys should produce 0 diagnostics"
    );
    assert_eq!(
        placeholders, 0,
        "Skipped placeholders should produce 0 diagnostics"
    );

    // Other checks should still run
    let empty = all
        .iter()
        .filter(|d| d.check == CheckId::EmptyValues)
        .count();
    assert!(
        empty > 0,
        "Non-skipped checks (empty-values) should still produce diagnostics"
    );
}

// ─── No Warnings ─────────────────────────────────────────────────────────────

#[test]
fn no_warnings_filters_warning_diagnostics() {
    let path = fixtures_dir().join("i18next");
    let mut cfg = make_config(&["en", "de", "ja"], &[]);
    cfg.no_warnings = true;
    let detected_layout = layout::detect(&path, cfg.layout).expect("layout detection");
    let ctx = discovery::discover(&path, &detected_layout, &cfg).expect("discovery");
    let all = validate::run_all(&ctx);

    let warnings = all
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();
    assert_eq!(
        warnings, 0,
        "With no_warnings=true, no warning diagnostics should be returned"
    );

    let errors = all.iter().filter(|d| d.severity == Severity::Error).count();
    assert!(errors > 0, "Errors should still be present");
}

// ─── Flat Layout Validation ──────────────────────────────────────────────────

#[test]
fn flat_layout_complete_translation_has_no_errors() {
    let ctx = build_flat_context(&["en", "de"]);
    let all = validate::run_all(&ctx);

    let errors: Vec<_> = all
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "Complete flat layout should have no errors, got: {:?}",
        errors.iter().map(|d| &d.message).collect::<Vec<_>>()
    );
}
