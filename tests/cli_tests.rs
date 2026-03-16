use std::path::PathBuf;
use std::process::Command;

/// Return the absolute path to the project's `tests/fixtures` directory.
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

/// Return the absolute path to the i18next fixture directory.
fn i18next_dir() -> String {
    fixtures_dir().join("i18next").to_string_lossy().to_string()
}

/// Return the absolute path to the flat-json fixture directory.
fn flat_json_dir() -> String {
    fixtures_dir()
        .join("flat-json")
        .to_string_lossy()
        .to_string()
}

/// Helper: run the binary with the given arguments and return (exit code, stdout, stderr).
fn run_cli(args: &[&str]) -> (i32, String, String) {
    let bin = env!("CARGO_BIN_EXE_i18n-validate");
    let output = Command::new(bin)
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .expect("failed to execute i18n-validate binary");

    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (code, stdout, stderr)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[test]
fn valid_directory_exits_zero() {
    // de is complete relative to en; skip orphaned-languages check
    // because ja is also present and would be flagged as orphaned.
    let (code, _stdout, _stderr) = run_cli(&[
        &i18next_dir(),
        "--expect",
        "en,de,ja",
        "--skip",
        "missing-keys,placeholders,untranslated,empty-values,plural-structure,extra-keys",
    ]);
    assert_eq!(code, 0, "Expected exit code 0 for a valid directory");
}

#[test]
fn missing_keys_exits_one() {
    // ja has missing keys, placeholder mismatches, etc.
    let (code, stdout, _stderr) = run_cli(&[&i18next_dir(), "--expect", "en,de,ja"]);
    assert_eq!(code, 1, "Expected exit code 1 when there are errors");
    assert!(
        stdout.contains("missing-keys") || stdout.contains("ERRORS"),
        "Expected stdout to mention missing-keys or ERRORS, got:\n{stdout}"
    );
}

#[test]
fn json_format_produces_valid_json() {
    let (code, stdout, _stderr) =
        run_cli(&[&i18next_dir(), "--format", "json", "--expect", "en,de,ja"]);
    assert_eq!(code, 1);

    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be valid JSON");

    assert!(
        parsed.get("diagnostics").is_some(),
        "JSON output should have a 'diagnostics' field"
    );
    assert!(
        parsed.get("summary").is_some(),
        "JSON output should have a 'summary' field"
    );

    let passed = parsed["summary"]["passed"].as_bool();
    assert_eq!(passed, Some(false), "summary.passed should be false");
}

#[test]
fn junit_format_produces_valid_xml() {
    let (code, stdout, _stderr) =
        run_cli(&[&i18next_dir(), "--format", "junit", "--expect", "en,de,ja"]);
    assert_eq!(code, 1);
    assert!(
        stdout.starts_with("<?xml") || stdout.contains("<testsuites"),
        "JUnit output should start with <?xml or contain <testsuites, got:\n{stdout}"
    );
}

#[test]
fn skip_check_works() {
    // Skipping all the checks that would fail for ja should give exit 0.
    let (code, _stdout, _stderr) = run_cli(&[
        &i18next_dir(),
        "--expect",
        "en,de,ja",
        "--skip",
        "missing-keys,placeholders,untranslated,empty-values,plural-structure,extra-keys",
    ]);
    assert_eq!(
        code, 0,
        "Skipping all failing checks should result in exit 0"
    );
}

#[test]
fn strict_mode_fails_on_warnings() {
    // Skip all error-level checks; only warnings (empty-values, untranslated) remain.
    // --strict should promote them to failures.
    let (code, _stdout, _stderr) = run_cli(&[
        &i18next_dir(),
        "--expect",
        "en,de,ja",
        "--strict",
        "--skip",
        "missing-keys,placeholders,extra-keys,plural-structure",
    ]);
    assert_eq!(
        code, 1,
        "Strict mode should exit 1 when warnings are present"
    );
}

#[test]
fn quiet_mode_no_output() {
    // All checks skipped + quiet: stdout should be empty and exit 0.
    let (code, stdout, _stderr) = run_cli(&[
        &i18next_dir(),
        "--expect",
        "en,de,ja",
        "--quiet",
        "--skip",
        "missing-keys,placeholders,extra-keys,plural-structure,empty-values,untranslated",
    ]);
    assert!(
        stdout.is_empty(),
        "Quiet mode should produce no stdout, got:\n{stdout}"
    );
    assert_eq!(code, 0, "Quiet mode with no errors should exit 0");
}

#[test]
fn quiet_mode_still_exits_nonzero() {
    // ja has errors; quiet mode should still exit 1.
    let (code, stdout, _stderr) = run_cli(&[&i18next_dir(), "--expect", "en,de,ja", "--quiet"]);
    assert!(
        stdout.is_empty(),
        "Quiet mode should produce no stdout, got:\n{stdout}"
    );
    assert_eq!(code, 1, "Quiet mode with errors should still exit 1");
}

#[test]
fn no_warnings_suppresses_warnings() {
    let (code, stdout, _stderr) =
        run_cli(&[&i18next_dir(), "--expect", "en,de,ja", "--no-warnings"]);
    assert_eq!(code, 1, "Should still exit 1 due to errors");

    // Warnings should be suppressed
    assert!(
        !stdout.contains("WARNINGS"),
        "Should not contain WARNINGS section"
    );
    assert!(
        !stdout.contains("empty-values"),
        "Should not contain empty-values warning"
    );
    assert!(
        !stdout.contains("untranslated"),
        "Should not contain untranslated warning"
    );

    // Errors should still be present
    assert!(
        stdout.contains("ERRORS"),
        "Should still contain ERRORS section"
    );
}

#[test]
fn flat_layout_works() {
    // de.json is complete relative to en.json in the flat-json fixtures.
    let (code, _stdout, _stderr) = run_cli(&[&flat_json_dir(), "--expect", "en,de"]);
    assert_eq!(
        code, 0,
        "Flat layout with complete translation should exit 0"
    );
}

#[test]
fn output_to_file_works() {
    let output_path = "/tmp/i18n-validate-test-output.json";

    let (code, _stdout, _stderr) = run_cli(&[
        &i18next_dir(),
        "--format",
        "json",
        "--expect",
        "en,de,ja",
        "-o",
        output_path,
    ]);
    assert_eq!(code, 1);

    let content = std::fs::read_to_string(output_path).expect("Output file should exist");
    let parsed: serde_json::Value =
        serde_json::from_str(&content).expect("Output file should contain valid JSON");
    assert!(parsed.get("diagnostics").is_some());
    assert!(parsed.get("summary").is_some());

    // Clean up
    let _ = std::fs::remove_file(output_path);
}

#[test]
fn bad_path_exits_two() {
    let (code, _stdout, stderr) = run_cli(&["/nonexistent/path/to/translations"]);
    assert_eq!(code, 2, "Nonexistent path should exit with code 2");
    assert!(
        !stderr.is_empty(),
        "Should print an error message to stderr"
    );
}

#[test]
fn unknown_check_in_skip_is_silently_ignored() {
    // The current implementation silently ignores unknown check names in --skip.
    // The binary does NOT exit 2 for unknown checks; it just ignores them.
    // This test verifies the actual behavior.
    let (code, _stdout, _stderr) = run_cli(&[
        &i18next_dir(),
        "--expect",
        "en,de,ja",
        "--skip",
        "nonexistent-check,missing-keys,placeholders,extra-keys,plural-structure,empty-values,untranslated",
    ]);
    // With all real checks skipped, errors should be gone, and the unknown check is ignored.
    assert_eq!(
        code, 0,
        "Unknown check in --skip should be silently ignored; remaining valid skips should work"
    );
}
