use i18n_convert::ir::EntryValue;

use crate::diagnostic::{CheckId, Diagnostic, Severity};
use crate::discovery::ValidationContext;
use crate::validate::Validator;

pub struct UntranslatedValidator;

impl Validator for UntranslatedValidator {
    fn id(&self) -> CheckId {
        CheckId::Untranslated
    }

    fn default_severity(&self) -> Severity {
        Severity::Warning
    }

    fn validate(&self, ctx: &ValidationContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (file_name, ref_resource) in &ctx.ref_resources {
            for (key, ref_entry) in &ref_resource.entries {
                let ref_text = match &ref_entry.value {
                    EntryValue::Simple(s) => s.clone(),
                    _ => continue,
                };

                // Skip short entries (likely abbreviations, codes, etc.)
                if ref_text.len() < 3 {
                    continue;
                }

                for (lang, lang_files) in &ctx.lang_resources {
                    if let Some(lang_resource) = lang_files.get(file_name) {
                        if let Some(lang_entry) = lang_resource.entries.get(key) {
                            if let EntryValue::Simple(ref lang_text) = lang_entry.value {
                                if lang_text == &ref_text {
                                    diagnostics.push(
                                        Diagnostic::warning(
                                            CheckId::Untranslated,
                                            lang,
                                            "Value appears untranslated (identical to reference)",
                                        )
                                        .with_file(file_name)
                                        .with_key(key),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        diagnostics
    }
}
