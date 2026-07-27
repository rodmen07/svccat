//! GitHub Actions workflow-command annotations for `svccat check`.
//!
//! This is the DEFAULT output format when `check` runs inside GitHub Actions
//! with no explicit `--format` (see `main.rs`), so everything printed here is
//! parsed by the runner as a workflow command. Two consequences shape this
//! module:
//!
//! - Every interpolated value is escaped per GitHub's workflow-command rules
//!   (`escape_data` / `escape_property`). Manifest content is untrusted input
//!   in the annotation context: a service name, path, or policy message
//!   containing a newline would otherwise terminate the `::error ...::`
//!   command early, and any following text beginning with `::` would be
//!   executed as a NEW workflow command by the runner (annotation spoofing,
//!   `::add-mask`, `::stop-commands`, ...).
//! - Annotations are file-level: drift has no line number to offer, so the
//!   annotation anchors to the manifest file, not a line in it.

use crate::drift::{DriftItem, DriftKind, DriftReport, Severity};
use crate::ping::{PingResult, PingStatus};
use std::collections::HashSet;

/// Escape a workflow-command message (the part after `::`) per GitHub's
/// rules: `%` → `%25`, `\r` → `%0D`, `\n` → `%0A`.
///
/// The `%` replacement MUST run first: running it after the others would
/// corrupt their output (`\n` → `%0A` → `%250A`), and skipping it would let
/// a literal `%0A` in the input decode as a newline on the runner.
fn escape_data(s: &str) -> String {
    s.replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}

/// Escape a workflow-command property value (`file=`, `title=`): data
/// escaping plus `:` → `%3A` and `,` → `%2C`, the property delimiters.
fn escape_property(s: &str) -> String {
    escape_data(s).replace(':', "%3A").replace(',', "%2C")
}

fn annotation_level(item: &DriftItem) -> &'static str {
    match item.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
    }
}

fn annotation_title(kind: &DriftKind) -> &'static str {
    match kind {
        DriftKind::DeclaredMissingFromRepo => "svccat [MISSING]",
        DriftKind::UndeclaredInRepo => "svccat [UNDECLARED]",
        DriftKind::MissingField => "svccat [FIELD]",
        DriftKind::MissingReferencedFile => "svccat [REF]",
        DriftKind::PolicyViolation => "svccat [POLICY]",
        DriftKind::DanglingDependency => "svccat [DEPENDS]",
        DriftKind::CircularDependency => "svccat [CYCLE]",
    }
}

/// The one shape every annotation this module emits has, with escaping baked
/// in so no call site can opt out (the `d3_force_graph.rs` precedent).
fn annotation_line(level: &str, file: &str, title: &str, message: &str) -> String {
    format!(
        "::{} file={},title={}::{}",
        level,
        escape_property(file),
        escape_property(title),
        escape_data(message)
    )
}

fn drift_annotation_line(item: &DriftItem, manifest: &str) -> String {
    annotation_line(
        annotation_level(item),
        manifest,
        annotation_title(&item.kind),
        &item.message,
    )
}

/// A ping outcome as an annotation line, or `None` when there is nothing to
/// report.
///
/// `Reachable` deliberately yields `None`: annotations are findings, and one
/// per healthy URL would bury the real ones. This mirrors the sarif and junit
/// renderers, where only the two non-healthy states are failures.
fn ping_annotation_line(result: &PingResult, manifest: &str) -> Option<String> {
    let (title, message) = match &result.ping {
        PingStatus::Reachable { .. } => return None,
        PingStatus::Unreachable { reason } => (
            "svccat [UNREACHABLE]",
            format!(
                "{} is unreachable at {}: {}",
                result.service, result.url, reason
            ),
        ),
        PingStatus::Invalid { reason } => (
            "svccat [INVALID-URL]",
            format!(
                "{} has an unusable url {}: {}",
                result.service, result.url, reason
            ),
        ),
    };
    Some(annotation_line("error", manifest, title, &message))
}

/// Every annotation `check` emits, in order: drift first, then non-healthy
/// ping outcomes. Pure so the full output is unit-testable.
fn check_annotation_lines(report: &DriftReport, ping_results: &[PingResult]) -> Vec<String> {
    let mut lines: Vec<String> = report
        .drifts
        .iter()
        .map(|item| drift_annotation_line(item, &report.manifest))
        .collect();
    lines.extend(
        ping_results
            .iter()
            .filter_map(|r| ping_annotation_line(r, &report.manifest)),
    );
    lines
}

/// Emit all drift items, and any failed `--ping` probes, as GitHub Actions
/// workflow annotations.
///
/// Errors become `::error` annotations; warnings become `::warning`
/// annotations. Each annotation is associated with the manifest file (drift
/// carries no line number), so it appears against that file in the
/// pull-request view.
pub fn render_check(report: &DriftReport, ping_results: &[PingResult]) {
    for line in check_annotation_lines(report, ping_results) {
        println!("{line}");
    }
}

/// The annotation lines for drift present in `new_report` but not
/// `old_report`. Pure so the since-filter is unit-testable.
fn since_annotation_lines(old_report: &DriftReport, new_report: &DriftReport) -> Vec<String> {
    let old_keys: HashSet<String> = old_report.drifts.iter().map(drift_key).collect();

    new_report
        .drifts
        .iter()
        .filter(|d| !old_keys.contains(&drift_key(d)))
        .map(|d| drift_annotation_line(d, &new_report.manifest))
        .collect()
}

/// Emit only new drift items (compared to `old_report`) as annotations.
///
/// Use this with `--since` so that only drift introduced by the current branch
/// is annotated — pre-existing drift is silently skipped.
pub fn render_since_annotations(old_report: &DriftReport, new_report: &DriftReport) -> usize {
    let lines = since_annotation_lines(old_report, new_report);
    for line in &lines {
        println!("{line}");
    }
    lines.len()
}

fn drift_key(item: &DriftItem) -> String {
    format!(
        "{:?}|{}|{}",
        item.kind,
        item.service,
        item.detail.as_deref().unwrap_or("")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // `DriftKind`, `Severity` and `PingStatus` are all `#[non_exhaustive]`,
    // so the wildcard-free matches below only compile in-crate — which is the
    // point: a new variant must extend these lists or the build fails, and an
    // integration test would be forced into a silently-weakening `_ =>` arm.

    /// Every `DriftKind`, listed once, under the compile-time guard.
    fn all_drift_kinds() -> Vec<DriftKind> {
        vec![
            DriftKind::DeclaredMissingFromRepo,
            DriftKind::UndeclaredInRepo,
            DriftKind::MissingField,
            DriftKind::MissingReferencedFile,
            DriftKind::PolicyViolation,
            DriftKind::DanglingDependency,
            DriftKind::CircularDependency,
        ]
    }

    /// Every `PingStatus`, listed once, under the same guard.
    fn all_ping_statuses() -> Vec<PingStatus> {
        vec![
            PingStatus::Reachable { code: 200 },
            PingStatus::Unreachable {
                reason: "connect timeout".into(),
            },
            PingStatus::Invalid {
                reason: "private IP address".into(),
            },
        ]
    }

    fn item(kind: DriftKind, severity: Severity, message: &str) -> DriftItem {
        DriftItem {
            kind,
            severity,
            service: "billing".into(),
            message: message.into(),
            detail: None,
        }
    }

    fn report(drifts: Vec<DriftItem>) -> DriftReport {
        DriftReport {
            manifest: "services.yaml".into(),
            declared: 1,
            discovered: 0,
            drifts,
        }
    }

    fn ping(service: &str, status: PingStatus) -> PingResult {
        PingResult {
            service: service.into(),
            url: "http://10.0.0.1/health".into(),
            ping: status,
        }
    }

    // -- escaping ----------------------------------------------------------

    #[test]
    fn escape_data_escapes_percent_first() {
        // A literal `%0A` in the input must come out as `%250A`, not survive
        // as a decodable newline; a real newline must become `%0A`.
        assert_eq!(escape_data("50% done\r\n"), "50%25 done%0D%0A");
        assert_eq!(escape_data("%0A"), "%250A");
    }

    #[test]
    fn escape_property_also_escapes_the_delimiters() {
        assert_eq!(escape_property("a:b,c"), "a%3Ab%2Cc");
        // And still applies the data escapes.
        assert_eq!(escape_property("x\ny"), "x%0Ay");
    }

    #[test]
    fn a_message_with_a_newline_stays_one_command() {
        // The injection shape: without escaping, everything after the
        // newline would be parsed by the runner as a fresh line — and a
        // leading `::` would start a NEW workflow command.
        let evil = item(
            DriftKind::DeclaredMissingFromRepo,
            Severity::Error,
            "gotcha\n::error title=fake::injected",
        );
        let line = drift_annotation_line(&evil, "services.yaml");
        assert!(
            !line.contains('\n') && !line.contains('\r'),
            "annotation must be a single line, got: {line}"
        );
        assert!(
            line.contains("gotcha%0A%3A%3Aerror") || line.contains("gotcha%0A::error"),
            "newline must be escaped, got: {line}"
        );
        // The command prefix appears exactly once — at the start.
        assert!(line.starts_with("::error "));
        assert_eq!(line.matches("\n::").count(), 0);
    }

    #[test]
    fn the_manifest_path_is_property_escaped() {
        // A `,` or `:` in the file property would end the property early and
        // corrupt the annotation's target.
        let it = item(DriftKind::MissingField, Severity::Warning, "m");
        let line = drift_annotation_line(&it, "dir,with:odd/services.yaml");
        assert!(
            line.contains("file=dir%2Cwith%3Aodd/services.yaml"),
            "got: {line}"
        );
    }

    // -- drift annotations -------------------------------------------------

    #[test]
    fn severity_maps_to_error_and_warning_levels() {
        let err = item(DriftKind::DeclaredMissingFromRepo, Severity::Error, "m");
        let warn = item(DriftKind::MissingField, Severity::Warning, "m");
        assert!(drift_annotation_line(&err, "services.yaml").starts_with("::error "));
        assert!(drift_annotation_line(&warn, "services.yaml").starts_with("::warning "));
    }

    #[test]
    fn every_drift_kind_has_a_distinct_title() {
        let kinds = all_drift_kinds();
        let titles: HashSet<&'static str> = kinds.iter().map(annotation_title).collect();
        assert_eq!(
            titles.len(),
            kinds.len(),
            "two DriftKinds share an annotation title"
        );
        for t in titles {
            assert!(t.starts_with("svccat ["), "title off-convention: {t}");
        }
    }

    // -- ping annotations --------------------------------------------------

    #[test]
    fn reachable_is_not_a_finding_and_the_other_two_are() {
        // Walks every PingStatus so a new variant fails compilation here.
        for status in all_ping_statuses() {
            let expect_line = !matches!(status, PingStatus::Reachable { .. });
            let line = ping_annotation_line(&ping("billing", status), "services.yaml");
            assert_eq!(line.is_some(), expect_line);
        }
    }

    #[test]
    fn unreachable_and_invalid_render_as_error_annotations() {
        let un = ping_annotation_line(
            &ping(
                "billing",
                PingStatus::Unreachable {
                    reason: "connect timeout".into(),
                },
            ),
            "services.yaml",
        )
        .expect("unreachable is a finding");
        assert!(un.starts_with("::error "), "got: {un}");
        assert!(un.contains("svccat [UNREACHABLE]"), "got: {un}");
        assert!(un.contains("billing is unreachable at"), "got: {un}");

        let inv = ping_annotation_line(
            &ping(
                "billing",
                PingStatus::Invalid {
                    reason: "private IP address".into(),
                },
            ),
            "services.yaml",
        )
        .expect("invalid is a finding");
        assert!(inv.contains("svccat [INVALID-URL]"), "got: {inv}");
        assert!(inv.contains("billing has an unusable url"), "got: {inv}");
    }

    #[test]
    fn check_lines_carry_drift_then_failed_pings() {
        let rep = report(vec![item(
            DriftKind::DeclaredMissingFromRepo,
            Severity::Error,
            "'billing' is declared in the manifest but not found in the repo",
        )]);
        let pings = vec![
            ping("billing", PingStatus::Reachable { code: 200 }),
            ping(
                "billing",
                PingStatus::Invalid {
                    reason: "private IP address".into(),
                },
            ),
        ];
        let lines = check_annotation_lines(&rep, &pings);
        assert_eq!(lines.len(), 2, "one drift + one failed ping: {lines:?}");
        assert!(lines[0].contains("svccat [MISSING]"));
        assert!(lines[1].contains("svccat [INVALID-URL]"));
    }

    #[test]
    fn check_lines_without_ping_results_are_drift_only() {
        let rep = report(vec![item(
            DriftKind::DeclaredMissingFromRepo,
            Severity::Error,
            "m",
        )]);
        let lines = check_annotation_lines(&rep, &[]);
        assert_eq!(lines.len(), 1);
        assert!(!lines[0].contains("UNREACHABLE"));
    }

    // -- since-filtering ---------------------------------------------------

    #[test]
    fn since_lines_skip_drift_already_in_the_old_report() {
        let old = report(vec![item(
            DriftKind::MissingField,
            Severity::Warning,
            "pre-existing",
        )]);
        let mut fresh = item(DriftKind::DanglingDependency, Severity::Error, "new drift");
        fresh.detail = Some("cache".into());
        let new = report(vec![
            item(DriftKind::MissingField, Severity::Warning, "pre-existing"),
            fresh,
        ]);
        let lines = since_annotation_lines(&old, &new);
        assert_eq!(lines.len(), 1, "only the new item: {lines:?}");
        assert!(lines[0].contains("svccat [DEPENDS]"));
        assert!(lines[0].contains("new drift"));
    }
}
