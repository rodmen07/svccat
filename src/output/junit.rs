use crate::drift::{DriftItem, DriftReport, Severity};
use crate::ping::{PingResult, PingStatus};
use anyhow::Result;
use std::collections::HashSet;

pub fn render_check(report: &DriftReport, ping_results: &[PingResult]) -> Result<()> {
    print!("{}", build_check_document(report, ping_results));
    Ok(())
}

pub fn render_since(old_report: &DriftReport, new_report: &DriftReport, git_ref: &str) -> usize {
    let (xml, new_count) = build_since_document(old_report, new_report, git_ref);
    print!("{}", xml);
    new_count
}

pub fn build_check_document(report: &DriftReport, ping_results: &[PingResult]) -> String {
    let mut body = String::new();
    let mut failures = 0usize;
    let mut tests = 0usize;

    if report.drifts.is_empty() && ping_results.is_empty() {
        tests += 1;
        body.push_str(&testcase("svccat.drift", "clean", None));
    } else {
        for item in &report.drifts {
            tests += 1;
            failures += 1;
            let failure = failure_xml(
                match item.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                },
                &item.message,
                &drift_failure_body(item),
            );
            body.push_str(&testcase(
                "svccat.drift",
                &format!("{:?}:{}", item.kind, item.service),
                Some(&failure),
            ));
        }
    }

    for ping in ping_results {
        tests += 1;
        let name = format!("{} ({})", ping.service, ping.url);
        match &ping.ping {
            PingStatus::Reachable { code } => {
                let out = format!("<system-out>reachable status {code}</system-out>");
                body.push_str(&testcase("svccat.ping", &name, Some(&out)));
            }
            PingStatus::Unreachable { reason } => {
                failures += 1;
                let failure = failure_xml("error", reason, reason);
                body.push_str(&testcase("svccat.ping", &name, Some(&failure)));
            }
        }
    }

    wrap_suite("svccat.check", tests, failures, &body, None)
}

pub fn build_since_document(
    old_report: &DriftReport,
    new_report: &DriftReport,
    git_ref: &str,
) -> (String, usize) {
    let old_keys: HashSet<String> = old_report.drifts.iter().map(drift_key).collect();
    let added: Vec<&DriftItem> = new_report
        .drifts
        .iter()
        .filter(|d| !old_keys.contains(&drift_key(d)))
        .collect();

    let mut body = String::new();
    if added.is_empty() {
        body.push_str(&testcase("svccat.since", "no-new-drift", None));
    } else {
        for item in &added {
            let failure = failure_xml(
                match item.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                },
                &item.message,
                &drift_failure_body(item),
            );
            body.push_str(&testcase(
                "svccat.since",
                &format!("{:?}:{}", item.kind, item.service),
                Some(&failure),
            ));
        }
    }

    let xml = wrap_suite(
        "svccat.check.since",
        added.len().max(1),
        added.len(),
        &body,
        Some(("git_ref", git_ref)),
    );
    (xml, added.len())
}

fn wrap_suite(
    suite_name: &str,
    tests: usize,
    failures: usize,
    body: &str,
    property: Option<(&str, &str)>,
) -> String {
    let properties = property
        .map(|(name, value)| {
            format!(
                "<properties><property name=\"{}\" value=\"{}\"/></properties>",
                escape_xml(name),
                escape_xml(value)
            )
        })
        .unwrap_or_default();

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<testsuites name=\"svccat\" tests=\"{tests}\" failures=\"{failures}\" errors=\"0\" skipped=\"0\">\n  \
<testsuite name=\"{}\" tests=\"{tests}\" failures=\"{failures}\" errors=\"0\" skipped=\"0\" time=\"0\">\n    \
{properties}\n    \
{body}  </testsuite>\n\
</testsuites>\n",
        escape_xml(suite_name),
    )
}

fn testcase(classname: &str, name: &str, inner: Option<&str>) -> String {
    match inner {
        Some(inner_xml) => format!(
            "<testcase classname=\"{}\" name=\"{}\" time=\"0\">{}</testcase>\n",
            escape_xml(classname),
            escape_xml(name),
            inner_xml
        ),
        None => format!(
            "<testcase classname=\"{}\" name=\"{}\" time=\"0\"/>\n",
            escape_xml(classname),
            escape_xml(name)
        ),
    }
}

fn failure_xml(failure_type: &str, message: &str, body: &str) -> String {
    format!(
        "<failure type=\"{}\" message=\"{}\">{}</failure>",
        escape_xml(failure_type),
        escape_xml(message),
        escape_xml(body)
    )
}

fn drift_failure_body(item: &DriftItem) -> String {
    format!(
        "{} | service={} | detail={}",
        item.message,
        item.service,
        item.detail.as_deref().unwrap_or("")
    )
}

fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn drift_key(item: &DriftItem) -> String {
    format!(
        "{:?}|{}|{}",
        item.kind,
        item.service,
        item.detail.as_deref().unwrap_or("")
    )
}
