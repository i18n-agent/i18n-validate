use std::collections::BTreeMap;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;

use crate::diagnostic::{Diagnostic, Severity};
use crate::discovery::ValidationContext;

pub fn render(
    diagnostics: &[Diagnostic],
    ctx: &ValidationContext,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut writer = Writer::new_with_indent(&mut buffer, b' ', 2);

    // XML declaration
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    // Group diagnostics by check type (each becomes a testsuite)
    let mut suites: BTreeMap<String, Vec<&Diagnostic>> = BTreeMap::new();
    for d in diagnostics {
        suites
            .entry(d.check.as_str().to_string())
            .or_default()
            .push(d);
    }

    // <testsuites>
    let total_tests = diagnostics.len();
    let total_failures = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();

    let mut testsuites = BytesStart::new("testsuites");
    testsuites.push_attribute(("name", "i18n-validate"));
    testsuites.push_attribute(("tests", total_tests.to_string().as_str()));
    testsuites.push_attribute(("failures", total_failures.to_string().as_str()));
    writer.write_event(Event::Start(testsuites))?;

    for (check_name, diags) in &suites {
        let suite_failures = diags
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count();

        let mut testsuite = BytesStart::new("testsuite");
        testsuite.push_attribute(("name", check_name.as_str()));
        testsuite.push_attribute(("tests", diags.len().to_string().as_str()));
        testsuite.push_attribute(("failures", suite_failures.to_string().as_str()));
        writer.write_event(Event::Start(testsuite))?;

        for d in diags {
            let case_name = format!(
                "{} [{}] {}",
                d.language,
                d.file.as_deref().unwrap_or(""),
                d.key.as_deref().unwrap_or("")
            );

            let mut testcase = BytesStart::new("testcase");
            testcase.push_attribute(("name", case_name.as_str()));
            testcase.push_attribute((
                "classname",
                format!("i18n-validate.{}", ctx.ref_lang).as_str(),
            ));
            writer.write_event(Event::Start(testcase))?;

            if d.severity == Severity::Error {
                let mut failure = BytesStart::new("failure");
                failure.push_attribute(("message", d.message.as_str()));
                failure.push_attribute(("type", check_name.as_str()));
                writer.write_event(Event::Start(failure))?;

                let detail = format!(
                    "Language: {}\nCheck: {}\n{}{}{}{}",
                    d.language,
                    d.check,
                    d.file
                        .as_ref()
                        .map(|f| format!("File: {f}\n"))
                        .unwrap_or_default(),
                    d.key
                        .as_ref()
                        .map(|k| format!("Key: {k}\n"))
                        .unwrap_or_default(),
                    d.expected
                        .as_ref()
                        .map(|e| format!("Expected: {e}\n"))
                        .unwrap_or_default(),
                    d.found
                        .as_ref()
                        .map(|f| format!("Found: {f}\n"))
                        .unwrap_or_default(),
                );
                writer.write_event(Event::Text(BytesText::new(&detail)))?;
                writer.write_event(Event::End(BytesEnd::new("failure")))?;
            } else if d.severity == Severity::Warning {
                // Warnings become system-out
                let system_out = BytesStart::new("system-out");
                writer.write_event(Event::Start(system_out))?;
                writer.write_event(Event::Text(BytesText::new(&format!(
                    "WARNING: {}",
                    d.message
                ))))?;
                writer.write_event(Event::End(BytesEnd::new("system-out")))?;
            }

            writer.write_event(Event::End(BytesEnd::new("testcase")))?;
        }

        writer.write_event(Event::End(BytesEnd::new("testsuite")))?;
    }

    writer.write_event(Event::End(BytesEnd::new("testsuites")))?;

    let xml = String::from_utf8(buffer)?;
    Ok(xml)
}
