use crate::manifest::ServiceEntry;
use anyhow::Result;
use colored::Colorize;
use serde::Deserialize;
use std::collections::HashMap;
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
    let before_drift: std::collections::HashSet<String> = before
        .drift
        .iter()
        .map(|d| format!("{}:{}", d.service, d.message))
        .collect();
    let after_drift: std::collections::HashSet<String> = after
        .drift
        .iter()
        .map(|d| format!("{}:{}", d.service, d.message))
        .collect();

    let new_drift: Vec<String> = after
        .drift
        .iter()
        .filter(|d| !before_drift.contains(&format!("{}:{}", d.service, d.message)))
        .map(|d| {
            format!(
                "[{}] {} — {}",
                d.severity.to_uppercase(),
                d.service,
                d.message
            )
        })
        .collect();

    let resolved_drift: Vec<String> = before
        .drift
        .iter()
        .filter(|d| !after_drift.contains(&format!("{}:{}", d.service, d.message)))
        .map(|d| {
            format!(
                "[{}] {} — {}",
                d.severity.to_uppercase(),
                d.service,
                d.message
            )
        })
        .collect();

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
    let before_snap: Snapshot =
        serde_json::from_value(before.clone()).map_err(|e| anyhow::anyhow!("invalid before snapshot: {e}"))?;
    let after_snap: Snapshot =
        serde_json::from_value(after.clone()).map_err(|e| anyhow::anyhow!("invalid after snapshot: {e}"))?;

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
                changed.push(ServiceDiff { name: name.to_string(), changes });
            }
        }
    }

    let before_drift: std::collections::HashSet<String> = before
        .drift
        .iter()
        .map(|d| format!("{}:{}", d.service, d.message))
        .collect();
    let after_drift: std::collections::HashSet<String> = after
        .drift
        .iter()
        .map(|d| format!("{}:{}", d.service, d.message))
        .collect();

    let new_drift: Vec<String> = after_drift.difference(&before_drift).cloned().collect();
    let resolved_drift: Vec<String> = before_drift.difference(&after_drift).cloned().collect();

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
