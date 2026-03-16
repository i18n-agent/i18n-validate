use crate::diagnostic::{CheckId, Diagnostic, Severity};
use crate::discovery::ValidationContext;
use crate::validate::missing_keys::get_all_keys;
use crate::validate::Validator;

pub struct ExtraKeysValidator;

impl Validator for ExtraKeysValidator {
    fn id(&self) -> CheckId {
        CheckId::ExtraKeys
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, ctx: &ValidationContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (lang, lang_files) in &ctx.lang_resources {
            for (file_name, lang_resource) in lang_files {
                let lang_keys = get_all_keys(lang_resource);

                let ref_keys = if let Some(ref_resource) = ctx.ref_resources.get(file_name) {
                    get_all_keys(ref_resource)
                } else {
                    // File exists in translation but not in reference
                    Vec::new()
                };

                for lang_key in &lang_keys {
                    if !ref_keys.contains(lang_key) {
                        diagnostics.push(
                            Diagnostic::error(
                                CheckId::ExtraKeys,
                                lang,
                                format!(
                                    "Key \"{lang_key}\" exists in translation but not in reference"
                                ),
                            )
                            .with_file(file_name)
                            .with_key(lang_key),
                        );
                    }
                }
            }
        }

        diagnostics
    }
}
