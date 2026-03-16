use std::path::Path;

use crate::locale;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Flat,
    Directory,
    SingleFile,
}

impl Layout {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "flat" => Some(Layout::Flat),
            "directory" | "dir" => Some(Layout::Directory),
            "single-file" | "single" | "singlefile" => Some(Layout::SingleFile),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Layout::Flat => "flat",
            Layout::Directory => "directory",
            Layout::SingleFile => "single-file",
        }
    }
}

impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Detect the layout of translation files at the given path.
/// If `override_layout` is provided, use it directly.
pub fn detect(
    path: &Path,
    override_layout: Option<Layout>,
) -> Result<Layout, Box<dyn std::error::Error>> {
    if let Some(layout) = override_layout {
        return Ok(layout);
    }

    if path.is_file() {
        return Ok(Layout::SingleFile);
    }

    if !path.is_dir() {
        return Err(format!("Path does not exist: {}", path.display()).into());
    }

    let mut has_locale_dirs = false;
    let mut has_locale_files = false;

    let entries = std::fs::read_dir(path)?;
    for entry in entries {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if file_type.is_dir() {
            if locale::extract_from_path(&name_str).is_some() {
                has_locale_dirs = true;
            }
        } else if file_type.is_file()
            && locale::extract_from_filename(&name_str).is_some()
        {
            has_locale_files = true;
        }
    }

    if has_locale_dirs {
        Ok(Layout::Directory)
    } else if has_locale_files {
        Ok(Layout::Flat)
    } else {
        Err("Could not detect layout: no locale directories or locale-named files found".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_from_str() {
        assert_eq!(Layout::parse("flat"), Some(Layout::Flat));
        assert_eq!(Layout::parse("directory"), Some(Layout::Directory));
        assert_eq!(Layout::parse("dir"), Some(Layout::Directory));
        assert_eq!(Layout::parse("single-file"), Some(Layout::SingleFile));
        assert_eq!(Layout::parse("unknown"), None);
    }

    #[test]
    fn test_layout_display() {
        assert_eq!(Layout::Flat.to_string(), "flat");
        assert_eq!(Layout::Directory.to_string(), "directory");
        assert_eq!(Layout::SingleFile.to_string(), "single-file");
    }
}
