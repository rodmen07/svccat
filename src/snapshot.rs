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

fn sbom_path(root: &Path, name: &str) -> PathBuf {
    snapshots_dir(root).join(format!("{name}.spdx.json"))
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

/// Error if the SBOM sidecar for `name` already exists.
///
/// Public so the CLI can run this precondition BEFORE `save` writes the
/// snapshot json: `save`-then-`save_sbom` used to fail on an existing
/// (orphaned) sidecar only AFTER the snapshot was written, leaving a
/// half-finished state (snapshot saved, command exited nonzero). The error
/// names `svccat snapshot delete <name>` because that command removes the
/// sidecar whether or not the snapshot json exists, so it is the one
/// recovery that always works.
pub fn ensure_no_sbom_sidecar(root: &Path, name: &str) -> Result<()> {
    let path = sbom_path(root, name);
    if path.exists() {
        bail!(
            "SBOM sidecar already exists ({}). Run `svccat snapshot delete {}` to remove it first.",
            path.display(),
            name
        );
    }
    Ok(())
}

/// Write an SPDX 2.3 JSON SBOM sidecar for a snapshot at
/// `<root>/.svccat/snapshots/<name>.spdx.json`. Errors if the sidecar
/// already exists. Does not print; the caller reports the path.
pub fn save_sbom(root: &Path, name: &str, manifest: &Manifest) -> Result<PathBuf> {
    ensure_no_sbom_sidecar(root, name)?;
    let path = sbom_path(root, name);
    let spdx = crate::output::spdx::render_export(manifest)?;
    std::fs::write(&path, spdx).with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
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

/// Delete a snapshot by name, along with its SBOM sidecar if present.
///
/// If the snapshot json is missing but an orphaned SBOM sidecar exists
/// (hand-deleted file, partial copy, cleaned checkout), the sidecar is
/// removed and the call succeeds: the orphan is exactly what blocks
/// `save --sbom` for that name, so `delete` doubles as the recovery the
/// `ensure_no_sbom_sidecar` error message points at. Only when NEITHER
/// file exists is "snapshot not found" an error.
pub fn delete(root: &Path, name: &str) -> Result<()> {
    let path = snapshot_path(root, name);
    let sidecar = sbom_path(root, name);
    if !path.exists() {
        if sidecar.exists() {
            std::fs::remove_file(&sidecar)
                .with_context(|| format!("deleting {}", sidecar.display()))?;
            eprintln!(
                "snapshot '{}' not found; removed orphaned SBOM sidecar ({})",
                name,
                sidecar.display()
            );
            return Ok(());
        }
        bail!("snapshot '{}' not found", name);
    }
    std::fs::remove_file(&path).with_context(|| format!("deleting {}", path.display()))?;
    if sidecar.exists() {
        let _ = std::fs::remove_file(&sidecar);
    }
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
        let dt = crate::timefmt::human_utc(s.created_at);
        println!("{:<20}  {:<24}  {}", s.name, dt, s.manifest);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drift::DriftReport;
    use crate::manifest::{Manifest, ServiceEntry};
    use tempfile::TempDir;

    fn sample_manifest() -> Manifest {
        let mut manifest = Manifest::default();
        manifest.services.push(ServiceEntry {
            name: "auth-service".to_string(),
            team: Some("security".to_string()),
            ..Default::default()
        });
        manifest
    }

    fn save_sample(root: &Path, name: &str) -> Manifest {
        let manifest = sample_manifest();
        let report = DriftReport::default();
        save(root, name, &manifest, &report).unwrap();
        manifest
    }

    #[test]
    fn save_sbom_writes_spdx_sidecar() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let manifest = save_sample(root, "v1");

        let path = save_sbom(root, "v1", &manifest).unwrap();
        assert_eq!(path, snapshots_dir(root).join("v1.spdx.json"));
        let text = std::fs::read_to_string(&path).unwrap();
        let v: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["spdxVersion"], "SPDX-2.3");
    }

    #[test]
    fn save_sbom_bails_when_sidecar_exists() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let manifest = save_sample(root, "v1");

        save_sbom(root, "v1", &manifest).unwrap();
        let err = save_sbom(root, "v1", &manifest).unwrap_err();
        assert!(err.to_string().contains("already exists"));
    }

    #[test]
    fn delete_removes_snapshot_and_sidecar() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let manifest = save_sample(root, "v1");
        save_sbom(root, "v1", &manifest).unwrap();

        delete(root, "v1").unwrap();
        assert!(!snapshot_path(root, "v1").exists());
        assert!(!snapshots_dir(root).join("v1.spdx.json").exists());
    }

    #[test]
    fn delete_without_sidecar_still_succeeds() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        save_sample(root, "v1");

        delete(root, "v1").unwrap();
        assert!(!snapshot_path(root, "v1").exists());
    }

    /// The v1.6.1 edge case, FIXED: the snapshot json is gone but its SBOM
    /// sidecar is not (hand-deleted file, partial copy, cleaned checkout).
    /// `delete` used to bail before its sidecar cleanup ran, so the sidecar
    /// stayed orphaned and then blocked `save --sbom` for that name with no
    /// documented recovery. `delete` now removes the orphan and succeeds.
    ///
    /// DELIBERATE behavior change: this supersedes the previous pin
    /// (`delete_with_missing_snapshot_bails_and_leaves_sidecar_orphaned`,
    /// PR #38), per the backlog's LOW bug entry filed 2026-07-26 — the pin's
    /// own doc comment routed the change here rather than blessing the orphan.
    #[test]
    fn delete_with_missing_snapshot_removes_orphaned_sidecar() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let manifest = save_sample(root, "v1");
        let sidecar = save_sbom(root, "v1", &manifest).unwrap();

        std::fs::remove_file(snapshot_path(root, "v1")).unwrap();

        delete(root, "v1").unwrap();
        assert!(!sidecar.exists(), "orphaned sidecar should be removed");

        // The recovery is complete: the name is usable again.
        save_sbom(root, "v1", &manifest).unwrap();
    }

    /// The error path survives the fix: when NEITHER the snapshot json nor
    /// the sidecar exists, `delete` still reports "not found" rather than
    /// silently succeeding on nothing.
    #[test]
    fn delete_with_neither_snapshot_nor_sidecar_still_bails() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        std::fs::create_dir_all(snapshots_dir(root)).unwrap();

        let err = delete(root, "missing").unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    /// The precondition error names the one recovery that always works
    /// (`svccat snapshot delete <name>`), and stays quiet when no sidecar
    /// exists for the name.
    #[test]
    fn ensure_no_sbom_sidecar_error_names_the_delete_recovery() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let manifest = save_sample(root, "v1");
        save_sbom(root, "v1", &manifest).unwrap();

        let err = ensure_no_sbom_sidecar(root, "v1").unwrap_err();
        assert!(err.to_string().contains("svccat snapshot delete v1"));
        assert!(ensure_no_sbom_sidecar(root, "v2").is_ok());
    }

    #[test]
    fn list_skips_sbom_sidecars() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let manifest = save_sample(root, "v1");
        save_sbom(root, "v1", &manifest).unwrap();

        let snaps = list(root).unwrap();
        assert_eq!(snaps.len(), 1);
        assert_eq!(snaps[0].name, "v1");
    }
}
