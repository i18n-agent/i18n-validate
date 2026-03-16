use i18n_convert::ir::{EntryValue, I18nResource};

use crate::diagnostic::{CheckId, Diagnostic, Severity};
use crate::discovery::ValidationContext;
use crate::validate::Validator;

pub struct MissingKeysValidator;

impl Validator for MissingKeysValidator {
    fn id(&self) -> CheckId {
        CheckId::MissingKeys
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, ctx: &ValidationContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (file_name, ref_resource) in &ctx.ref_resources {
            let ref_keys = get_all_keys(ref_resource);

            for (lang, lang_files) in &ctx.lang_resources {
                let lang_keys = if let Some(lang_resource) = lang_files.get(file_name) {
                    get_all_keys(lang_resource)
                } else {
                    // If the entire file is missing, every key is missing
                    Vec::new()
                };

                for ref_key in &ref_keys {
                    if !lang_keys.contains(ref_key) {
                        diagnostics.push(
                            Diagnostic::error(
                                CheckId::MissingKeys,
                                lang,
                                format!("Key \"{ref_key}\" missing in translation"),
                            )
                            .with_file(file_name)
                            .with_key(ref_key),
                        );
                    }
                }
            }
        }

        diagnostics
    }
}

/// Recursively extract all dotted key paths from a resource.
pub fn get_all_keys(resource: &I18nResource) -> Vec<String> {
    let mut keys = Vec::new();
    for (key, entry) in &resource.entries {
        collect_keys_from_value(key, &entry.value, &mut keys);
    }
    keys
}

fn collect_keys_from_value(prefix: &str, value: &EntryValue, keys: &mut Vec<String>) {
    match value {
        EntryValue::Simple(_)
        | EntryValue::Plural(_)
        | EntryValue::Array(_)
        | EntryValue::Select(_)
        | EntryValue::MultiVariablePlural(_) => {
            keys.push(prefix.to_string());
        }
    }
}
