use i18n_convert::ir::{EntryValue, PluralSet};

use crate::cldr_plurals::{self, PluralCategory};
use crate::diagnostic::{CheckId, Diagnostic, Severity};
use crate::discovery::ValidationContext;
use crate::validate::Validator;

/// Validates that plural entries contain the correct CLDR-required categories
/// for each target language.
///
/// - Missing required categories → Error
/// - Extra (unnecessary) categories → Warning
pub struct PluralRequirementsValidator;

impl Validator for PluralRequirementsValidator {
    fn id(&self) -> CheckId {
        CheckId::PluralRequirements
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, ctx: &ValidationContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Check reference language plural entries against CLDR
        if let Some(required) = cldr_plurals::lookup(&ctx.ref_lang) {
            for (file_name, resource) in &ctx.ref_resources {
                for (key, entry) in &resource.entries {
                    if let EntryValue::Plural(plural) = &entry.value {
                        check_plural(
                            plural,
                            required,
                            &ctx.ref_lang,
                            file_name,
                            key,
                            &mut diagnostics,
                        );
                    }
                }
            }
        }

        // Check each translation language's plural entries against CLDR
        for (lang, lang_files) in &ctx.lang_resources {
            let required = match cldr_plurals::lookup(lang) {
                Some(r) => r,
                None => continue, // Unknown language, skip
            };

            for (file_name, resource) in lang_files {
                for (key, entry) in &resource.entries {
                    if let EntryValue::Plural(plural) = &entry.value {
                        check_plural(plural, required, lang, file_name, key, &mut diagnostics);
                    }
                }
            }
        }

        diagnostics
    }
}

/// Check a single plural entry against CLDR requirements for its language.
fn check_plural(
    plural: &PluralSet,
    required: &[PluralCategory],
    language: &str,
    file_name: &str,
    key: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let present = present_categories(plural);

    // Missing required categories → Error
    let missing: Vec<_> = required
        .iter()
        .filter(|cat| !present.contains(cat))
        .collect();

    if !missing.is_empty() {
        let missing_str: Vec<_> = missing.iter().map(|c| c.as_str()).collect();
        let required_str: Vec<_> = required.iter().map(|c| c.as_str()).collect();
        let present_str: Vec<_> = present.iter().map(|c| c.as_str()).collect();
        diagnostics.push(
            Diagnostic::error(
                CheckId::PluralRequirements,
                language,
                format!(
                    "Missing CLDR-required plural forms: {}",
                    missing_str.join(", ")
                ),
            )
            .with_file(file_name)
            .with_key(key)
            .with_expected(required_str.join(", "))
            .with_found(present_str.join(", ")),
        );
    }

    // Extra categories (present but not required) → Warning
    let extra: Vec<_> = present
        .iter()
        .filter(|cat| !required.contains(cat))
        .collect();

    if !extra.is_empty() {
        let extra_str: Vec<_> = extra.iter().map(|c| c.as_str()).collect();
        let required_str: Vec<_> = required.iter().map(|c| c.as_str()).collect();
        let present_str: Vec<_> = present.iter().map(|c| c.as_str()).collect();
        diagnostics.push(
            Diagnostic::warning(
                CheckId::PluralRequirements,
                language,
                format!(
                    "Unnecessary plural forms for this language: {}",
                    extra_str.join(", ")
                ),
            )
            .with_file(file_name)
            .with_key(key)
            .with_expected(required_str.join(", "))
            .with_found(present_str.join(", ")),
        );
    }
}

/// Collect which plural categories are present in a PluralSet.
fn present_categories(plural: &PluralSet) -> Vec<PluralCategory> {
    let mut cats = Vec::with_capacity(6);
    if plural.zero.is_some() {
        cats.push(PluralCategory::Zero);
    }
    if plural.one.is_some() {
        cats.push(PluralCategory::One);
    }
    if plural.two.is_some() {
        cats.push(PluralCategory::Two);
    }
    if plural.few.is_some() {
        cats.push(PluralCategory::Few);
    }
    if plural.many.is_some() {
        cats.push(PluralCategory::Many);
    }
    // `other` is always present (non-optional String field in PluralSet)
    cats.push(PluralCategory::Other);
    cats
}
