# i18n-validate

[![CI](https://github.com/i18n-agent/i18n-validate/actions/workflows/ci.yml/badge.svg)](https://github.com/i18n-agent/i18n-validate/actions/workflows/ci.yml)

**Validate i18n translation files for consistency across 32 formats.**

Catch missing keys, broken placeholders, orphaned languages, and malformed files — before your users do. Works with any i18n format, zero-config, and designed for CI/CD.

## The Problem

Translation issues are caught at runtime when users see broken UI:

- Missing keys → blank text or key paths displayed to users
- Broken placeholders → `{price}` shows up as literal text instead of `$9.99`
- Orphaned files → outdated translations confuse the build system
- Malformed JSON/XML → runtime crashes when users switch languages

Most validation tools are format-specific (JSON-only, YAML-only) and require custom test code.

## The Solution

`i18n-validate` uses [i18n-convert](https://github.com/i18n-agent/i18n-convert) to parse **32 i18n formats** into a common representation, then runs consistency checks across all your translations. One tool, any format, zero-config.

## Features

- **9 validation checks** — missing keys, extra keys, broken placeholders, malformed plurals, empty values, untranslated strings, and more
- **32 formats** — JSON, YAML, XLIFF, Android XML, iOS Strings, Gettext PO, and [27 more](#supported-formats)
- **3 output modes** — colored terminal, JSON, JUnit XML (for CI test reporters)
- **Zero-config** — auto-detects layout, format, and reference language
- **Configurable** — `.i18n-validate.toml` for per-check severity overrides, per-language exceptions, and file filtering
- **Locale-aware** — normalizes `zh_Hans`, `zh-Hans`, `zh-hans` to the same code
- **Fast** — native Rust binary, processes thousands of files in milliseconds

## Installation

### npm (recommended for CI)

```bash
npm install -g @i18n-agent/i18n-validate
```

### Homebrew (macOS / Linux)

```bash
brew tap i18n-agent/tap
brew install i18n-validate
```

### Binary download

Download pre-built binaries from [GitHub Releases](https://github.com/i18n-agent/i18n-validate/releases):

| Platform | Architecture | Download |
|----------|-------------|----------|
| macOS | Apple Silicon (M1/M2/M3) | `i18n-validate-aarch64-apple-darwin.tar.gz` |
| macOS | Intel | `i18n-validate-x86_64-apple-darwin.tar.gz` |
| Linux | x86_64 | `i18n-validate-x86_64-unknown-linux-gnu.tar.gz` |
| Linux | ARM64 | `i18n-validate-aarch64-unknown-linux-gnu.tar.gz` |
| Windows | x86_64 | `i18n-validate-x86_64-pc-windows-msvc.zip` |

### Build from source

```bash
cargo install --git https://github.com/i18n-agent/i18n-validate
```

## Quick Start

### Step 1: Run validation (zero-config)

```bash
i18n-validate ./locales
```

That's it. The tool auto-detects your directory layout and file format, uses `en` as the reference language, and validates all other languages against it.

### Step 2: See the report

```
i18n-validate v0.1.0 — validating ./locales

  Reference: en (2 files: translation.json, apiTester.json)
  Languages: de, ja, fr, es, zh-Hans, pt-BR, ko (7 found, 7 expected)
  Layout   : directory (auto-detected)
  Formats  : i18next JSON (auto-detected)

────────────────────────────────────────────────

ERRORS (5)

  ✗ missing-keys │ translation.json
    Key "settings.billing.title"
      missing in: ja, ko
    Key "settings.billing.description"
      missing in: ko

  ✗ placeholders │ translation.json
    Key "pricing.total" — expected: {price}, {currency}
      de:  found {price} only — missing {currency}

────────────────────────────────────────────────

WARNINGS (2)

  ⚠ empty-values │ translation.json
    Key "onboarding.step3.hint"
      empty in: de, fr

  ⚠ untranslated │ apiTester.json
    Key "errors.timeout"
      untranslated in: es, pt-BR

────────────────────────────────────────────────

  5 errors, 2 warnings across 7 languages
  ✗ Validation failed
```

### Step 3: Add to CI/CD

**GitHub Actions:**

```yaml
- name: Validate translations
  run: npx @i18n-agent/i18n-validate ./locales --format junit -o i18n-report.xml

- name: Upload test report
  uses: dorny/test-reporter@v1
  if: always()
  with:
    name: i18n validation
    path: i18n-report.xml
    reporter: java-junit
```

**GitLab CI:**

```yaml
validate-i18n:
  script:
    - npx @i18n-agent/i18n-validate ./locales --format junit -o i18n-report.xml
  artifacts:
    reports:
      junit: i18n-report.xml
```

### Step 4: Customize (optional)

Create `.i18n-validate.toml` in your project root:

```toml
ref = "en"
expect = ["de", "ja", "fr", "es", "zh-Hans", "pt-BR", "ko"]

[checks]
empty-values = "off"          # Don't check for empty values
untranslated = "off"          # Don't check for untranslated strings

[languages.ko]
missing-keys = "warning"      # Korean is WIP, don't fail CI

[languages.ar]
skip = true                   # Exclude Arabic from validation
```

## Validation Checks

### Errors (default)

| Check | ID | What it catches |
|-------|-----|----------------|
| Missing languages | `missing-languages` | Expected language has no files/directory |
| Orphaned languages | `orphaned-languages` | Language files/directory exist but isn't in the expected list |
| Missing keys | `missing-keys` | Key exists in reference but is absent in translation |
| Extra keys | `extra-keys` | Key exists in translation but not in reference |
| Placeholder mismatch | `placeholders` | `{price}`, `{{name}}`, `%s` differ between languages |
| Plural structure | `plural-structure` | Plural forms are malformed or missing required categories |
| Parse errors | `parse-errors` | File fails to parse (broken JSON, XML, YAML, etc.) |

### Warnings (default)

| Check | ID | What it catches |
|-------|-----|----------------|
| Empty values | `empty-values` | Key is present but the translation is an empty string |
| Untranslated | `untranslated` | Translation is identical to the reference language (likely copy-paste) |

## CLI Reference

```
i18n-validate [OPTIONS] <PATH>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<PATH>` | Path to locales directory or single translation file |

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `--ref <LANG>` | Reference language code | `en` |
| `--expect <LANGS>` | Comma-separated expected language codes | auto-discover |
| `--layout <TYPE>` | Layout: `flat`, `directory`, `single-file` | auto-detect |
| `--include <PATTERN>` | Include file patterns (glob, repeatable) | all files |
| `--exclude <PATTERN>` | Exclude file patterns (glob, repeatable) | none |
| `--format <FORMAT>` | Output: `terminal`, `json`, `junit` | `terminal` |
| `-o, --output <FILE>` | Write output to file | stdout |
| `--strict` | Treat warnings as errors (exit 1) | false |
| `--no-warnings` | Suppress all warnings | false |
| `--skip <CHECKS>` | Comma-separated checks to skip | none |
| `--quiet` | Suppress all output, rely on exit code | false |
| `--config <PATH>` | Path to config file | auto-detect |

### Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Validation passed |
| `1` | Errors found (or warnings with `--strict`) |
| `2` | Bad arguments or configuration |

## Config File

`.i18n-validate.toml` — auto-detected in project root or any parent directory.

```toml
# Reference language (source of truth)
ref = "en"

# Expected languages (omit to auto-discover)
expect = ["de", "ja", "fr", "es", "zh-Hans", "pt-BR", "ko"]

# Layout override (omit for auto-detect)
# layout = "flat"           # en.json, de.json
# layout = "directory"      # en/translation.json
# layout = "single-file"    # messages.xliff

# File patterns (glob, relative to target directory)
include = ["*.json", "*.yaml"]
exclude = ["_old/**", "draft/**"]

# Suppress all warnings
no_warnings = false

# Per-check severity: "error", "warning", or "off"
[checks]
missing-keys = "error"
extra-keys = "error"
placeholders = "error"
plural-structure = "error"
missing-languages = "error"
orphaned-languages = "error"
parse-errors = "error"
empty-values = "warning"
untranslated = "warning"

# Per-language overrides
[languages.ko]
missing-keys = "warning"       # Korean is WIP

[languages.ar]
skip = true                    # Exclude from validation
```

## Supported Formats

Powered by [i18n-convert](https://github.com/i18n-agent/i18n-convert), i18n-validate supports **32 formats**:

### Mobile & Desktop
Android XML, Xcode String Catalog (`.xcstrings`), iOS Strings (`.strings`), iOS Stringsdict, iOS Property List, Flutter ARB, Qt Linguist (`.ts`)

### Web & Frameworks
Structured JSON, i18next JSON, JSON5, HJSON, YAML (Rails), YAML (Plain), JavaScript, TypeScript, PHP/Laravel, NEON

### Standards & Exchange
XLIFF 1.2, XLIFF 2.0, Gettext PO, TMX, .NET RESX, Java Properties

### Data & Other
CSV, Excel (`.xlsx`), TOML, INI, SRT Subtitles, Markdown, Plain Text

### Vendor-Specific
iSpring Suite XLIFF, Adobe Captivate XML

## Directory Layouts

The tool auto-detects your project's directory structure:

| Layout | Structure | Example |
|--------|-----------|---------|
| `directory` | One directory per language | `locales/en/translation.json` |
| `flat` | One file per language | `locales/en.json` |
| `single-file` | All languages in one file | `Localizable.xcstrings` |

## Locale Code Handling

The tool normalizes locale codes automatically:

| Your files | Normalized | Matched |
|------------|-----------|---------|
| `zh_Hans` | `zh-Hans` | ✓ |
| `zh-hans` | `zh-Hans` | ✓ |
| `values-zh-rCN` | `zh-CN` | ✓ (Android) |
| `zh-Hans.lproj` | `zh-Hans` | ✓ (iOS) |
| `messages_pt_BR.properties` | `pt-BR` | ✓ (Java) |

## Examples

### Validate a React project (i18next)

```bash
i18n-validate ./public/locales
```

### Validate an Android project

```bash
i18n-validate ./app/src/main/res --layout directory
```

### Validate with JSON output for scripting

```bash
i18n-validate ./locales --format json | jq '.summary'
```

### Skip checks for a new language being onboarded

```bash
i18n-validate ./locales --skip untranslated,empty-values
```

### Strict mode for production branches

```bash
i18n-validate ./locales --strict
```

## Built by

Built by [i18nagent.ai](https://i18nagent.ai) — AI-powered localization for developers. We build open-source tools that make internationalization easier for everyone.

See also:
- [i18n-convert](https://github.com/i18n-agent/i18n-convert) — Convert between 32 i18n file formats
- [i18n-pseudo](https://github.com/i18n-agent/i18n-pseudo) — Pseudo-translate files for i18n testing

## License

MIT — see [LICENSE](LICENSE).
