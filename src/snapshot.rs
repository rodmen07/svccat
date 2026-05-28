use crate::drift::DriftReport;
use crate::manifest::Manifest;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

const SNAPSHOTS_DIR: &str = ".svccat/snapshots";

// ── Snapshot file format ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub name: String,
    pub created_at: u64,
    pub manifest: String,
    /// Full export JSON blob (same schema as `svccat export --format json`).
    pub payload: Value,
}

// ── Path helpers ───────────────────────────────────────────────────────────────

fn snapshots_dir(root: &Path) -> PathBuf {
    root.join(SNAPSHOTS_DIR)
}

fn snapshot_path(root: &Path, name: &str) -> PathBuf {
    snapshots_dir(root).join(format!("{name}.json"))
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ── Public API ─────────────────────────────────────────────────────────────────

/// Save the current manifest + drift report as a named snapshot.
///
/// Writes to `<root>/.svccat/snapshots/<name>.json`.
/// Errors if a snapshot with that name already exists.
pub fn save(root: &Path, name: &str, manifest: &Manifest, report: &DriftReport) -> Result<()> {
    let dir = snapshots_dir(root);
    std::fs::create_dir_all(&dir).context("creating .svccat/snapshots")?;

    let path = snapshot_path(root, name);
    if path.exists() {
        bail!(
            "snapshot '{}' already exists ({}). Use a different name or delete it first.",
            name,
            path.display()
        );
    }

    let payload = serde_json::json!({
        "version": manifest.version,
        "manifest": report.manifest,
        "summary": {
            "declared": report.declared,
            "discovered": report.discovered,
            "errors": report.error_count(),
            "warnings": report.warning_count(),
        },
        "services": manifest.services,
        "drift": report.drifts,
    });

    let snap = Snapshot {
        name: name.to_string(),
        created_at: now_secs(),
        manifest: report.manifest.clone(),
        payload,
    };

    let json = serde_json::to_string_pretty(&snap).context("serialising snapshot")?;
    std::fs::write(&path, json).with_context(|| format!("writing {}", path.display()))?;
    eprintln!("saved snapshot '{}' -> {}", name, path.display());
    Ok(())
}

/// List all snapshots in `<root>/.svccat/snapshots/`.
pub fn list(root: &Path) -> Result<Vec<Snapshot>> {
    let dir = snapshots_dir(root);
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut snaps: Vec<Snapshot> = std::fs::read_dir(&dir)
        .context("reading .svccat/snapshots")?
        .filter_map(|entry| {
            let e = entry.ok()?;
            let path = e.path();
            if path.extension()? != "json" {
                return None;
            }
            let text = std::fs::read_to_string(&path).ok()?;
            serde_json::from_str::<Snapshot>(&text).ok()
        })
        .collect();

    snaps.sort_by_key(|s| s.created_at);
    Ok(snaps)
}

/// Load a snapshot by name.
pub fn load(root: &Path, name: &str) -> Result<Snapshot> {
    let path = snapshot_path(root, name);
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("snapshot '{}' not found", name))?;
    serde_json::from_str(&text).context("parsing snapshot")
}

/// Delete a snapshot by name.
pub fn delete(root: &Path, name: &str) -> Result<()> {
    let path = snapshot_path(root, name);
    if !path.exists() {
        bail!("snapshot '{}' not found", name);
    }
    std::fs::remove_file(&path).with_context(|| format!("deleting {}", path.display()))?;
    eprintln!("deleted snapshot '{}'", name);
    Ok(())
}

/// Compare two named snapshots against each other, returning a `DiffReport`.
///
/// `before_name` is the older snapshot; `after_name` is the newer one.
pub fn compare(
    root: &Path,
    before_name: &str,
    after_name: &str,
) -> Result<crate::diff::DiffReport> {
    let before = load(root, before_name)?;
    let after = load(root, after_name)?;
    crate::diff::diff_from_json(&before.payload, &after.payload, before_name, after_name)
}

// ── Renderers ──────────────────────────────────────────────────────────────────

pub fn render_list(snaps: &[Snapshot]) {
    use colored::Colorize;
    if snaps.is_empty() {
        println!("No snapshots found. Run: svccat snapshot save <name>");
        return;
    }
    println!(
        "{}",
        format!("{:<20}  {:<24}  manifest", "NAME", "CREATED").bold()
    );
    println!("{}", "-".repeat(72).dimmed());
    for s in snaps {
        let dt = format_ts(s.created_at);
        println!("{:<20}  {:<24}  {}", s.name, dt, s.manifest);
    }
}

fn format_ts(secs: u64) -> String {
    // Simple UTC formatting without chrono dependency.
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    let days = secs / 86400;
    // Days since Unix epoch -> approximate calendar date.
    let (y, mo, d) = days_to_ymd(days);
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{m:02}:{s:02} UTC")
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    // Gregorian calendar approximation from day count since 1970-01-01.
    let mut year = 1970u64;
    loop {
        let leap = is_leap(year);
        let days_in_year = if leap { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    let leap = is_leap(year);
    let months = if leap {
        [31u64, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31u64, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u64;
    for &days_in_month in &months {
        if days < days_in_month {
            break;
        }
        days -= days_in_month;
        month += 1;
    }
    (year, month, days + 1)
}

#[allow(clippy::manual_is_multiple_of)]
fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}
