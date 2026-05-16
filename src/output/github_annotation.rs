use crate::drift::{DriftItem, DriftKind, DriftReport, Severity};
use std::collections::HashSet;

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

fn emit_annotation(item: &DriftItem, manifest: &str) {
    println!(
        "::{} file={},title={}::{}",
        annotation_level(item),
        manifest,
        annotation_title(&item.kind),
        item.message
    );
}

/// Emit all drift items as GitHub Actions workflow annotations.
///
/// Errors become `::error` annotations; warnings become `::warning` annotations.
/// Each annotation is associated with the manifest file so it appears in the
/// pull-request file diff view.
pub fn render_check(report: &DriftReport) {
    for item in &report.drifts {
        emit_annotation(item, &report.manifest);
    }
}

/// Emit only new drift items (compared to `old_report`) as annotations.
///
/// Use this with `--since` so that only drift introduced by the current branch
/// is annotated — pre-existing drift is silently skipped.
pub fn render_since_annotations(old_report: &DriftReport, new_report: &DriftReport) -> usize {
    let old_keys: HashSet<String> = old_report.drifts.iter().map(drift_key).collect();

    let added: Vec<&DriftItem> = new_report
        .drifts
        .iter()
        .filter(|d| !old_keys.contains(&drift_key(d)))
        .collect();

    for item in &added {
        emit_annotation(item, &new_report.manifest);
    }

    added.len()
}

fn drift_key(item: &DriftItem) -> String {
    format!(
        "{:?}|{}|{}",
        item.kind,
        item.service,
        item.detail.as_deref().unwrap_or("")
    )
}
