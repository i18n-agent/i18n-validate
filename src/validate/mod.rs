mod empty_values;
mod extra_keys;
mod missing_keys;
mod missing_languages;
mod orphaned_languages;
mod parse_errors;
mod placeholders;
mod plural_structure;
mod untranslated;

use crate::config::{ResolvedConfig, SeverityOverride};
use crate::diagnostic::{CheckId, Diagnostic, Severity};
use crate::discovery::ValidationContext;

pub trait Validator {
    fn id(&self) -> CheckId;
    #[allow(dead_code)]
    fn default_severity(&self) -> Severity;
    fn validate(&self, ctx: &ValidationContext) -> Vec<Diagnostic>;
}

/// Run all validators against the context, applying config overrides.
pub fn run_all(ctx: &ValidationContext) -> Vec<Diagnostic> {
    let validators: Vec<Box<dyn Validator>> = vec![
        Box::new(missing_languages::MissingLanguagesValidator),
        Box::new(orphaned_languages::OrphanedLanguagesValidator),
        Box::new(missing_keys::MissingKeysValidator),
        Box::new(extra_keys::ExtraKeysValidator),
        Box::new(placeholders::PlaceholdersValidator),
        Box::new(plural_structure::PluralStructureValidator),
        Box::new(parse_errors::ParseErrorsValidator),
        Box::new(empty_values::EmptyValuesValidator),
        Box::new(untranslated::UntranslatedValidator),
    ];

    let config = &ctx.config;
    let mut all_diagnostics = Vec::new();

    for validator in &validators {
        let check_id = validator.id();

        // Skip if check is disabled
        if config.skip_checks.contains(&check_id) {
            continue;
        }

        // Skip if severity override is Off
        if let Some(SeverityOverride::Off) = config.check_severity.get(&check_id) {
            continue;
        }

        let mut diagnostics = validator.validate(ctx);

        // Apply severity overrides
        apply_severity_overrides(&mut diagnostics, config);

        // Filter per-language skips
        diagnostics.retain(|d| !is_language_skipped(&d.language, &d.check, config));

        // Filter warnings if --no-warnings
        if config.no_warnings {
            diagnostics.retain(|d| d.severity != Severity::Warning);
        }

        all_diagnostics.extend(diagnostics);
    }

    all_diagnostics
}

fn apply_severity_overrides(diagnostics: &mut [Diagnostic], config: &ResolvedConfig) {
    for d in diagnostics.iter_mut() {
        // Global check severity override
        if let Some(override_sev) = config.check_severity.get(&d.check) {
            if let Some(sev) = override_sev.to_severity() {
                d.severity = sev;
            }
        }

        // Per-language check severity override
        if let Some(lang_config) = config.language_configs.get(&d.language) {
            if let Some(sev_str) = lang_config.check_overrides.get(d.check.as_str()) {
                if let Some(override_sev) = SeverityOverride::parse(sev_str) {
                    if let Some(sev) = override_sev.to_severity() {
                        d.severity = sev;
                    }
                }
            }
        }
    }
}

fn is_language_skipped(language: &str, check: &CheckId, config: &ResolvedConfig) -> bool {
    if let Some(lang_config) = config.language_configs.get(language) {
        // Language-level skip
        if lang_config.skip == Some(true) {
            return true;
        }

        // Per-check "off" at language level
        if let Some(sev_str) = lang_config.check_overrides.get(check.as_str()) {
            if SeverityOverride::parse(sev_str) == Some(SeverityOverride::Off) {
                return true;
            }
        }
    }
    false
}
