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
    let has_errors = diagnostics
        .iter()
        .any(|d| matches!(d.severity, Severity::Error));
    let has_warnings = diagnostics
        .iter()
        .any(|d| matches!(d.severity, Severity::Warning));
    let fail = has_errors || (config.strict && has_warnings);

    if args.quiet {
        return Ok(fail);
    }

    let output = match args.output_format.as_str() {
        "json" => json::render(diagnostics, ctx, config.strict)?,
        "junit" => junit::render(diagnostics, ctx)?,
        _ => terminal::render(diagnostics, ctx, args)?,
    };

    // Write output
    if let Some(ref path) = args.output {
        std::fs::write(path, &output)?;
    } else {
        print!("{output}");
    }

    Ok(fail)
}
