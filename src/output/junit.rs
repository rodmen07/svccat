use crate::drift::{DriftItem, DriftReport, Severity};
use crate::output::terminal::drift_identity_key;
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
            PingStatus::Invalid { reason } => {
                failures += 1;
                let failure = failure_xml("error", &format!("invalid URL: {}", reason), reason);
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
    let old_keys: HashSet<String> = old_report.drifts.iter().map(drift_identity_key).collect();
    let added: Vec<&DriftItem> = new_report
        .drifts
        .iter()
        .filter(|d| !old_keys.contains(&drift_identity_key(d)))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drift::DriftItem;
    use crate::ping::PingResult;
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;
    use quick_xml::XmlVersion;

    // ── Fixtures, mirroring output::sarif's tests module ──────────────────────

    fn report_with(drifts: Vec<DriftItem>) -> DriftReport {
        DriftReport {
            manifest: "services.yaml".to_string(),
            declared: drifts.len(),
            discovered: 0,
            drifts,
        }
    }

    fn drift(service: &str, message: &str, severity: Severity) -> DriftItem {
        DriftItem {
            kind: crate::drift::DriftKind::DeclaredMissingFromRepo,
            severity,
            service: service.to_string(),
            message: message.to_string(),
            detail: None,
            line: None,
        }
    }

    fn ping(service: &str, url: &str, status: PingStatus) -> PingResult {
        PingResult {
            service: service.to_string(),
            url: url.to_string(),
            ping: status,
        }
    }

    /// A string carrying every character `escape_xml` maps, plus a couple
    /// that must NOT be mangled (ASCII text, a unicode character) so a
    /// mutation that over-escapes would show up too.
    const NASTY: &str = "5 < 6 & 7 > 3, say \"hi\" or 'bye' — café";

    // ── Real-parser well-formedness, not string spot checks (L-003) ───────────

    /// Parses the whole document with a real XML reader, failing loudly (with
    /// the byte offset and the document itself) on the first well-formedness
    /// error, rather than eyeballing a handful of `contains()` calls. This is
    /// the same "verified against a real consumer" shape the SARIF URI fix
    /// used `url::Url::parse` for.
    fn assert_well_formed(xml: &str) {
        let mut reader = Reader::from_str(xml);
        loop {
            match reader.read_event() {
                Err(e) => panic!(
                    "malformed XML at byte {}: {e}\n--- document ---\n{xml}",
                    reader.buffer_position()
                ),
                Ok(Event::Eof) => break,
                Ok(_) => {}
            }
        }
    }

    /// Every value of the attribute `attr_name` on every `<tag>` start/empty
    /// element, in document order, with entities decoded back to the
    /// original text by the real parser (not by re-implementing
    /// `escape_xml` in reverse).
    ///
    /// `Explicit1_0` because these documents open with an explicit
    /// `<?xml version="1.0" …?>`; it and `Implicit1_0` route to the same
    /// normalizer, so the choice is about describing the input honestly
    /// rather than about behaviour.
    fn attr_values(xml: &str, tag: &[u8], attr_name: &[u8]) -> Vec<String> {
        let mut reader = Reader::from_str(xml);
        let mut out = Vec::new();
        loop {
            match reader
                .read_event()
                .expect("document already proven well-formed")
            {
                Event::Eof => break,
                Event::Start(e) | Event::Empty(e) if e.name().as_ref() == tag => {
                    for a in e.attributes() {
                        let a = a.expect("well-formed attribute");
                        if a.key.as_ref() == attr_name {
                            out.push(
                                a.normalized_value(XmlVersion::Explicit1_0)
                                    .expect("valid entity reference")
                                    .into_owned(),
                            );
                        }
                    }
                }
                _ => {}
            }
        }
        out
    }

    /// Text content of every `<failure>` element, ONE STRING PER ELEMENT, in
    /// document order, entity-decoded by the real parser.
    ///
    /// `read_text` is load-bearing here rather than a tidier spelling of a
    /// `read_event` loop: quick-xml reports every `&…;` inside text as its
    /// own `Event::GeneralRef` event, NOT as part of the surrounding
    /// `Event::Text`. So accumulating `Text` events yields one fragment per
    /// run of unescaped characters — a body carrying the five XML
    /// metacharacters arrives as 15 fragments, not one — and any assertion
    /// on `len()` is really counting metacharacters. `read_text` instead
    /// returns the element's whole raw span (entities still escaped, which
    /// is why `unescape` follows), so one `<failure>` yields one body no
    /// matter what it contains.
    fn failure_texts(xml: &str) -> Vec<String> {
        let mut reader = Reader::from_str(xml);
        let mut out = Vec::new();
        loop {
            match reader
                .read_event()
                .expect("document already proven well-formed")
            {
                Event::Eof => break,
                Event::Start(e) if e.name().as_ref() == b"failure" => {
                    let end = e.to_end().into_owned();
                    let raw = reader
                        .read_text(end.name())
                        .expect("<failure> is closed and holds no nested element");
                    let decoded = raw.decode().expect("valid utf-8 text content");
                    out.push(
                        quick_xml::escape::unescape(&decoded)
                            .expect("valid entity reference")
                            .into_owned(),
                    );
                }
                _ => {}
            }
        }
        out
    }

    // ── escape_xml itself ──────────────────────────────────────────────────────

    #[test]
    fn escape_xml_maps_every_xml_metacharacter() {
        assert_eq!(escape_xml("&"), "&amp;");
        assert_eq!(escape_xml("<"), "&lt;");
        assert_eq!(escape_xml(">"), "&gt;");
        assert_eq!(escape_xml("\""), "&quot;");
        assert_eq!(escape_xml("'"), "&apos;");
        assert_eq!(escape_xml("plain text"), "plain text");
    }

    #[test]
    fn escape_xml_does_not_double_escape_a_literal_ampersand_entity() {
        // `&` must be replaced first: escaping it LAST would turn the `&`
        // that a `<`-escape or `"`-escape just produced back into `&amp;lt;`
        // etc. A literal `&amp;` typed by a user must come out as one extra
        // layer of escaping (`&amp;amp;`), not two.
        assert_eq!(escape_xml("&amp;"), "&amp;amp;");
        assert_eq!(escape_xml("<>&\"'"), "&lt;&gt;&amp;&quot;&apos;");
    }

    // ── build_check_document: well-formedness + round-trip as a property ──────

    #[test]
    fn clean_check_document_is_well_formed_with_one_passing_testcase() {
        let xml = build_check_document(&report_with(vec![]), &[]);
        assert_well_formed(&xml);
        assert!(xml.contains(r#"tests="1" failures="0""#));
        assert!(xml.contains(r#"name="clean""#));
    }

    #[test]
    fn a_drift_service_name_and_message_survive_round_trip_through_a_real_xml_parser() {
        let report = report_with(vec![drift(NASTY, NASTY, Severity::Error)]);
        let xml = build_check_document(&report, &[]);
        assert_well_formed(&xml);

        let names = attr_values(&xml, b"testcase", b"name");
        assert_eq!(names.len(), 1);
        assert!(
            names[0].contains(NASTY),
            "testcase name lost the drift service/message content: {:?}",
            names[0]
        );

        let messages = attr_values(&xml, b"failure", b"message");
        assert_eq!(messages, vec![NASTY.to_string()]);

        let bodies = failure_texts(&xml);
        assert_eq!(bodies.len(), 1);
        assert!(
            bodies[0].contains(NASTY),
            "failure body lost the drift content: {:?}",
            bodies[0]
        );
    }

    #[test]
    fn each_failure_yields_exactly_one_body_however_many_entities_it_carries() {
        // Regression guard for the way quick-xml actually reports text: every
        // `&…;` is its own `Event::GeneralRef`, so a `read_event` loop that
        // collects `Event::Text` returns one fragment per run of unescaped
        // characters instead of one body per `<failure>`. Two bodies each
        // carrying the five XML metacharacters come back as ~30 fragments
        // under that reading and as exactly 2 under `read_text`, so this
        // fails loudly if `failure_texts` is ever rewritten as an event loop.
        // Two drifts, not one, because a single-failure document cannot tell
        // "one body" apart from "the first of several fragments".
        let report = report_with(vec![
            drift(NASTY, NASTY, Severity::Error),
            drift(NASTY, NASTY, Severity::Warning),
        ]);
        let xml = build_check_document(&report, &[]);
        assert_well_formed(&xml);

        let bodies = failure_texts(&xml);
        assert_eq!(
            bodies.len(),
            2,
            "expected one body per <failure>, got fragments: {bodies:?}"
        );
        for body in &bodies {
            assert!(body.contains(NASTY), "failure body lost content: {body:?}");
        }
    }

    #[test]
    fn an_unreachable_ping_reason_and_url_survive_round_trip() {
        let report = report_with(vec![]);
        let ping_results = vec![ping(
            NASTY,
            "https://example.com/?a=1&b=2",
            PingStatus::Unreachable {
                reason: NASTY.to_string(),
            },
        )];
        let xml = build_check_document(&report, &ping_results);
        assert_well_formed(&xml);

        let names = attr_values(&xml, b"testcase", b"name");
        assert_eq!(names.len(), 1);
        assert!(
            names[0].contains(NASTY),
            "ping service lost: {:?}",
            names[0]
        );
        assert!(
            names[0].contains("https://example.com/?a=1&b=2"),
            "ping url lost: {:?}",
            names[0]
        );

        let messages = attr_values(&xml, b"failure", b"message");
        assert_eq!(messages, vec![NASTY.to_string()]);
    }

    #[test]
    fn an_invalid_ping_reason_survives_round_trip_and_is_counted_as_a_failure() {
        let report = report_with(vec![]);
        let ping_results = vec![ping(
            "svc",
            "not a url & <broken>",
            PingStatus::Invalid {
                reason: NASTY.to_string(),
            },
        )];
        let xml = build_check_document(&report, &ping_results);
        assert_well_formed(&xml);
        assert!(xml.contains(r#"tests="1" failures="1""#));

        let messages = attr_values(&xml, b"failure", b"message");
        assert_eq!(messages.len(), 1);
        assert!(
            messages[0].contains(NASTY),
            "invalid reason lost: {:?}",
            messages[0]
        );
    }

    #[test]
    fn a_reachable_ping_is_well_formed_and_not_counted_as_a_failure() {
        let report = report_with(vec![]);
        let ping_results = vec![ping(
            "svc",
            "https://svc.example.com/health",
            PingStatus::Reachable { code: 200 },
        )];
        let xml = build_check_document(&report, &ping_results);
        assert_well_formed(&xml);
        assert!(xml.contains(r#"tests="1" failures="0""#));
        assert!(!xml.contains("<failure"));
    }

    #[test]
    fn mixed_report_counts_tests_and_failures_correctly() {
        // 2 drift items (1 error, 1 warning) + 1 unreachable ping (failure)
        // + 1 reachable ping (not a failure) = 4 tests, 3 failures. Parsed
        // from the real document rather than computed separately, so the
        // assertion can't drift from what the renderer actually emits.
        let report = report_with(vec![
            drift("api", "missing", Severity::Error),
            drift("worker", "undeclared", Severity::Warning),
        ]);
        let ping_results = vec![
            ping(
                "cache",
                "https://cache.example.com",
                PingStatus::Unreachable {
                    reason: "timeout".to_string(),
                },
            ),
            ping(
                "queue",
                "https://queue.example.com",
                PingStatus::Reachable { code: 200 },
            ),
        ];
        let xml = build_check_document(&report, &ping_results);
        assert_well_formed(&xml);
        assert!(xml.contains(r#"tests="4" failures="3""#));
    }

    // ── build_since_document: same property, plus the git_ref property ────────

    #[test]
    fn since_document_with_no_new_drift_is_well_formed() {
        let old = report_with(vec![drift("api", "missing", Severity::Error)]);
        let new = old.clone();
        let (xml, new_count) = build_since_document(&old, &new, "main");
        assert_well_formed(&xml);
        assert_eq!(new_count, 0);
        assert!(xml.contains(r#"name="no-new-drift""#));
    }

    #[test]
    fn since_document_new_drift_and_git_ref_survive_round_trip() {
        let old = report_with(vec![]);
        let new = report_with(vec![drift(NASTY, NASTY, Severity::Error)]);
        let (xml, new_count) = build_since_document(&old, &new, NASTY);
        assert_well_formed(&xml);
        assert_eq!(new_count, 1);
        assert!(xml.contains(r#"tests="1" failures="1""#));

        let property_values = attr_values(&xml, b"property", b"value");
        assert_eq!(property_values, vec![NASTY.to_string()]);

        let messages = attr_values(&xml, b"failure", b"message");
        assert_eq!(messages, vec![NASTY.to_string()]);
    }
}
