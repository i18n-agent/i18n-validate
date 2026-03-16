use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::cli::Args;
use crate::diagnostic::{CheckId, Severity};
use crate::layout::Layout;

/// Per-language configuration overrides.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct LanguageConfig {
    pub skip: Option<bool>,
    #[serde(flatten)]
    pub check_overrides: HashMap<String, String>,
}

/// Raw TOML file structure.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TomlConfig {
    #[serde(rename = "ref")]
    pub ref_lang: Option<String>,
    pub expect: Option<Vec<String>>,
    pub layout: Option<String>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
    pub no_warnings: Option<bool>,
    pub strict: Option<bool>,
    pub skip: Option<Vec<String>>,
    pub checks: Option<HashMap<String, String>>,
    pub languages: Option<HashMap<String, LanguageConfig>>,
}

/// Resolved configuration after merging CLI + TOML.
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub ref_lang: String,
    pub expected_languages: Vec<String>,
    pub layout: Option<Layout>,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub no_warnings: bool,
    pub strict: bool,
    pub skip_checks: Vec<CheckId>,
    pub check_severity: HashMap<CheckId, SeverityOverride>,
    pub language_configs: HashMap<String, LanguageConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeverityOverride {
    Error,
    Warning,
    Off,
}

impl SeverityOverride {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "error" => Some(SeverityOverride::Error),
            "warning" | "warn" => Some(SeverityOverride::Warning),
            "off" | "none" | "disable" => Some(SeverityOverride::Off),
            _ => None,
        }
    }

    pub fn to_severity(self) -> Option<Severity> {
        match self {
            SeverityOverride::Error => Some(Severity::Error),
            SeverityOverride::Warning => Some(Severity::Warning),
            SeverityOverride::Off => None,
        }
    }
}

/// Find a `.i18n-validate.toml` by walking up from the given path.
fn find_config_file(start: &Path) -> Option<std::path::PathBuf> {
    let mut dir = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };

    loop {
        let candidate = dir.join(".i18n-validate.toml");
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

/// Load and parse a TOML config file.
fn load_toml(path: &Path) -> Result<TomlConfig, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let config: TomlConfig = toml::from_str(&content)?;
    Ok(config)
}

/// Resolve configuration by merging CLI args with optional TOML config.
/// CLI values take precedence over TOML values.
pub fn resolve(args: &Args) -> Result<ResolvedConfig, Box<dyn std::error::Error>> {
    let path = Path::new(&args.path);

    // Load TOML config
    let toml_config = if let Some(ref config_path) = args.config {
        Some(load_toml(Path::new(config_path))?)
    } else {
        find_config_file(path).and_then(|p| load_toml(&p).ok())
    };

    let toml = toml_config.unwrap_or_default();

    // Merge: CLI > TOML > defaults
    let ref_lang = if args.ref_lang != "en" {
        args.ref_lang.clone()
    } else {
        toml.ref_lang.unwrap_or_else(|| args.ref_lang.clone())
    };

    let expected_languages = if !args.expect.is_empty() {
        args.expect.clone()
    } else {
        toml.expect.unwrap_or_default()
    };

    let layout = if let Some(ref layout_str) = args.layout {
        Some(
            Layout::parse(layout_str)
                .ok_or_else(|| format!("Unknown layout: {layout_str}"))?,
        )
    } else if let Some(ref layout_str) = toml.layout {
        Some(
            Layout::parse(layout_str)
                .ok_or_else(|| format!("Unknown layout in config: {layout_str}"))?,
        )
    } else {
        None
    };

    let include = if !args.include.is_empty() {
        args.include.clone()
    } else {
        toml.include.unwrap_or_default()
    };

    let exclude = if !args.exclude.is_empty() {
        args.exclude.clone()
    } else {
        toml.exclude.unwrap_or_default()
    };

    let no_warnings = args.no_warnings || toml.no_warnings.unwrap_or(false);
    let strict = args.strict || toml.strict.unwrap_or(false);

    // Merge skip checks
    let mut skip_checks = Vec::new();
    let skip_strs: Vec<&str> = if !args.skip.is_empty() {
        args.skip.iter().map(|s| s.as_str()).collect()
    } else {
        toml.skip
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default()
    };
    for s in skip_strs {
        if let Some(check) = CheckId::parse(s) {
            skip_checks.push(check);
        }
    }

    // Merge check severity overrides
    let mut check_severity = HashMap::new();
    if let Some(ref checks) = toml.checks {
        for (key, value) in checks {
            if let Some(check_id) = CheckId::parse(key) {
                if let Some(sev) = SeverityOverride::parse(value) {
                    check_severity.insert(check_id, sev);
                }
            }
        }
    }

    let language_configs = toml.languages.unwrap_or_default();

    Ok(ResolvedConfig {
        ref_lang,
        expected_languages,
        layout,
        include,
        exclude,
        no_warnings,
        strict,
        skip_checks,
        check_severity,
        language_configs,
    })
}
