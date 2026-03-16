use std::collections::HashMap;
use std::path::Path;

use glob::Pattern;
use i18n_convert::detect::detect_best;
use i18n_convert::formats::FormatRegistry;
use i18n_convert::ir::{EntryValue, I18nEntry, I18nResource, ResourceMetadata};
use indexmap::IndexMap;

use crate::config::ResolvedConfig;
use crate::diagnostic::{CheckId, Diagnostic};
use crate::layout::Layout;
use crate::locale;

/// Holds all parsed resources and metadata needed for validation.
pub struct ValidationContext {
    pub ref_lang: String,
    pub ref_resources: HashMap<String, I18nResource>,
    pub lang_resources: HashMap<String, HashMap<String, I18nResource>>,
    pub parse_failures: Vec<Diagnostic>,
    pub discovered_languages: Vec<String>,
    pub discovered_formats: Vec<String>,
    pub config: ResolvedConfig,
    pub layout: Layout,
    pub ref_file_names: Vec<String>,
}

/// Check if a filename matches include/exclude patterns.
fn matches_filters(filename: &str, include: &[String], exclude: &[String]) -> bool {
    if !include.is_empty() {
        let included = include.iter().any(|pat| {
            Pattern::new(pat)
                .map(|p| p.matches(filename))
                .unwrap_or(false)
        });
        if !included {
            return false;
        }
    }

    if !exclude.is_empty() {
        let excluded = exclude.iter().any(|pat| {
            Pattern::new(pat)
                .map(|p| p.matches(filename))
                .unwrap_or(false)
        });
        if excluded {
            return false;
        }
    }

    true
}

/// Parse a file using auto-detected format.
fn parse_file(
    registry: &FormatRegistry,
    file_path: &Path,
) -> Result<(I18nResource, String), String> {
    let content = std::fs::read(file_path)
        .map_err(|e| format!("Failed to read {}: {e}", file_path.display()))?;

    let format_id = detect_best(registry, file_path, &content)
        .ok_or_else(|| format!("Could not detect format for {}", file_path.display()))?;

    let format_entry = registry
        .get(format_id)
        .ok_or_else(|| format!("Unknown format: {format_id}"))?;

    let resource = format_entry
        .parser
        .parse(&content)
        .map_err(|e| format!("Parse error in {}: {e}", file_path.display()))?;

    Ok((resource, format_id.to_string()))
}

/// Discover and parse translation files based on layout.
pub fn discover(
    path: &Path,
    layout: &Layout,
    config: &ResolvedConfig,
) -> Result<ValidationContext, Box<dyn std::error::Error>> {
    let registry = FormatRegistry::new();
    let mut ctx = ValidationContext {
        ref_lang: config.ref_lang.clone(),
        ref_resources: HashMap::new(),
        lang_resources: HashMap::new(),
        parse_failures: Vec::new(),
        discovered_languages: Vec::new(),
        discovered_formats: Vec::new(),
        config: config.clone(),
        layout: *layout,
        ref_file_names: Vec::new(),
    };

    match layout {
        Layout::Directory => discover_directory(path, &registry, config, &mut ctx)?,
        Layout::Flat => discover_flat(path, &registry, config, &mut ctx)?,
        Layout::SingleFile => discover_single_file(path, &registry, &mut ctx)?,
    }

    // Sort discovered languages for deterministic output
    ctx.discovered_languages.sort();
    ctx.discovered_formats.sort();
    ctx.discovered_formats.dedup();

    Ok(ctx)
}

fn discover_directory(
    path: &Path,
    registry: &FormatRegistry,
    config: &ResolvedConfig,
    ctx: &mut ValidationContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let entries = std::fs::read_dir(path)?;

    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let dir_name = entry.file_name();
        let dir_name_str = dir_name.to_string_lossy();

        let lang_code = match locale::extract_from_path(&dir_name_str) {
            Some(code) => code,
            None => continue,
        };

        // Parse all files in this language directory
        let lang_dir = entry.path();
        let files = std::fs::read_dir(&lang_dir)?;

        for file_entry in files {
            let file_entry = file_entry?;
            if !file_entry.file_type()?.is_file() {
                continue;
            }

            let file_name = file_entry.file_name();
            let file_name_str = file_name.to_string_lossy().to_string();

            if !matches_filters(&file_name_str, &config.include, &config.exclude) {
                continue;
            }

            let file_path = file_entry.path();
            match parse_file(registry, &file_path) {
                Ok((resource, format_id)) => {
                    if !ctx.discovered_formats.contains(&format_id) {
                        ctx.discovered_formats.push(format_id);
                    }

                    if locale::fuzzy_eq(&lang_code, &config.ref_lang) {
                        ctx.ref_resources.insert(file_name_str.clone(), resource);
                        if !ctx.ref_file_names.contains(&file_name_str) {
                            ctx.ref_file_names.push(file_name_str);
                        }
                    } else {
                        ctx.lang_resources
                            .entry(lang_code.clone())
                            .or_default()
                            .insert(file_name_str, resource);
                    }
                }
                Err(e) => {
                    ctx.parse_failures.push(
                        Diagnostic::error(CheckId::ParseErrors, &lang_code, e)
                            .with_file(file_name_str),
                    );
                }
            }
        }

        if !locale::fuzzy_eq(&lang_code, &config.ref_lang)
            && !ctx.discovered_languages.contains(&lang_code)
        {
            ctx.discovered_languages.push(lang_code);
        }
    }

    Ok(())
}

/// In flat layout, each file IS a language (en.json, de.json, etc.).
/// We need a canonical key so validators can match reference keys against translation keys.
/// We use the actual filename for the reference file, and store translation files under the
/// same canonical key (the reference filename).
fn discover_flat(
    path: &Path,
    registry: &FormatRegistry,
    config: &ResolvedConfig,
    ctx: &mut ValidationContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let entries: Vec<_> = std::fs::read_dir(path)?.filter_map(|e| e.ok()).collect();

    // First pass: find the reference file to get its canonical name
    let mut ref_file_name = None;
    for entry in &entries {
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy().to_string();
        if let Some(lang) = locale::extract_from_filename(&file_name_str) {
            if locale::fuzzy_eq(&lang, &config.ref_lang) {
                ref_file_name = Some(file_name_str);
                break;
            }
        }
    }

    let canonical_key = ref_file_name
        .clone()
        .unwrap_or_else(|| "translations".to_string());

    // Second pass: parse all files
    for entry in &entries {
        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }

        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy().to_string();

        if !matches_filters(&file_name_str, &config.include, &config.exclude) {
            continue;
        }

        let lang_code = match locale::extract_from_filename(&file_name_str) {
            Some(code) => code,
            None => continue,
        };

        let file_path = entry.path();
        match parse_file(registry, &file_path) {
            Ok((resource, format_id)) => {
                if !ctx.discovered_formats.contains(&format_id) {
                    ctx.discovered_formats.push(format_id);
                }

                if locale::fuzzy_eq(&lang_code, &config.ref_lang) {
                    ctx.ref_resources.insert(canonical_key.clone(), resource);
                    if !ctx.ref_file_names.contains(&file_name_str) {
                        ctx.ref_file_names.push(file_name_str);
                    }
                } else {
                    ctx.lang_resources
                        .entry(lang_code.clone())
                        .or_default()
                        .insert(canonical_key.clone(), resource);
                    if !ctx.discovered_languages.contains(&lang_code) {
                        ctx.discovered_languages.push(lang_code);
                    }
                }
            }
            Err(e) => {
                ctx.parse_failures.push(
                    Diagnostic::error(CheckId::ParseErrors, &lang_code, e).with_file(file_name_str),
                );
            }
        }
    }

    Ok(())
}

fn discover_single_file(
    path: &Path,
    registry: &FormatRegistry,
    ctx: &mut ValidationContext,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default();

    match parse_file(registry, path) {
        Ok((resource, format_id)) => {
            if !ctx.discovered_formats.contains(&format_id) {
                ctx.discovered_formats.push(format_id.clone());
            }

            if format_id == "xcstrings" {
                discover_single_file_xcstrings(&file_name, resource, ctx);
            } else {
                // Bilingual formats (XLIFF, PO, etc.): source + target in one resource.
                discover_single_file_bilingual(&file_name, resource, ctx);
            }
        }
        Err(e) => {
            ctx.parse_failures.push(
                Diagnostic::error(CheckId::ParseErrors, ctx.ref_lang.clone(), e)
                    .with_file(file_name),
            );
        }
    }

    Ok(())
}

/// Handle bilingual single-file formats like XLIFF where each entry has
/// a source string (reference) and a target string (translation).
fn discover_single_file_bilingual(
    file_name: &str,
    resource: I18nResource,
    ctx: &mut ValidationContext,
) {
    // Determine reference and target languages from metadata
    let source_lang = resource
        .metadata
        .source_locale
        .clone()
        .unwrap_or_else(|| ctx.ref_lang.clone());
    let target_lang = resource.metadata.locale.clone();

    // Override ctx.ref_lang with the file's declared source language
    ctx.ref_lang = locale::normalize(&source_lang);

    // Build the reference resource from source strings
    let mut ref_entries = IndexMap::new();
    for (key, entry) in &resource.entries {
        let ref_value = entry
            .source
            .as_ref()
            .map(|s| EntryValue::Simple(s.clone()))
            .unwrap_or_else(|| EntryValue::Simple(String::new()));

        ref_entries.insert(
            key.clone(),
            I18nEntry {
                key: key.clone(),
                value: ref_value,
                comments: entry.comments.clone(),
                contexts: entry.contexts.clone(),
                translatable: entry.translatable,
                ..Default::default()
            },
        );
    }

    let ref_resource = I18nResource {
        metadata: ResourceMetadata {
            source_format: resource.metadata.source_format,
            locale: Some(locale::normalize(&source_lang)),
            source_locale: Some(locale::normalize(&source_lang)),
            ..Default::default()
        },
        entries: ref_entries,
    };

    ctx.ref_resources
        .insert(file_name.to_string(), ref_resource);
    ctx.ref_file_names.push(file_name.to_string());

    // Store the original parsed resource as the target language resource
    if let Some(tl) = target_lang {
        let normalized_tl = locale::normalize(&tl);
        if !locale::fuzzy_eq(&normalized_tl, &ctx.ref_lang) {
            ctx.lang_resources
                .entry(normalized_tl.clone())
                .or_default()
                .insert(file_name.to_string(), resource);

            if !ctx.discovered_languages.contains(&normalized_tl) {
                ctx.discovered_languages.push(normalized_tl);
            }
        }
    }
}

/// Handle multi-language single-file formats like xcstrings where the primary
/// locale entries are in the resource and additional locales are serialized
/// in entry properties as `xcstrings.localization.{locale}`.
fn discover_single_file_xcstrings(
    file_name: &str,
    resource: I18nResource,
    ctx: &mut ValidationContext,
) {
    // The source locale from xcstrings metadata
    let source_lang = resource
        .metadata
        .source_locale
        .clone()
        .or_else(|| resource.metadata.locale.clone())
        .unwrap_or_else(|| ctx.ref_lang.clone());

    ctx.ref_lang = locale::normalize(&source_lang);

    // Collect all locale codes from entry properties
    let mut locale_codes: Vec<String> = Vec::new();
    for (_key, entry) in &resource.entries {
        for prop_key in entry.properties.keys() {
            if let Some(locale_str) = prop_key.strip_prefix("xcstrings.localization.") {
                let normalized = locale::normalize(locale_str);
                if !locale::fuzzy_eq(&normalized, &ctx.ref_lang)
                    && !locale_codes.contains(&normalized)
                {
                    locale_codes.push(normalized);
                }
            }
        }
    }

    // Build the reference resource (primary locale entries as-is)
    let mut ref_entries = IndexMap::new();
    for (key, entry) in &resource.entries {
        ref_entries.insert(
            key.clone(),
            I18nEntry {
                key: key.clone(),
                value: entry.value.clone(),
                comments: entry.comments.clone(),
                translatable: entry.translatable,
                ..Default::default()
            },
        );
    }

    let ref_resource = I18nResource {
        metadata: ResourceMetadata {
            source_format: resource.metadata.source_format,
            locale: Some(ctx.ref_lang.clone()),
            source_locale: Some(ctx.ref_lang.clone()),
            ..Default::default()
        },
        entries: ref_entries,
    };

    ctx.ref_resources
        .insert(file_name.to_string(), ref_resource);
    ctx.ref_file_names.push(file_name.to_string());

    // Build a resource for each target locale from the serialized property data
    for locale_code in &locale_codes {
        let mut lang_entries = IndexMap::new();

        for (key, entry) in &resource.entries {
            // Find the matching property for this locale (try original and normalized forms)
            let prop_json = entry
                .properties
                .iter()
                .find(|(pk, _)| {
                    pk.strip_prefix("xcstrings.localization.")
                        .map(|lc| locale::fuzzy_eq(&locale::normalize(lc), locale_code))
                        .unwrap_or(false)
                })
                .map(|(_, v)| v.as_str());

            if let Some(json_str) = prop_json {
                // Parse the localization JSON to extract the value
                if let Ok(loc_data) = serde_json::from_str::<serde_json::Value>(json_str) {
                    let value = extract_xcstrings_value(&loc_data);
                    lang_entries.insert(
                        key.clone(),
                        I18nEntry {
                            key: key.clone(),
                            value,
                            translatable: entry.translatable,
                            ..Default::default()
                        },
                    );
                }
            }
            // If no property for this locale, the key is missing in that language
            // (which is correct — the missing-keys validator will flag it)
        }

        if !lang_entries.is_empty() {
            let lang_resource = I18nResource {
                metadata: ResourceMetadata {
                    source_format: resource.metadata.source_format,
                    locale: Some(locale_code.clone()),
                    source_locale: Some(ctx.ref_lang.clone()),
                    ..Default::default()
                },
                entries: lang_entries,
            };

            ctx.lang_resources
                .entry(locale_code.clone())
                .or_default()
                .insert(file_name.to_string(), lang_resource);
        }

        if !ctx.discovered_languages.contains(locale_code) {
            ctx.discovered_languages.push(locale_code.clone());
        }
    }
}

/// Extract the entry value from an xcstrings localization JSON object.
/// The JSON can contain a `stringUnit`, `variations`, or `substitutions`.
fn extract_xcstrings_value(loc_data: &serde_json::Value) -> EntryValue {
    let loc_obj = match loc_data.as_object() {
        Some(o) => o,
        None => return EntryValue::Simple(String::new()),
    };

    // Check for stringUnit (simple value)
    if let Some(su) = loc_obj.get("stringUnit") {
        if let Some(value) = su.get("value").and_then(|v| v.as_str()) {
            return EntryValue::Simple(value.to_string());
        }
    }

    // Check for variations -> plural
    if let Some(variations) = loc_obj.get("variations") {
        if let Some(plural) = variations.get("plural").and_then(|p| p.as_object()) {
            let mut ps = i18n_convert::ir::PluralSet::default();
            for (category, variant) in plural {
                let value = variant
                    .get("stringUnit")
                    .and_then(|su| su.get("value"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                match category.as_str() {
                    "zero" => ps.zero = Some(value),
                    "one" => ps.one = Some(value),
                    "two" => ps.two = Some(value),
                    "few" => ps.few = Some(value),
                    "many" => ps.many = Some(value),
                    "other" => ps.other = value,
                    _ => {}
                }
            }
            return EntryValue::Plural(ps);
        }
    }

    EntryValue::Simple(String::new())
}
