use crate::drift::{DriftItem, DriftReport, Severity};
use crate::ping::{PingResult, PingStatus};
use std::fmt::Write;

fn drift_kind_label(kind: &crate::drift::DriftKind) -> &'static str {
    match kind {
        crate::drift::DriftKind::DeclaredMissingFromRepo => "MISSING",
        crate::drift::DriftKind::UndeclaredInRepo => "UNDECLARED",
        crate::drift::DriftKind::MissingField => "FIELD",
        crate::drift::DriftKind::MissingReferencedFile => "REF",
        crate::drift::DriftKind::PolicyViolation => "POLICY",
        crate::drift::DriftKind::DanglingDependency => "DEPENDS",
        crate::drift::DriftKind::CircularDependency => "CYCLE",
    }
}

fn drift_row(item: &DriftItem) -> String {
    let (icon, sev) = match item.severity {
        Severity::Error => ("❌", "Error"),
        Severity::Warning => ("⚠️", "Warning"),
    };
    format!(
        "| {} {} | {} | `{}` | {} |",
        icon,
        sev,
        drift_kind_label(&item.kind),
        item.service,
        item.message
    )
}

/// Render a drift report as Markdown, suitable for GitHub PR comments.
pub fn render_check_markdown(report: &DriftReport, ping_results: &[PingResult]) -> String {
    let mut out = String::new();
    let errors = report.error_count();
    let warnings = report.warning_count();

    writeln!(out, "## 🔍 svccat drift check").unwrap();
    writeln!(out).unwrap();
    writeln!(
        out,
        "**{} declared · {} discovered** — `{}`",
        report.declared, report.discovered, report.manifest
    )
    .unwrap();
    writeln!(out).unwrap();

    if report.drifts.is_empty() {
        writeln!(out, "✅ **No drift detected**").unwrap();
    } else {
        writeln!(
            out,
            "❌ **DRIFT DETECTED** ({} error{}, {} warning{})",
            errors,
            if errors == 1 { "" } else { "s" },
            warnings,
            if warnings == 1 { "" } else { "s" }
        )
        .unwrap();
        writeln!(out).unwrap();
        writeln!(out, "| Severity | Kind | Service | Message |").unwrap();
        writeln!(out, "|----------|------|---------|---------|").unwrap();
        for item in &report.drifts {
            writeln!(out, "{}", drift_row(item)).unwrap();
        }
    }

    if !ping_results.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "### Ping Results").unwrap();
        writeln!(out).unwrap();
        writeln!(out, "| Status | Service | URL | Detail |").unwrap();
        writeln!(out, "|--------|---------|-----|--------|").unwrap();
        for r in ping_results {
            let row = match &r.ping {
                PingStatus::Reachable { code } if *code < 400 => {
                    format!("| ✅ OK | `{}` | {} | HTTP {} |", r.service, r.url, code)
                }
                PingStatus::Reachable { code } => {
                    format!("| ⚠️ Warn | `{}` | {} | HTTP {} |", r.service, r.url, code)
                }
                PingStatus::Unreachable { reason } => {
                    format!("| ❌ Fail | `{}` | {} | {} |", r.service, r.url, reason)
                }
            };
            writeln!(out, "{}", row).unwrap();
        }
    }

    out
}

/// Render a `--since` diff as Markdown, suitable for GitHub PR comments.
pub fn render_since_diff_markdown(
    old_report: &DriftReport,
    new_report: &DriftReport,
    git_ref: &str,
) -> String {
    use std::collections::HashSet;

    let old_keys: HashSet<String> = old_report.drifts.iter().map(drift_key).collect();
    let new_keys: HashSet<String> = new_report.drifts.iter().map(drift_key).collect();

    let added: Vec<&DriftItem> = new_report
        .drifts
        .iter()
        .filter(|d| !old_keys.contains(&drift_key(d)))
        .collect();
    let resolved: Vec<&DriftItem> = old_report
        .drifts
        .iter()
        .filter(|d| !new_keys.contains(&drift_key(d)))
        .collect();
    let unchanged = new_report.drifts.len().saturating_sub(added.len());

    let mut out = String::new();
    writeln!(out, "## 🔍 svccat drift check (since `{}`)", git_ref).unwrap();
    writeln!(out).unwrap();
    writeln!(
        out,
        "**{} declared · {} discovered** — `{}`",
        new_report.declared, new_report.discovered, new_report.manifest
    )
    .unwrap();
    writeln!(out).unwrap();

    if added.is_empty() && resolved.is_empty() {
        writeln!(
            out,
            "✅ **No change in drift since `{}`** ({} existing item{})",
            git_ref,
            unchanged,
            if unchanged == 1 { "" } else { "s" }
        )
        .unwrap();
        return out;
    }

    if !added.is_empty() {
        writeln!(out, "### ❌ New drift since `{}`", git_ref).unwrap();
        writeln!(out).unwrap();
        writeln!(out, "| Severity | Kind | Service | Message |").unwrap();
        writeln!(out, "|----------|------|---------|---------|").unwrap();
        for item in &added {
            writeln!(out, "{}", drift_row(item)).unwrap();
        }
        writeln!(out).unwrap();
    }

    if !resolved.is_empty() {
        writeln!(out, "### ✅ Resolved since `{}`", git_ref).unwrap();
        writeln!(out).unwrap();
        writeln!(out, "| Kind | Service | Message |").unwrap();
        writeln!(out, "|------|---------|---------|").unwrap();
        for item in &resolved {
            writeln!(
                out,
                "| {} | `{}` | {} |",
                drift_kind_label(&item.kind),
                item.service,
                item.message
            )
            .unwrap();
        }
        writeln!(out).unwrap();
    }

    if unchanged > 0 {
        writeln!(
            out,
            "> {} existing drift item{} unchanged",
            unchanged,
            if unchanged == 1 { "" } else { "s" }
        )
        .unwrap();
    }

    out
}

fn drift_key(item: &DriftItem) -> String {
    format!(
        "{:?}|{}|{}",
        item.kind,
        item.service,
        item.detail.as_deref().unwrap_or("")
    )
}
