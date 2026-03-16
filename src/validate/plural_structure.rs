use i18n_convert::ir::EntryValue;

use crate::diagnostic::{CheckId, Diagnostic, Severity};
use crate::discovery::ValidationContext;
use crate::validate::Validator;

pub struct PluralStructureValidator;

impl Validator for PluralStructureValidator {
    fn id(&self) -> CheckId {
        CheckId::PluralStructure
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, ctx: &ValidationContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (file_name, ref_resource) in &ctx.ref_resources {
            for (key, ref_entry) in &ref_resource.entries {
                let ref_plural = match &ref_entry.value {
                    EntryValue::Plural(p) => p,
                    _ => continue,
                };

                for (lang, lang_files) in &ctx.lang_resources {
                    if let Some(lang_resource) = lang_files.get(file_name) {
                        if let Some(lang_entry) = lang_resource.entries.get(key) {
                            match &lang_entry.value {
                                EntryValue::Plural(lang_plural) => {
                                    // Check that `other` is non-empty
                                    if lang_plural.other.is_empty() {
                                        diagnostics.push(
                                            Diagnostic::error(
                                                CheckId::PluralStructure,
                                                lang,
                                                "Plural form \"other\" is empty (required)",
                                            )
                                            .with_file(file_name)
                                            .with_key(key),
                                        );
                                    }

                                    // Check for missing plural forms that exist in reference
                                    let mut missing_forms = Vec::new();

                                    if ref_plural.one.is_some() && lang_plural.one.is_none() {
                                        missing_forms.push("one");
                                    }
                                    if ref_plural.zero.is_some() && lang_plural.zero.is_none() {
                                        missing_forms.push("zero");
                                    }
                                    if ref_plural.two.is_some() && lang_plural.two.is_none() {
                                        missing_forms.push("two");
                                    }
                                    if ref_plural.few.is_some() && lang_plural.few.is_none() {
                                        missing_forms.push("few");
                                    }
                                    if ref_plural.many.is_some() && lang_plural.many.is_none() {
                                        missing_forms.push("many");
                                    }

                                    if !missing_forms.is_empty() {
                                        diagnostics.push(
                                            Diagnostic::error(
                                                CheckId::PluralStructure,
                                                lang,
                                                format!(
                                                    "Missing plural forms: {}",
                                                    missing_forms.join(", ")
                                                ),
                                            )
                                            .with_file(file_name)
                                            .with_key(key),
                                        );
                                    }
                                }
                                _ => {
                                    // Reference is plural but translation is not
                                    diagnostics.push(
                                        Diagnostic::error(
                                            CheckId::PluralStructure,
                                            lang,
                                            "Expected plural value but found non-plural",
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
