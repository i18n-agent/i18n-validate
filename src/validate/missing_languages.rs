use crate::diagnostic::{CheckId, Diagnostic, Severity};
use crate::discovery::ValidationContext;
use crate::locale;
use crate::validate::Validator;

pub struct MissingLanguagesValidator;

impl Validator for MissingLanguagesValidator {
    fn id(&self) -> CheckId {
        CheckId::MissingLanguages
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, ctx: &ValidationContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        if ctx.config.expected_languages.is_empty() {
            return diagnostics;
        }

        for expected in &ctx.config.expected_languages {
            // Skip the reference language itself
            if locale::fuzzy_eq(expected, &ctx.ref_lang) {
                continue;
            }

            let found = ctx
                .discovered_languages
                .iter()
                .any(|d| locale::fuzzy_eq(d, expected));

            if !found {
                diagnostics.push(Diagnostic::error(
                    CheckId::MissingLanguages,
                    expected,
                    format!("Expected language \"{expected}\" not found"),
                ));
            }
        }

        diagnostics
    }
}
