use i18n_convert::ir::EntryValue;

use crate::diagnostic::{CheckId, Diagnostic, Severity};
use crate::discovery::ValidationContext;
use crate::validate::Validator;

pub struct EmptyValuesValidator;

impl Validator for EmptyValuesValidator {
    fn id(&self) -> CheckId {
        CheckId::EmptyValues
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, ctx: &ValidationContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (lang, lang_files) in &ctx.lang_resources {
            for (file_name, lang_resource) in lang_files {
                for (key, entry) in &lang_resource.entries {
                    if let EntryValue::Simple(ref s) = entry.value {
                        if s.is_empty() {
                            diagnostics.push(
                                Diagnostic::warning(
                                    CheckId::EmptyValues,
                                    lang,
                                    format!("Empty value for key \"{key}\""),
                                )
                                .with_file(file_name)
                                .with_key(key),
                            );
                        }
                    }
                }
            }
        }

        diagnostics
    }
}
