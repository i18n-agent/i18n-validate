use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Severity {
    Error,
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckId {
    MissingLanguages,
    OrphanedLanguages,
    MissingKeys,
    ExtraKeys,
    Placeholders,
    PluralStructure,
    ParseErrors,
    EmptyValues,
    Untranslated,
}

impl CheckId {
    pub fn as_str(&self) -> &'static str {
        match self {
            CheckId::MissingLanguages => "missing-languages",
            CheckId::OrphanedLanguages => "orphaned-languages",
            CheckId::MissingKeys => "missing-keys",
            CheckId::ExtraKeys => "extra-keys",
            CheckId::Placeholders => "placeholders",
            CheckId::PluralStructure => "plural-structure",
            CheckId::ParseErrors => "parse-errors",
            CheckId::EmptyValues => "empty-values",
            CheckId::Untranslated => "untranslated",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "missing-languages" => Some(CheckId::MissingLanguages),
            "orphaned-languages" => Some(CheckId::OrphanedLanguages),
            "missing-keys" => Some(CheckId::MissingKeys),
            "extra-keys" => Some(CheckId::ExtraKeys),
            "placeholders" => Some(CheckId::Placeholders),
            "plural-structure" => Some(CheckId::PluralStructure),
            "parse-errors" => Some(CheckId::ParseErrors),
            "empty-values" => Some(CheckId::EmptyValues),
            "untranslated" => Some(CheckId::Untranslated),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn default_severity(&self) -> Severity {
        match self {
            CheckId::EmptyValues | CheckId::Untranslated => Severity::Warning,
            _ => Severity::Error,
        }
    }

    #[allow(dead_code)]
    pub fn all() -> &'static [CheckId] {
        &[
            CheckId::MissingLanguages,
            CheckId::OrphanedLanguages,
            CheckId::MissingKeys,
            CheckId::ExtraKeys,
            CheckId::Placeholders,
            CheckId::PluralStructure,
            CheckId::ParseErrors,
            CheckId::EmptyValues,
            CheckId::Untranslated,
        ]
    }
}

impl fmt::Display for CheckId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub check: CheckId,
    pub file: Option<String>,
    pub key: Option<String>,
    pub language: String,
    pub message: String,
    pub expected: Option<String>,
    pub found: Option<String>,
}

impl Diagnostic {
    pub fn error(check: CheckId, language: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            check,
            file: None,
            key: None,
            language: language.into(),
            message: message.into(),
            expected: None,
            found: None,
        }
    }

    pub fn warning(
        check: CheckId,
        language: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity: Severity::Warning,
            check,
            file: None,
            key: None,
            language: language.into(),
            message: message.into(),
            expected: None,
            found: None,
        }
    }

    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
        self.expected = Some(expected.into());
        self
    }

    pub fn with_found(mut self, found: impl Into<String>) -> Self {
        self.found = Some(found.into());
        self
    }
}
