//! `svccat export --format backstage-yaml` must emit the same bytes every time
//! it is given the same manifest.
//!
//! A Backstage `catalog-info.yaml` is a file you COMMIT, so this is a
//! byte-determinism contract rather than a display-order preference: if the
//! bytes move on their own, every regeneration produces a diff on services
//! nothing changed about, and a `git diff --exit-code` drift check in CI reports
//! a change that is not one.
//!
//! `CatalogMetadata::annotations` used to be a `HashMap`, which `serde_yaml`
//! serializes in iteration order — randomised per process by `RandomState`. Ten
//! runs of the pre-fix binary over a single four-annotation service produced
//! NINE distinct orderings.
//!
//! These are binary-level tests on purpose, and they complement rather than
//! duplicate the in-process ones in `src/output/backstage.rs`: separate
//! PROCESSES are what a user actually experiences (`svccat export` today,
//! `svccat export` next week), and only this level exercises the `export`
//! command's wiring — the `--output <file>` path in particular, which writes the
//! artifact this contract is about.

use assert_cmd::Command;
use std::collections::BTreeSet;
use std::fs;
use tempfile::TempDir;

/// How many separate processes each determinism assertion samples.
///
/// Four annotation keys give 4! = 24 orderings, and the pre-fix binary really
/// did wander over them (9 distinct in 10 runs), so the chance of ten runs
/// agreeing by luck is negligible. One run would prove nothing whatever.
const RUNS: usize = 10;

/// One service declaring every annotation the exporter knows how to emit:
/// `oncall`, `path`, `docs` and `ci`.
const FULLY_ANNOTATED_CATALOG: &str = "\
version: \"1\"

services:
  - name: auth
    oncall: \"@sec-team\"
    path: services/auth
    docs: docs/auth.md
    ci: .github/workflows/auth.yml
";

/// Two services, so the multi-document (`---`-separated) form is covered too.
const TWO_ANNOTATED_SERVICES: &str = "\
version: \"1\"

services:
  - name: auth
    oncall: \"@sec-team\"
    path: services/auth
    docs: docs/auth.md
  - name: billing
    oncall: \"@payments\"
    path: services/billing
    ci: .github/workflows/billing.yml
";

/// Run `svccat export --format backstage-yaml` in a temp dir holding
/// `manifest_yaml`, and return stdout.
fn export_stdout(manifest_yaml: &str) -> String {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("services.yaml"), manifest_yaml).unwrap();

    let out = Command::cargo_bin("svccat")
        .unwrap()
        .current_dir(tmp.path())
        .env("NO_COLOR", "1")
        .arg("export")
        .arg("--format")
        .arg("backstage-yaml")
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "export exited {:?}, stderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// Run the same export with `--output <file>` and return the file's contents.
fn export_to_file(manifest_yaml: &str) -> String {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("services.yaml"), manifest_yaml).unwrap();
    let out_path = tmp.path().join("catalog-info.yaml");

    let out = Command::cargo_bin("svccat")
        .unwrap()
        .current_dir(tmp.path())
        .env("NO_COLOR", "1")
        .arg("export")
        .arg("--format")
        .arg("backstage-yaml")
        .arg("--output")
        .arg(&out_path)
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "export --output exited {:?}, stderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    fs::read_to_string(&out_path).unwrap()
}

/// The `svccat.io/*` annotation keys of an exported document, in printed order.
///
/// Panics when fewer than two are present, so an export that emitted no
/// annotations at all can never masquerade as a stable one: two empty lists
/// compare equal, and every assertion in this file would pass vacuously.
fn annotation_keys(yaml: &str) -> Vec<String> {
    let keys: Vec<String> = yaml
        .lines()
        .map(|l| l.trim())
        .filter(|l| l.starts_with("svccat.io/"))
        .map(|l| {
            l.split(':')
                .next()
                .expect("an annotation line is `key: value`")
                .to_string()
        })
        .collect();

    assert!(
        keys.len() >= 2,
        "expected at least two `svccat.io/` annotations, got {keys:?} from:\n{yaml}"
    );
    keys
}

// ── Determinism: the defect itself ──────────────────────────────────────────

#[test]
fn repeated_exports_of_one_manifest_are_byte_identical() {
    let outputs: BTreeSet<String> = (0..RUNS)
        .map(|_| export_stdout(FULLY_ANNOTATED_CATALOG))
        .collect();

    assert_eq!(
        outputs.len(),
        1,
        "{RUNS} exports of one unchanged manifest produced {} distinct documents:\n{:#?}",
        outputs.len(),
        outputs
    );

    // Not a restatement of the line above: it guards against the whole document
    // becoming stable for a reason that has nothing to do with annotations
    // (an empty catalog, say), which the emptiness check inside
    // `annotation_keys` would then be the only thing to catch.
    annotation_keys(outputs.iter().next().unwrap());
}

#[test]
fn a_multi_service_export_is_byte_identical_across_runs() {
    let outputs: BTreeSet<String> = (0..RUNS)
        .map(|_| export_stdout(TWO_ANNOTATED_SERVICES))
        .collect();

    assert_eq!(
        outputs.len(),
        1,
        "{RUNS} exports of one unchanged two-service manifest produced {} distinct documents:\n{:#?}",
        outputs.len(),
        outputs
    );
}

#[test]
fn the_file_written_by_output_is_byte_identical_across_runs() {
    let files: BTreeSet<String> = (0..RUNS)
        .map(|_| export_to_file(FULLY_ANNOTATED_CATALOG))
        .collect();

    assert_eq!(
        files.len(),
        1,
        "{RUNS} `--output` writes of one unchanged manifest produced {} distinct files:\n{:#?}",
        files.len(),
        files
    );
}

// ── Which order, not merely that there is one (L-072) ───────────────────────

#[test]
fn annotations_are_exported_in_alphabetical_key_order() {
    // Sampled over RUNS processes rather than one: with the pre-fix binary a
    // single run landed on the sorted order about one time in twenty-four, so a
    // one-shot assertion here would have been a lottery ticket, not a control.
    let expected = vec![
        "svccat.io/ci".to_string(),
        "svccat.io/docs".to_string(),
        "svccat.io/oncall".to_string(),
        "svccat.io/path".to_string(),
    ];

    for run in 0..RUNS {
        assert_eq!(
            annotation_keys(&export_stdout(FULLY_ANNOTATED_CATALOG)),
            expected,
            "run {run}: annotation keys should be sorted"
        );
    }
}

// ── The export is otherwise unchanged ───────────────────────────────────────

#[test]
fn the_exported_document_still_carries_every_annotation_and_its_value() {
    let yaml = export_stdout(FULLY_ANNOTATED_CATALOG);

    for (key, value) in [
        ("svccat.io/ci", ".github/workflows/auth.yml"),
        ("svccat.io/docs", "docs/auth.md"),
        ("svccat.io/oncall", "@sec-team"),
        ("svccat.io/path", "services/auth"),
    ] {
        assert!(
            yaml.contains(key),
            "annotation `{key}` missing from export:\n{yaml}"
        );
        assert!(
            yaml.contains(value),
            "value of `{key}` missing from export:\n{yaml}"
        );
    }

    assert!(
        yaml.contains("apiVersion: backstage.io/v1alpha1"),
        "export is not a Backstage catalog document:\n{yaml}"
    );
    assert!(
        yaml.contains("kind: Component"),
        "export is not a Backstage catalog document:\n{yaml}"
    );
}

#[test]
fn stdout_and_the_output_file_carry_the_same_bytes() {
    assert_eq!(
        export_stdout(FULLY_ANNOTATED_CATALOG),
        export_to_file(FULLY_ANNOTATED_CATALOG),
        "`--output` must write exactly what stdout prints"
    );
}
