use crate::manifest::ServiceEntry;
use anyhow::Result;
use colored::Colorize;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

// ── Snapshot (subset of export JSON we care about) ───────────────────────────

#[derive(Debug, Deserialize)]
struct Snapshot {
    #[serde(default)]
    services: Vec<ServiceEntry>,
    #[serde(default)]
    drift: Vec<DriftSummaryItem>,
}

#[derive(Debug, Deserialize)]
struct DriftSummaryItem {
    service: String,
    message: String,
    severity: String,
}

// ── Diff types ────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct FieldChange {
    pub field: String,
    pub before: String,
    pub after: String,
}

#[derive(Debug)]
pub struct ServiceDiff {
    pub name: String,
    pub changes: Vec<FieldChange>,
}

#[derive(Debug)]
pub struct DiffReport {
    pub before_path: String,
    pub after_path: String,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<ServiceDiff>,
    pub new_drift: Vec<String>,
    pub resolved_drift: Vec<String>,
}

impl DiffReport {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty()
            && self.removed.is_empty()
            && self.changed.is_empty()
            && self.new_drift.is_empty()
            && self.resolved_drift.is_empty()
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Load two JSON export snapshots and compute the diff.
pub fn diff_snapshots(before_path: &Path, after_path: &Path) -> Result<DiffReport> {
    let before_text = std::fs::read_to_string(before_path)
        .map_err(|e| anyhow::anyhow!("cannot read {}: {e}", before_path.display()))?;
    let after_text = std::fs::read_to_string(after_path)
        .map_err(|e| anyhow::anyhow!("cannot read {}: {e}", after_path.display()))?;

    let before: Snapshot = serde_json::from_str(&before_text)
        .map_err(|e| anyhow::anyhow!("cannot parse {}: {e}", before_path.display()))?;
    let after: Snapshot = serde_json::from_str(&after_text)
        .map_err(|e| anyhow::anyhow!("cannot parse {}: {e}", after_path.display()))?;

    let before_map: HashMap<&str, &ServiceEntry> = before
        .services
        .iter()
        .map(|s| (s.name.as_str(), s))
        .collect();
    let after_map: HashMap<&str, &ServiceEntry> = after
        .services
        .iter()
        .map(|s| (s.name.as_str(), s))
        .collect();

    // Added / removed services
    let added: Vec<String> = after
        .services
        .iter()
        .filter(|s| !before_map.contains_key(s.name.as_str()))
        .map(|s| s.name.clone())
        .collect();

    let removed: Vec<String> = before
        .services
        .iter()
        .filter(|s| !after_map.contains_key(s.name.as_str()))
        .map(|s| s.name.clone())
        .collect();

    // Changed services (field-level diff on services present in both)
    let mut changed = Vec::new();
    for (name, before_svc) in &before_map {
        if let Some(after_svc) = after_map.get(name) {
            let changes = field_diff(before_svc, after_svc);
            if !changes.is_empty() {
                changed.push(ServiceDiff {
                    name: name.to_string(),
                    changes,
                });
            }
        }
    }
    changed.sort_by(|a, b| a.name.cmp(&b.name));

    // Drift changes
    let (new_drift, resolved_drift) = drift_changes(&before.drift, &after.drift);

    Ok(DiffReport {
        before_path: before_path.display().to_string(),
        after_path: after_path.display().to_string(),
        added,
        removed,
        changed,
        new_drift,
        resolved_drift,
    })
}

/// Render a diff report as a Markdown document.
pub fn render_diff_markdown(report: &DiffReport) {
    use std::fmt::Write;
    let mut out = String::new();

    writeln!(out, "# svccat diff").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "| | Path |").unwrap();
    writeln!(out, "|-|------|").unwrap();
    writeln!(out, "| Before | `{}` |", report.before_path).unwrap();
    writeln!(out, "| After  | `{}` |", report.after_path).unwrap();

    if report.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "> No changes detected.").unwrap();
        print!("{out}");
        return;
    }

    if !report.added.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "## Services Added ({})", report.added.len()).unwrap();
        writeln!(out).unwrap();
        for name in &report.added {
            writeln!(out, "- `{name}`").unwrap();
        }
    }

    if !report.removed.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "## Services Removed ({})", report.removed.len()).unwrap();
        writeln!(out).unwrap();
        for name in &report.removed {
            writeln!(out, "- `{name}`").unwrap();
        }
    }

    if !report.changed.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "## Services Changed ({})", report.changed.len()).unwrap();
        writeln!(out).unwrap();
        writeln!(out, "| Service | Field | Before | After |").unwrap();
        writeln!(out, "|---------|-------|--------|-------|").unwrap();
        for svc in &report.changed {
            for fc in &svc.changes {
                writeln!(
                    out,
                    "| `{}` | `{}` | {} | {} |",
                    svc.name, fc.field, fc.before, fc.after
                )
                .unwrap();
            }
        }
    }

    if !report.new_drift.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "## New Drift ({})", report.new_drift.len()).unwrap();
        writeln!(out).unwrap();
        for msg in &report.new_drift {
            writeln!(out, "- {msg}").unwrap();
        }
    }

    if !report.resolved_drift.is_empty() {
        writeln!(out).unwrap();
        writeln!(out, "## Resolved Drift ({})", report.resolved_drift.len()).unwrap();
        writeln!(out).unwrap();
        for msg in &report.resolved_drift {
            writeln!(out, "- {msg}").unwrap();
        }
    }

    print!("{out}");
}

/// Render a diff report to the terminal.
pub fn render_diff(report: &DiffReport) {
    println!(
        "{}",
        format!(
            "svccat diff: {} → {}",
            report.before_path, report.after_path
        )
        .bold()
    );

    if report.is_empty() {
        println!("\n  {} No changes detected", "OK".green().bold());
        return;
    }

    if !report.added.is_empty() {
        println!(
            "\n  {} ({}):",
            "Services added".green().bold(),
            report.added.len()
        );
        for name in &report.added {
            println!("    {}  {}", "+".green().bold(), name);
        }
    }

    if !report.removed.is_empty() {
        println!(
            "\n  {} ({}):",
            "Services removed".red().bold(),
            report.removed.len()
        );
        for name in &report.removed {
            println!("    {}  {}", "-".red().bold(), name);
        }
    }

    if !report.changed.is_empty() {
        println!(
            "\n  {} ({}):",
            "Services changed".yellow().bold(),
            report.changed.len()
        );
        for svc in &report.changed {
            println!("    {}  {}", "~".yellow().bold(), svc.name.bold());
            for fc in &svc.changes {
                println!(
                    "       {}: {} → {}",
                    fc.field,
                    fc.before.red(),
                    fc.after.green()
                );
            }
        }
    }

    if !report.new_drift.is_empty() {
        println!(
            "\n  {} ({}):",
            "New drift".red().bold(),
            report.new_drift.len()
        );
        for msg in &report.new_drift {
            println!("    {}  {}", "+".red().bold(), msg);
        }
    }

    if !report.resolved_drift.is_empty() {
        println!(
            "\n  {} ({}):",
            "Resolved drift".green().bold(),
            report.resolved_drift.len()
        );
        for msg in &report.resolved_drift {
            println!("    {}  {}", "✓".green().bold(), msg);
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// The identity two snapshots compare drift entries by: same service, same
/// message. Severity is deliberately excluded, so a drift item whose severity was
/// re-classified between snapshots is not reported as one resolved plus one new.
fn drift_key(item: &DriftSummaryItem) -> String {
    format!("{}:{}", item.service, item.message)
}

/// One rendered drift line, as both renderers print it.
fn drift_line(item: &DriftSummaryItem) -> String {
    format!(
        "[{}] {} — {}",
        item.severity.to_uppercase(),
        item.service,
        item.message
    )
}

/// The `(new_drift, resolved_drift)` pair for two snapshots' drift lists.
///
/// Both lists walk their SOURCE vector in order instead of differencing two hash
/// sets, so the output follows the snapshot the reader is looking at and is
/// identical across runs; `HashSet` iteration order is unspecified and is
/// re-randomised per set by the default hasher, which made these lists shuffle
/// between two runs over byte-identical input. Each `service:message` is reported
/// once, keeping its first occurrence, because nothing upstream dedupes the drift
/// vector.
///
/// This is the single implementation behind BOTH `diff_snapshots` (the
/// `svccat diff` path) and `build_diff` (the `svccat snapshot diff` path). They
/// used to compute these lists separately and disagreed on both the order and the
/// text: one emitted the severity-prefixed line, the other the raw
/// `service:message` key, out of one public `DiffReport` field.
/// `tests/diff_drift_order_tests.rs::both_diff_entry_points_produce_identical_drift_lists`
/// reads both entry points and is what keeps them from splitting again.
fn drift_changes(
    before: &[DriftSummaryItem],
    after: &[DriftSummaryItem],
) -> (Vec<String>, Vec<String>) {
    fn only_in(source: &[DriftSummaryItem], other: &[DriftSummaryItem]) -> Vec<String> {
        let other_keys: HashSet<String> = other.iter().map(drift_key).collect();
        let mut seen: HashSet<String> = HashSet::new();
        source
            .iter()
            .filter(|item| {
                let key = drift_key(item);
                !other_keys.contains(&key) && seen.insert(key)
            })
            .map(drift_line)
            .collect()
    }

    (only_in(after, before), only_in(before, after))
}

fn opt_str(v: &Option<String>) -> String {
    v.as_deref().unwrap_or("(none)").to_string()
}

fn field_diff(before: &ServiceEntry, after: &ServiceEntry) -> Vec<FieldChange> {
    let mut changes = Vec::new();

    macro_rules! check_field {
        ($field:ident) => {
            if before.$field != after.$field {
                changes.push(FieldChange {
                    field: stringify!($field).to_string(),
                    before: opt_str(&before.$field),
                    after: opt_str(&after.$field),
                });
            }
        };
    }

    check_field!(language);
    check_field!(platform);
    check_field!(role);
    check_field!(url);
    check_field!(docs);
    check_field!(ci);

    if before.depends_on != after.depends_on {
        changes.push(FieldChange {
            field: "depends_on".to_string(),
            before: format!("[{}]", before.depends_on.join(", ")),
            after: format!("[{}]", after.depends_on.join(", ")),
        });
    }

    changes
}

// ── Diff from in-memory JSON (for snapshot diff) ──────────────────────────────

/// Compute a diff between two snapshot JSON payloads without touching the filesystem.
///
/// `before_label` and `after_label` are used only as display names in the report.
pub fn diff_from_json(
    before: &serde_json::Value,
    after: &serde_json::Value,
    before_label: &str,
    after_label: &str,
) -> Result<DiffReport> {
    let before_snap: Snapshot = serde_json::from_value(before.clone())
        .map_err(|e| anyhow::anyhow!("invalid before snapshot: {e}"))?;
    let after_snap: Snapshot = serde_json::from_value(after.clone())
        .map_err(|e| anyhow::anyhow!("invalid after snapshot: {e}"))?;

    build_diff(before_snap, after_snap, before_label, after_label)
}

fn build_diff(
    before: Snapshot,
    after: Snapshot,
    before_label: &str,
    after_label: &str,
) -> Result<DiffReport> {
    use std::collections::HashMap;

    let before_map: HashMap<&str, &ServiceEntry> = before
        .services
        .iter()
        .map(|s| (s.name.as_str(), s))
        .collect();
    let after_map: HashMap<&str, &ServiceEntry> = after
        .services
        .iter()
        .map(|s| (s.name.as_str(), s))
        .collect();

    let added: Vec<String> = after
        .services
        .iter()
        .filter(|s| !before_map.contains_key(s.name.as_str()))
        .map(|s| s.name.clone())
        .collect();

    let removed: Vec<String> = before
        .services
        .iter()
        .filter(|s| !after_map.contains_key(s.name.as_str()))
        .map(|s| s.name.clone())
        .collect();

    let mut changed = Vec::new();
    let mut sorted_keys: Vec<&str> = before_map.keys().cloned().collect();
    sorted_keys.sort_unstable();
    for name in sorted_keys {
        if let Some(after_svc) = after_map.get(name) {
            let changes = field_diff(before_map[name], after_svc);
            if !changes.is_empty() {
                changed.push(ServiceDiff {
                    name: name.to_string(),
                    changes,
                });
            }
        }
    }

    let (new_drift, resolved_drift) = drift_changes(&before.drift, &after.drift);

    Ok(DiffReport {
        before_path: before_label.to_string(),
        after_path: after_label.to_string(),
        added,
        removed,
        changed,
        new_drift,
        resolved_drift,
    })
}
