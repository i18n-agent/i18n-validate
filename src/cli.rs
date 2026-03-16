use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "i18n-validate")]
#[command(version)]
#[command(about = "Validate i18n translation files for consistency across 32 formats")]
pub struct Args {
    /// Path to the translation files directory or single file
    pub path: String,

    /// Reference language code (default: en)
    #[arg(long = "ref")]
    pub ref_lang: Option<String>,

    /// Expected languages (comma-separated, e.g., "de,ja,fr")
    #[arg(long, value_delimiter = ',')]
    pub expect: Vec<String>,

    /// Layout override: flat, directory, or single-file
    #[arg(long)]
    pub layout: Option<String>,

    /// Include glob patterns (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub include: Vec<String>,

    /// Exclude glob patterns (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub exclude: Vec<String>,

    /// Output format: terminal, json, or junit
    #[arg(long = "format", default_value = "terminal")]
    pub output_format: String,

    /// Output file path (default: stdout)
    #[arg(short = 'o', long)]
    pub output: Option<String>,

    /// Treat warnings as errors
    #[arg(long)]
    pub strict: bool,

    /// Suppress warnings
    #[arg(long)]
    pub no_warnings: bool,

    /// Skip specific checks (comma-separated)
    #[arg(long, value_delimiter = ',')]
    pub skip: Vec<String>,

    /// Quiet mode (only output errors)
    #[arg(short, long)]
    pub quiet: bool,

    /// Path to config file (default: auto-detect .i18n-validate.toml)
    #[arg(long)]
    pub config: Option<String>,
}

pub fn parse() -> Args {
    Args::parse()
}
