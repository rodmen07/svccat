//! Manifest line numbers for service entries, recovered by a text scan.
//!
//! `serde_yaml_ng` 0.10 exposes position information only on `Error`
//! (`error.rs` defines `Location`, populated from a *failed* parse; `de.rs`
//! has no span surface for values that parse successfully), so a loaded
//! [`Manifest`] knows nothing about where its entries sit in the file. This
//! module recovers that positionally: the Nth `name:` key inside the
//! top-level `services:` block belongs to the Nth entry of
//! `Manifest::services`, because serde fills the `Vec` in document order.
//!
//! # Fail-closed contract
//!
//! The scan is textual, not a YAML parse, so exotic input can fool it — a
//! block scalar (`docs: |`) whose body contains a line that *looks* like a
//! `name:` key adds a phantom match, and a quoted key (`"name":`) is missed.
//! Every consumer therefore goes through [`attach`], which compares the match
//! count against the service count and attaches **nothing** on disagreement.
//! Degrading means file-level findings — exactly the pre-feature behavior —
//! never a finding anchored to the wrong line.

use crate::drift::DriftReport;
use crate::manifest::Manifest;
use std::collections::HashMap;

/// The 1-based line numbers of each service entry's `name:` key, in document
/// order.
///
/// Only lines inside the top-level `services:` block are considered, so
/// `workspace.name`, a top-level `name:` key, and `name:` keys under other
/// sections never match. List items at any indentation are handled, including
/// the column-0 style (`services:` followed by `- name: x` at column 0) and
/// the dash-on-its-own-line style.
pub fn service_name_lines(yaml: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let mut in_services = false;

    for (idx, raw) in yaml.lines().enumerate() {
        // A top-level key starts a new section and ends the previous one.
        // Column-0 list items (`- ...`), comments, and blank lines do not:
        // a `services:` sequence may legally sit at column 0.
        match raw.bytes().next() {
            Some(b' ') | Some(b'\t') | Some(b'#') | Some(b'-') | Some(b'\r') | None => {}
            Some(_) => {
                in_services = is_services_key(raw);
                continue;
            }
        }

        if in_services && is_name_key_line(raw) {
            out.push(idx + 1);
        }
    }

    out
}

/// Attach manifest line numbers to every drift item whose service the
/// manifest declares.
///
/// `manifest` must be the manifest parsed from `manifest_text` — the *full*
/// one, not a filtered view — because the positional zip below pairs the Nth
/// scanned line with the Nth parsed entry. When the scan and the parser
/// disagree about how many services the file holds, nothing is attached (see
/// the module docs for why wrong lines are worse than no lines).
///
/// For a duplicate service name (tolerated by `Manifest::load`, flagged only
/// by `svccat lint`) the first entry's line wins, matching the first-wins
/// reporting convention `watch::detect_changes` uses for the same input.
/// Items naming an undeclared service (`UndeclaredInRepo`) stay line-less.
pub fn attach(report: &mut DriftReport, manifest: &Manifest, manifest_text: &str) {
    let lines = service_name_lines(manifest_text);
    if lines.len() != manifest.services.len() {
        return; // fail closed: no lines rather than possibly-wrong lines
    }

    let mut by_name: HashMap<&str, usize> = HashMap::with_capacity(lines.len());
    for (svc, line) in manifest.services.iter().zip(&lines) {
        by_name.entry(svc.name.as_str()).or_insert(*line);
    }

    for item in &mut report.drifts {
        if item.line.is_none() {
            item.line = by_name.get(item.service.as_str()).copied();
        }
    }
}

/// Is this column-0 line the `services:` mapping key?
fn is_services_key(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("services") else {
        return false;
    };
    let rest = rest.trim_start_matches([' ', '\t']);
    let Some(after_colon) = rest.strip_prefix(':') else {
        return false;
    };
    // `services:` introducing a block (possibly with a trailing comment).
    // `services: []` or `services: [...]` declares an inline value whose
    // entries the line scan cannot see; treating it as "not the block" keeps
    // the block-less form at zero matches, which `attach` then reconciles
    // against the parsed count (zero services: fine; some: fail closed).
    let value = after_colon.trim();
    value.is_empty() || value.starts_with('#')
}

/// Does this line carry a service entry's `name:` key — optionally introduced
/// by a list dash — as YAML would read it?
fn is_name_key_line(line: &str) -> bool {
    let mut rest = line.trim_start();
    if let Some(after_dash) = rest.strip_prefix('-') {
        // The dash only introduces a list item when followed by whitespace;
        // `-name:` is a plain scalar, and a bare `-` carries no key.
        match after_dash.bytes().next() {
            Some(b' ') | Some(b'\t') => rest = after_dash.trim_start(),
            _ => return false,
        }
    }
    let Some(after_key) = rest.strip_prefix("name") else {
        return false;
    };
    let after_key = after_key.trim_start_matches([' ', '\t']);
    let Some(after_colon) = after_key.strip_prefix(':') else {
        return false;
    };
    // In YAML, `name:foo` is a plain scalar, not a mapping: the colon only
    // separates a key when followed by whitespace or the end of the line.
    matches!(
        after_colon.bytes().next(),
        None | Some(b' ') | Some(b'\t') | Some(b'\r')
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drift::{DriftItem, DriftKind, Severity};

    fn manifest_from(yaml: &str) -> Manifest {
        serde_yaml::from_str(yaml).expect("test manifest parses")
    }

    fn report_for(services: &[&str]) -> DriftReport {
        let mut report = DriftReport::default();
        for svc in services {
            report.drifts.push(DriftItem {
                kind: DriftKind::MissingField,
                severity: Severity::Warning,
                service: svc.to_string(),
                message: format!("'{svc}' is missing recommended field: role"),
                detail: Some("role".to_string()),
                line: None,
            });
        }
        report
    }

    fn lines_of(report: &DriftReport) -> Vec<Option<usize>> {
        report.drifts.iter().map(|d| d.line).collect()
    }

    const TWO_SERVICES: &str = "version: \"1\"\n\nservices:\n  - name: billing\n    role: api\n  - name: auth\n    role: api\n";

    // ── The scan ────────────────────────────────────────────────────────────

    #[test]
    fn finds_each_service_names_line_in_document_order() {
        assert_eq!(service_name_lines(TWO_SERVICES), vec![4, 6]);
    }

    #[test]
    fn handles_column_zero_list_items_and_dash_on_its_own_line() {
        let yaml = "services:\n- name: billing\n  role: api\n-\n  name: auth\n";
        assert_eq!(service_name_lines(yaml), vec![2, 5]);
    }

    #[test]
    fn ignores_name_keys_outside_the_services_block() {
        let yaml = "workspace:\n  name: platform\n\nname: toplevel\n\nservices:\n  - name: billing\n\npolicy:\n  rules:\n    - id: has-name\n      name: not-a-service\n";
        assert_eq!(service_name_lines(yaml), vec![7]);
    }

    #[test]
    fn ignores_comments_similar_keys_and_scalar_lookalikes() {
        let yaml = "services:\n  # name: commented-out\n  - name: billing\n    names: [alias]\n    name_prefix: x\n    docs: name:not-a-key\n";
        assert_eq!(service_name_lines(yaml), vec![3]);
    }

    #[test]
    fn crlf_input_scans_identically_to_lf() {
        let crlf = TWO_SERVICES.replace('\n', "\r\n");
        assert_eq!(service_name_lines(&crlf), service_name_lines(TWO_SERVICES));
    }

    #[test]
    fn an_inline_empty_services_list_yields_no_matches() {
        assert_eq!(service_name_lines("services: []\n"), Vec::<usize>::new());
    }

    // ── attach: the fail-closed contract ────────────────────────────────────

    #[test]
    fn attach_gives_each_item_its_services_name_line() {
        let mut report = report_for(&["auth", "billing"]);
        attach(&mut report, &manifest_from(TWO_SERVICES), TWO_SERVICES);
        assert_eq!(lines_of(&report), vec![Some(6), Some(4)]);
    }

    #[test]
    fn attach_leaves_undeclared_services_line_less() {
        let mut report = report_for(&["billing", "not-in-manifest"]);
        attach(&mut report, &manifest_from(TWO_SERVICES), TWO_SERVICES);
        assert_eq!(lines_of(&report), vec![Some(4), None]);
    }

    /// A block scalar whose body *looks* like a `name:` key is the documented
    /// way to fool the text scan. The phantom match must make `attach` refuse
    /// to attach anything, not mis-align every line after it.
    #[test]
    fn attach_fails_closed_when_the_scan_and_the_parser_disagree() {
        let yaml =
            "services:\n  - name: billing\n    docs: |\n      name: phantom\n  - name: auth\n";
        let manifest = manifest_from(yaml);
        assert_eq!(manifest.services.len(), 2, "fixture must parse as 2");
        assert_eq!(
            service_name_lines(yaml).len(),
            3,
            "fixture must fool the scan, or this test guards nothing"
        );

        let mut report = report_for(&["billing", "auth"]);
        attach(&mut report, &manifest, yaml);
        assert_eq!(
            lines_of(&report),
            vec![None, None],
            "a count mismatch must attach nothing"
        );
    }

    /// `Manifest::load` tolerates duplicate names (only `svccat lint` flags
    /// them), so the map must not panic or pick nondeterministically: the
    /// first entry wins, as it does in `watch::detect_changes`.
    #[test]
    fn attach_uses_the_first_entry_for_a_duplicated_name() {
        let yaml = "services:\n  - name: billing\n  - name: billing\n";
        let mut report = report_for(&["billing"]);
        attach(&mut report, &manifest_from(yaml), yaml);
        assert_eq!(lines_of(&report), vec![Some(2)]);
    }

    #[test]
    fn attach_does_not_overwrite_a_line_already_present() {
        let mut report = report_for(&["billing"]);
        report.drifts[0].line = Some(99);
        attach(&mut report, &manifest_from(TWO_SERVICES), TWO_SERVICES);
        assert_eq!(lines_of(&report), vec![Some(99)]);
    }
}
