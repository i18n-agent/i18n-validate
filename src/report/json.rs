use serde::Serialize;

use crate::diagnostic::{Diagnostic, Severity};
use crate::discovery::ValidationContext;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize)]
struct JsonReport {
    version: String,
    ref_lang: String,
    languages: Vec<String>,
    layout: String,
    diagnostics: Vec<JsonDiagnostic>,
    summary: JsonSummary,
}

#[derive(Serialize)]
struct JsonDiagnostic {
    severity: String,
    check: String,
    file: Option<String>,
    key: Option<String>,
    language: String,
    message: String,
    expected: Option<String>,
    found: Option<String>,
}

#[derive(Serialize)]
struct JsonSummary {
    errors: usize,
    warnings: usize,
    passed: bool,
}

pub fn render(
    diagnostics: &[Diagnostic],
    ctx: &ValidationContext,
) -> Result<String, Box<dyn std::error::Error>> {
    let error_count = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warning_count = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();

    let report = JsonReport {
        version: VERSION.to_string(),
        ref_lang: ctx.ref_lang.clone(),
        languages: ctx.discovered_languages.clone(),
        layout: ctx.layout.to_string(),
        diagnostics: diagnostics
            .iter()
            .map(|d| JsonDiagnostic {
                severity: d.severity.to_string(),
                check: d.check.to_string(),
                file: d.file.clone(),
                key: d.key.clone(),
                language: d.language.clone(),
                message: d.message.clone(),
                expected: d.expected.clone(),
                found: d.found.clone(),
            })
            .collect(),
        summary: JsonSummary {
            errors: error_count,
            warnings: warning_count,
            passed: error_count == 0,
        },
    };

    let json = serde_json::to_string_pretty(&report)?;
    Ok(json)
}
