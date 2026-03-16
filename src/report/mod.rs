pub mod json;
pub mod junit;
pub mod terminal;

use crate::cli::Args;
use crate::config::ResolvedConfig;
use crate::diagnostic::{Diagnostic, Severity};
use crate::discovery::ValidationContext;

/// Render diagnostics in the requested format.
/// Returns true if there are actionable failures (errors, or warnings in strict mode).
pub fn render(
    diagnostics: &[Diagnostic],
    ctx: &ValidationContext,
    config: &ResolvedConfig,
    args: &Args,
) -> Result<bool, Box<dyn std::error::Error>> {
    let output = match args.output_format.as_str() {
        "json" => json::render(diagnostics, ctx)?,
        "junit" => junit::render(diagnostics, ctx)?,
        _ => terminal::render(diagnostics, ctx, args)?,
    };

    // Write output
    if let Some(ref path) = args.output {
        std::fs::write(path, &output)?;
    } else {
        print!("{output}");
    }

    // Determine if validation failed
    let error_count = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warning_count = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();

    let has_failures = error_count > 0 || (config.strict && warning_count > 0);
    Ok(has_failures)
}
