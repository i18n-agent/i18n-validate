use std::collections::HashSet;

use regex::Regex;

use crate::diagnostic::{CheckId, Diagnostic, Severity};
use crate::discovery::ValidationContext;
use crate::validate::Validator;

pub struct PlaceholdersValidator;

impl Validator for PlaceholdersValidator {
    fn id(&self) -> CheckId {
        CheckId::Placeholders
    }

    fn default_severity(&self) -> Severity {
        Severity::Error
    }

    fn validate(&self, ctx: &ValidationContext) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for (file_name, ref_resource) in &ctx.ref_resources {
            for (key, ref_entry) in &ref_resource.entries {
                // Get reference placeholders
                let ref_placeholders = extract_placeholder_names(ref_entry);
                if ref_placeholders.is_empty() {
                    continue;
                }

                for (lang, lang_files) in &ctx.lang_resources {
                    if let Some(lang_resource) = lang_files.get(file_name) {
                        if let Some(lang_entry) = lang_resource.entries.get(key) {
                            let lang_placeholders = extract_placeholder_names(lang_entry);

                            // Check for missing placeholders
                            let missing: Vec<&String> = ref_placeholders
                                .iter()
                                .filter(|p| !lang_placeholders.contains(*p))
                                .collect();

                            // Check for extra placeholders
                            let extra: Vec<&String> = lang_placeholders
                                .iter()
                                .filter(|p| !ref_placeholders.contains(*p))
                                .collect();

                            if !missing.is_empty() || !extra.is_empty() {
                                let mut parts = Vec::new();
                                if !missing.is_empty() {
                                    parts.push(format!(
                                        "missing: {}",
                                        missing
                                            .iter()
                                            .map(|p| format!("{{{p}}}"))
                                            .collect::<Vec<_>>()
                                            .join(", ")
                                    ));
                                }
                                if !extra.is_empty() {
                                    parts.push(format!(
                                        "extra: {}",
                                        extra
                                            .iter()
                                            .map(|p| format!("{{{p}}}"))
                                            .collect::<Vec<_>>()
                                            .join(", ")
                                    ));
                                }

                                diagnostics.push(
                                    Diagnostic::error(
                                        CheckId::Placeholders,
                                        lang,
                                        format!("Placeholder mismatch: {}", parts.join("; ")),
                                    )
                                    .with_file(file_name)
                                    .with_key(key)
                                    .with_expected(format_set(&ref_placeholders))
                                    .with_found(format_set(&lang_placeholders)),
                                );
                            }
                        }
                    }
                }
            }
        }

        diagnostics
    }
}

fn extract_placeholder_names(entry: &i18n_convert::ir::I18nEntry) -> HashSet<String> {
    // First try IR-provided placeholders
    if !entry.placeholders.is_empty() {
        return entry
            .placeholders
            .iter()
            .map(|p| p.name.clone())
            .collect();
    }

    // Fall back to regex extraction from the value text
    let text = extract_text_from_value(&entry.value);
    extract_placeholders_from_text(&text)
}

fn extract_text_from_value(value: &i18n_convert::ir::EntryValue) -> String {
    match value {
        i18n_convert::ir::EntryValue::Simple(s) => s.clone(),
        i18n_convert::ir::EntryValue::Plural(p) => {
            // Combine all plural forms
            let mut parts = vec![p.other.clone()];
            if let Some(ref v) = p.one {
                parts.push(v.clone());
            }
            if let Some(ref v) = p.zero {
                parts.push(v.clone());
            }
            if let Some(ref v) = p.two {
                parts.push(v.clone());
            }
            if let Some(ref v) = p.few {
                parts.push(v.clone());
            }
            if let Some(ref v) = p.many {
                parts.push(v.clone());
            }
            parts.join(" ")
        }
        i18n_convert::ir::EntryValue::Array(arr) => arr.join(" "),
        i18n_convert::ir::EntryValue::Select(sel) => {
            sel.cases.values().cloned().collect::<Vec<_>>().join(" ")
        }
        i18n_convert::ir::EntryValue::MultiVariablePlural(mvp) => mvp.pattern.clone(),
    }
}

fn extract_placeholders_from_text(text: &str) -> HashSet<String> {
    let mut result = HashSet::new();

    // {name} - ICU style
    let icu_re = Regex::new(r"\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();
    for cap in icu_re.captures_iter(text) {
        result.insert(cap[1].to_string());
    }

    // {{name}} - Handlebars / i18next style
    let hbs_re = Regex::new(r"\{\{([a-zA-Z_][a-zA-Z0-9_]*)\}\}").unwrap();
    for cap in hbs_re.captures_iter(text) {
        result.insert(cap[1].to_string());
    }

    // %s, %d - printf positional
    let printf_re = Regex::new(r"%(\d+\$)?[sdfo]").unwrap();
    for cap in printf_re.captures_iter(text) {
        result.insert(cap[0].to_string());
    }

    // %{name} - Ruby style
    let ruby_re = Regex::new(r"%\{([a-zA-Z_][a-zA-Z0-9_]*)\}").unwrap();
    for cap in ruby_re.captures_iter(text) {
        result.insert(cap[1].to_string());
    }

    // :name - Laravel style
    let laravel_re = Regex::new(r":([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    // Only use laravel style if the text doesn't look like it has other placeholder styles
    if result.is_empty() {
        for cap in laravel_re.captures_iter(text) {
            result.insert(cap[1].to_string());
        }
    }

    result
}

fn format_set(set: &HashSet<String>) -> String {
    let mut items: Vec<&String> = set.iter().collect();
    items.sort();
    items
        .iter()
        .map(|s| format!("{{{s}}}"))
        .collect::<Vec<_>>()
        .join(", ")
}
