use std::collections::BTreeMap;

use colored::Colorize;

use crate::cli::Args;
use crate::diagnostic::{Diagnostic, Severity};
use crate::discovery::ValidationContext;

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Group key: (check_id_str, file_name)
type GroupKey = (String, String);

/// Per-key: key -> Vec<language>
type KeyLanguages = BTreeMap<String, Vec<String>>;

pub fn render(
    diagnostics: &[Diagnostic],
    ctx: &ValidationContext,
    args: &Args,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut out = String::new();

    if !args.quiet {
        render_header(&mut out, ctx, args);
    }

    let error_count = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warning_count = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();

    let separator = format!("{}\n", "\u{2500}".repeat(48).dimmed());

    if error_count > 0 {
        out.push('\n');
        out.push_str(&separator);
        out.push('\n');
        out.push_str(&format!(
            "{}\n",
            format!("ERRORS ({error_count})").red().bold()
        ));
        out.push('\n');
        render_grouped(&mut out, diagnostics, Severity::Error);
    }

    if warning_count > 0 && !args.quiet {
        out.push('\n');
        out.push_str(&separator);
        out.push('\n');
        out.push_str(&format!(
            "{}\n",
            format!("WARNINGS ({warning_count})").yellow().bold()
        ));
        out.push('\n');
        render_grouped(&mut out, diagnostics, Severity::Warning);
    }

    // Footer
    out.push('\n');
    out.push_str(&separator);
    out.push('\n');

    let lang_count = ctx.discovered_languages.len();
    let summary_line = format!(
        "  {error_count} error{}, {warning_count} warning{} across {lang_count} language{}\n",
        if error_count != 1 { "s" } else { "" },
        if warning_count != 1 { "s" } else { "" },
        if lang_count != 1 { "s" } else { "" },
    );
    out.push_str(&summary_line);

    if error_count > 0 {
        out.push_str(&format!("  {}\n", "x Validation failed".red().bold()));
    } else {
        out.push_str(&format!("  {}\n", "v Validation passed".green().bold()));
    }

    Ok(out)
}

fn render_header(out: &mut String, ctx: &ValidationContext, args: &Args) {
    out.push_str(&format!(
        "\n{} {} {}\n",
        "i18n-validate".bold(),
        format!("v{VERSION}").dimmed(),
        format!("-- validating {}", args.path).dimmed()
    ));
    out.push('\n');

    // Reference info
    let ref_file_count = ctx.ref_resources.len();
    let ref_file_names: Vec<String> = ctx.ref_file_names.clone();
    out.push_str(&format!(
        "  {}: {} ({} file{}: {})\n",
        "Reference".bold(),
        ctx.ref_lang,
        ref_file_count,
        if ref_file_count != 1 { "s" } else { "" },
        ref_file_names.join(", ")
    ));

    // Languages info
    let lang_count = ctx.discovered_languages.len();
    let expected_count = ctx.config.expected_languages.len();
    let langs_str = ctx.discovered_languages.join(", ");
    if expected_count > 0 {
        out.push_str(&format!(
            "  {}: {} ({} found, {} expected)\n",
            "Languages".bold(),
            langs_str,
            lang_count,
            expected_count
        ));
    } else {
        out.push_str(&format!(
            "  {}: {} ({} found)\n",
            "Languages".bold(),
            langs_str,
            lang_count
        ));
    }

    // Layout info
    let layout_str = ctx.layout.as_str();
    let layout_note = if ctx.config.layout.is_some() {
        "configured"
    } else {
        "auto-detected"
    };
    out.push_str(&format!(
        "  {}: {} ({})\n",
        "Layout   ".bold(),
        layout_str,
        layout_note
    ));

    // Formats info
    if !ctx.discovered_formats.is_empty() {
        let format_names: Vec<&str> = ctx.discovered_formats.iter().map(|s| s.as_str()).collect();
        out.push_str(&format!(
            "  {}: {} (auto-detected)\n",
            "Formats  ".bold(),
            format_names.join(", ")
        ));
    }
}

fn render_grouped(out: &mut String, diagnostics: &[Diagnostic], severity: Severity) {
    // Group by (check, file), then by key, collecting languages per key
    let mut groups: BTreeMap<GroupKey, KeyLanguages> = BTreeMap::new();

    // Also track key-less diagnostics (like missing-languages)
    let mut keyless: BTreeMap<GroupKey, Vec<String>> = BTreeMap::new();

    for d in diagnostics {
        if d.severity != severity {
            continue;
        }

        let check = d.check.as_str().to_string();
        let file = d.file.clone().unwrap_or_default();
        let group_key = (check, file);

        if let Some(ref key) = d.key {
            groups
                .entry(group_key)
                .or_default()
                .entry(key.clone())
                .or_default()
                .push(d.language.clone());
        } else {
            keyless
                .entry(group_key)
                .or_default()
                .push(d.message.clone());
        }
    }

    let icon = match severity {
        Severity::Error => "x".red().bold().to_string(),
        Severity::Warning => "!".yellow().bold().to_string(),
    };

    // Render keyless diagnostics
    for ((check, file), messages) in &keyless {
        if file.is_empty() {
            out.push_str(&format!("  {} {}\n", icon, check.bold()));
        } else {
            out.push_str(&format!(
                "  {} {} {} {}\n",
                icon,
                check.bold(),
                "|".dimmed(),
                file
            ));
        }

        for msg in messages {
            out.push_str(&format!("    {msg}\n"));
        }
        out.push('\n');
    }

    // Render keyed diagnostics
    for ((check, file), keys) in &groups {
        if file.is_empty() {
            out.push_str(&format!("  {} {}\n", icon, check.bold()));
        } else {
            out.push_str(&format!(
                "  {} {} {} {}\n",
                icon,
                check.bold(),
                "|".dimmed(),
                file
            ));
        }

        for (key, langs) in keys {
            let mut unique_langs = langs.clone();
            unique_langs.sort();
            unique_langs.dedup();

            let verb = match severity {
                Severity::Error => {
                    if check == "missing-keys" {
                        "missing in"
                    } else if check == "extra-keys" {
                        "extra in"
                    } else if check == "placeholders" {
                        "mismatched in"
                    } else if check == "plural-structure" {
                        "invalid in"
                    } else {
                        "in"
                    }
                }
                Severity::Warning => {
                    if check == "empty-values" {
                        "empty in"
                    } else if check == "untranslated" {
                        "untranslated in"
                    } else {
                        "in"
                    }
                }
            };

            out.push_str(&format!(
                "    Key \"{key}\"\n      {verb}: {}\n",
                unique_langs.join(", ")
            ));
        }
        out.push('\n');
    }
}
