//! Drift-list ordering and cross-entry-point agreement for `src/diff.rs`.
//!
//! `DiffReport::new_drift` / `resolved_drift` are public fields rendered by the
//! markdown renderer, the terminal renderer and (through the public struct) any
//! JSON consumer, and they are built by **two** entry points:
//!
//! * `diff_snapshots`, behind `svccat diff <before.json> <after.json>`,
//! * `diff_from_json` / `snapshot::diff`, behind `svccat snapshot diff`.
//!
//! Until this file existed the two disagreed on both counts. The snapshot path
//! built its lists with `HashSet::difference`, whose iteration order is
//! unspecified and re-randomised per set by the default hasher, so two runs over
//! byte-identical inputs printed the same drift in different orders. It also
//! emitted the raw `service:message` dedup key while the file path emitted the
//! severity-prefixed line. One public field, two formats, one of them
//! nondeterministic.
//!
//! The tests below pin the three properties a caller can actually rely on:
//! source order, one line per `service:message`, and (the guard that keeps the
//! two builders from drifting apart again) that both entry points produce
//! byte-identical lists for the same input. That last one reads BOTH sources, so
//! re-splitting the implementation cannot pass it.

use std::fs;
use tempfile::TempDir;

/// A snapshot payload with a fixed service list and the given drift entries.
///
/// Drift order in the JSON is the order the assertions expect back out, so the
/// entries are written in an order no sort would reproduce (`zulu` first,
/// `alpha` last), so a builder that sorted instead of preserving source order
/// would fail here rather than pass by accident.
fn snapshot_json(drift: &[(&str, &str, &str)]) -> serde_json::Value {
    let drift: Vec<serde_json::Value> = drift
        .iter()
        .map(|(service, message, severity)| {
            serde_json::json!({
                "service": service,
                "message": message,
                "severity": severity,
            })
        })
        .collect();

    serde_json::json!({
        "services": [
            {"name": "zulu", "language": "Rust", "platform": "Cloud Run", "role": "api", "depends_on": []},
            {"name": "mike", "language": "Go", "platform": "Fly.io", "role": "worker", "depends_on": []},
            {"name": "alpha", "language": "Python", "platform": "Cloud Run", "role": "api", "depends_on": []}
        ],
        "drift": drift,
    })
}

const BEFORE: &[(&str, &str, &str)] = &[
    ("zulu", "declared but not discovered", "error"),
    ("mike", "missing url", "warning"),
];

const AFTER: &[(&str, &str, &str)] = &[
    ("zulu", "declared but not discovered", "error"),
    ("alpha", "discovered but not declared", "error"),
    ("mike", "missing docs", "warning"),
    ("zulu", "missing ci", "warning"),
    ("alpha", "missing url", "warning"),
];

fn from_json(before: &serde_json::Value, after: &serde_json::Value) -> svccat::diff::DiffReport {
    svccat::diff::diff_from_json(before, after, "before", "after").unwrap()
}

fn from_files(
    dir: &TempDir,
    before: &serde_json::Value,
    after: &serde_json::Value,
) -> svccat::diff::DiffReport {
    let before_path = dir.path().join("before.json");
    let after_path = dir.path().join("after.json");
    fs::write(&before_path, serde_json::to_string(before).unwrap()).unwrap();
    fs::write(&after_path, serde_json::to_string(after).unwrap()).unwrap();
    svccat::diff::diff_snapshots(&before_path, &after_path).unwrap()
}

#[test]
fn snapshot_diff_reports_new_drift_in_snapshot_order() {
    let report = from_json(&snapshot_json(BEFORE), &snapshot_json(AFTER));

    assert_eq!(
        report.new_drift,
        vec![
            "[ERROR] alpha — discovered but not declared".to_string(),
            "[WARNING] mike — missing docs".to_string(),
            "[WARNING] zulu — missing ci".to_string(),
            "[WARNING] alpha — missing url".to_string(),
        ],
        "new drift must follow the order of the `after` snapshot's drift list"
    );
}

#[test]
fn snapshot_diff_reports_resolved_drift_in_snapshot_order() {
    let report = from_json(&snapshot_json(BEFORE), &snapshot_json(AFTER));

    assert_eq!(
        report.resolved_drift,
        vec!["[WARNING] mike — missing url".to_string()],
        "resolved drift must follow the order of the `before` snapshot's drift list"
    );
}

/// The property the `HashSet::difference` implementation could not hold.
///
/// `RandomState` seeds each `HashSet` independently, so this fails on the old
/// code from repeated calls inside one process, with no separate run needed.
#[test]
fn drift_lists_are_identical_across_repeated_calls() {
    let before = snapshot_json(BEFORE);
    let after = snapshot_json(AFTER);

    let first = from_json(&before, &after);
    for attempt in 1..=32 {
        let again = from_json(&before, &after);
        assert_eq!(
            again.new_drift, first.new_drift,
            "new drift changed order on attempt {attempt}; the list is not deterministic"
        );
        assert_eq!(
            again.resolved_drift, first.resolved_drift,
            "resolved drift changed order on attempt {attempt}; the list is not deterministic"
        );
    }
}

/// The drift guard: it reads BOTH builders, so they cannot diverge again.
#[test]
fn both_diff_entry_points_produce_identical_drift_lists() {
    let dir = TempDir::new().unwrap();
    let before = snapshot_json(BEFORE);
    let after = snapshot_json(AFTER);

    let via_files = from_files(&dir, &before, &after);
    let via_json = from_json(&before, &after);

    assert_eq!(
        via_json.new_drift, via_files.new_drift,
        "`svccat snapshot diff` and `svccat diff` must report the same new drift, \
         in the same order and the same format"
    );
    assert_eq!(
        via_json.resolved_drift, via_files.resolved_drift,
        "`svccat snapshot diff` and `svccat diff` must report the same resolved drift, \
         in the same order and the same format"
    );
    assert!(
        !via_files.new_drift.is_empty() && !via_files.resolved_drift.is_empty(),
        "the fixture must produce both kinds of drift change, or this guard compares \
         two empty vectors and cannot fail"
    );
}

/// A snapshot may list the same `service` + `message` twice (nothing dedupes the
/// drift vector upstream); the diff reports it once, exactly as the watch-mode
/// change summary does for a service declared twice.
#[test]
fn a_repeated_service_message_is_reported_once() {
    let after_with_dupe: &[(&str, &str, &str)] = &[
        ("alpha", "missing url", "warning"),
        ("mike", "missing docs", "warning"),
        ("alpha", "missing url", "error"),
    ];

    let report = from_json(&snapshot_json(&[]), &snapshot_json(after_with_dupe));

    assert_eq!(
        report.new_drift,
        vec![
            "[WARNING] alpha — missing url".to_string(),
            "[WARNING] mike — missing docs".to_string(),
        ],
        "a `service:message` repeated in the source must be reported once, keeping \
         its first occurrence"
    );
}

/// Identical snapshots must produce no drift change at all, from either builder.
#[test]
fn identical_snapshots_report_no_drift_change() {
    let dir = TempDir::new().unwrap();
    let snapshot = snapshot_json(AFTER);

    let via_json = from_json(&snapshot, &snapshot);
    let via_files = from_files(&dir, &snapshot, &snapshot);

    assert!(via_json.new_drift.is_empty() && via_json.resolved_drift.is_empty());
    assert!(via_files.new_drift.is_empty() && via_files.resolved_drift.is_empty());
}
