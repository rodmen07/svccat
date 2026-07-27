//! Binary-level tests for the orphaned-SBOM-sidecar recovery path
//! (`svccat snapshot save --sbom` / `svccat snapshot delete`).
//!
//! The unit half of this coverage lives in `src/snapshot.rs`; this file
//! follows the `tests/cli_binary_tests.rs` precedent and spawns the real
//! compiled binary, because the half-finished-state defect it guards was a
//! WIRING property: `main.rs` called `snapshot::save` (which wrote the
//! snapshot json) before `snapshot::save_sbom` (which bailed on an existing
//! orphaned sidecar), so the command exited nonzero having already mutated
//! the filesystem. No in-process test of either function alone can catch
//! that ordering; only running the real `Save` arm can.
//!
//! Fixture shape: an orphaned sidecar is `.svccat/snapshots/<name>.spdx.json`
//! existing while `.svccat/snapshots/<name>.json` does not — the state a
//! hand-deleted snapshot file, a partial copy, or a cleaned checkout leaves
//! behind. Its content never matters (the precondition is an existence
//! check), so a one-line placeholder is written.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// A minimal repo root: one declared service, no discovery drift needed —
/// these tests exercise snapshot bookkeeping, not drift analysis.
fn repo_with_manifest() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    std::fs::write(
        dir.path().join("services.yaml"),
        r#"version: "1"

services:
  - name: billing
    language: Rust
"#,
    )
    .expect("write services.yaml");
    dir
}

fn snapshots_dir(root: &Path) -> PathBuf {
    root.join(".svccat").join("snapshots")
}

/// Create the orphan state: sidecar present, snapshot json absent.
fn write_orphaned_sidecar(root: &Path, name: &str) -> PathBuf {
    let dir = snapshots_dir(root);
    std::fs::create_dir_all(&dir).expect("create snapshots dir");
    let sidecar = dir.join(format!("{name}.spdx.json"));
    std::fs::write(&sidecar, "{}\n").expect("write orphan sidecar");
    sidecar
}

fn svccat(root: &Path) -> Command {
    let mut cmd = Command::cargo_bin("svccat").expect("binary");
    cmd.arg("--root").arg(root);
    cmd
}

/// The half-finished state is gone: with an orphaned sidecar in place,
/// `snapshot save <name> --sbom` fails BEFORE writing the snapshot json.
/// Pre-fix, this command exited nonzero having already written
/// `.svccat/snapshots/v1.json` — the assertion on that path's absence is
/// what fails if the `main.rs` precondition call is severed (NC-2).
#[test]
fn save_sbom_with_orphaned_sidecar_fails_without_writing_the_snapshot() {
    let dir = repo_with_manifest();
    let root = dir.path();
    write_orphaned_sidecar(root, "v1");

    svccat(root)
        .args(["snapshot", "save", "v1", "--sbom"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"))
        .stderr(predicate::str::contains("svccat snapshot delete v1"));

    assert!(
        !snapshots_dir(root).join("v1.json").exists(),
        "the snapshot json must NOT be written when the sidecar precondition fails"
    );
}

/// `snapshot delete <name>` on the orphan state succeeds and removes the
/// sidecar — the recovery the error message above points at. Pre-fix this
/// exited nonzero ("snapshot 'v1' not found") and left the sidecar behind.
#[test]
fn delete_removes_an_orphaned_sidecar_via_the_binary() {
    let dir = repo_with_manifest();
    let root = dir.path();
    let sidecar = write_orphaned_sidecar(root, "v1");

    svccat(root)
        .args(["snapshot", "delete", "v1"])
        .assert()
        .success()
        .stderr(predicate::str::contains("orphaned SBOM sidecar"));

    assert!(!sidecar.exists(), "delete must remove the orphaned sidecar");
}

/// Deleting a name with neither snapshot nor sidecar is still an error:
/// the orphan fix must not turn `delete` into a silent no-op on typos.
#[test]
fn delete_with_nothing_to_delete_still_fails_via_the_binary() {
    let dir = repo_with_manifest();
    let root = dir.path();
    std::fs::create_dir_all(snapshots_dir(root)).expect("create snapshots dir");

    svccat(root)
        .args(["snapshot", "delete", "missing"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

/// The full recovery, end to end, exactly as the error message instructs:
/// orphan blocks `save --sbom`; `delete` clears it; `save --sbom` then
/// succeeds and both files exist.
#[test]
fn orphan_recovery_end_to_end() {
    let dir = repo_with_manifest();
    let root = dir.path();
    write_orphaned_sidecar(root, "v1");

    svccat(root)
        .args(["snapshot", "save", "v1", "--sbom"])
        .assert()
        .failure();

    svccat(root)
        .args(["snapshot", "delete", "v1"])
        .assert()
        .success();

    svccat(root)
        .args(["snapshot", "save", "v1", "--sbom"])
        .assert()
        .success()
        .stderr(predicate::str::contains("wrote SPDX SBOM"));

    assert!(snapshots_dir(root).join("v1.json").exists());
    assert!(snapshots_dir(root).join("v1.spdx.json").exists());
}
