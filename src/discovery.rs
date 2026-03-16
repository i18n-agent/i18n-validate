use std::collections::HashMap;
use std::path::Path;

use glob::Pattern;
use i18n_convert::detect::detect_best;
use i18n_convert::formats::FormatRegistry;
use i18n_convert::ir::I18nResource;

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
        let included = include
            .iter()
            .any(|pat| Pattern::new(pat).map(|p| p.matches(filename)).unwrap_or(false));
        if !included {
            return false;
        }
    }

    if !exclude.is_empty() {
        let excluded = exclude
            .iter()
            .any(|pat| Pattern::new(pat).map(|p| p.matches(filename)).unwrap_or(false));
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
                        ctx.ref_resources
                            .insert(file_name_str.clone(), resource);
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
    let entries: Vec<_> = std::fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .collect();

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

    let canonical_key = ref_file_name.clone().unwrap_or_else(|| "translations".to_string());

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
                    ctx.ref_resources
                        .insert(canonical_key.clone(), resource);
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
                    Diagnostic::error(CheckId::ParseErrors, &lang_code, e)
                        .with_file(file_name_str),
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
                ctx.discovered_formats.push(format_id);
            }
            // For single-file formats, we store everything as reference
            ctx.ref_resources.insert(file_name, resource);
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
