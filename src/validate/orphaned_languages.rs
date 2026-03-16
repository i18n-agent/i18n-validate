use crate::diagnostic::{CheckId, Diagnostic, Severity};
use crate::discovery::ValidationContext;
use crate::locale;
use crate::validate::Validator;

pub struct OrphanedLanguagesValidator;

impl Validator for OrphanedLanguagesValidator {
    fn id(&self) -> CheckId {
        CheckId::OrphanedLanguages
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, ctx: &ValidationContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        // Only run if expected_languages is set
        if ctx.config.expected_languages.is_empty() {
            return diagnostics;
        }

        for discovered in &ctx.discovered_languages {
            let expected = ctx
                .config
                .expected_languages
                .iter()
                .any(|e| locale::fuzzy_eq(e, discovered));

            // Also check if it's the reference language
            let is_ref = locale::fuzzy_eq(discovered, &ctx.ref_lang);

            if !expected && !is_ref {
                diagnostics.push(Diagnostic::error(
                    CheckId::OrphanedLanguages,
                    discovered,
                    format!(
                        "Language \"{discovered}\" found but not in expected list"
                    ),
                ));
            }
        }

        diagnostics
    }
}
