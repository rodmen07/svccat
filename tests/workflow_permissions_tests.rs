//! PR-time drift guards for GITHUB_TOKEN `permissions:` blocks in
//! `.github/workflows/`.
//!
//! Three workflows (`coverage.yml`, `fuzzing.yml`, `publish.yml`) and two
//! `ci.yml` jobs ran for months with no `permissions:` block at all, so their
//! GITHUB_TOKEN carried whatever the repo-level default happens to be. That
//! default (`default_workflow_permissions`) is a settings-page toggle anyone
//! with admin access can flip to read-write without a commit — which matters
//! most for `publish.yml`, the workflow holding the crates.io publish path. An
//! explicit block is checked in and reviewed; these tests make its absence (or
//! a quiet widening) a red PR instead of a silent inheritance.
//!
//! The two tests fail on disjoint mutations on purpose: deleting a block fails
//! only the coverage test, widening a read-only block fails only the scope
//! test.

use std::fs;
use std::path::PathBuf;

/// Workflows allowed to grant more than `contents: read`, with the reason.
/// Everything else must stay read-only.
const WRITE_GRANT_ALLOWLIST: &[(&str, &str)] = &[
    (
        "svccat-pr.yml",
        "posts PR annotations, needs pull-requests: write",
    ),
    (
        "benchmark.yml",
        "pushes benchmark data, needs contents/pull-requests: write",
    ),
];

fn workflows_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".github")
        .join("workflows")
}

/// Every workflow file as `(file name, text)`, with CRLF normalised:
/// `.github/workflows/*.yml` in this repo is CRLF, so LF-anchored matching
/// silently finds nothing.
fn workflows() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for entry in fs::read_dir(workflows_dir()).expect(".github/workflows is readable") {
        let path = entry.expect("readable dir entry").path();
        let is_yaml = path
            .extension()
            .is_some_and(|ext| ext == "yml" || ext == "yaml");
        if !is_yaml {
            continue;
        }
        let name = path
            .file_name()
            .expect("workflow file has a name")
            .to_string_lossy()
            .into_owned();
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
            .replace("\r\n", "\n");
        out.push((name, text));
    }
    out.sort();
    assert!(
        out.len() >= 6,
        "expected at least the six known workflows, found {}: {:?}",
        out.len(),
        out.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>()
    );
    out
}

/// The scopes granted by the workflow-level `permissions:` block (column 0),
/// or `None` when the file has no such block. `Some(vec![])` is a bare
/// `permissions: {}` (grant nothing).
fn top_level_permissions(text: &str) -> Option<Vec<String>> {
    let mut lines = text.lines();
    lines.find(|line| line.starts_with("permissions:"))?;
    let mut scopes = Vec::new();
    for line in lines {
        let is_scope = line.starts_with("  ")
            && !line.starts_with("   ")
            && !line.trim_start().starts_with('#');
        if !is_scope {
            break;
        }
        scopes.push(line.trim().to_string());
    }
    Some(scopes)
}

/// Every top-level job key with its body (same heuristic as
/// `ci_lint_gate_tests.rs`: a job key is a two-space-indented `key:` line
/// after `jobs:`).
fn jobs(text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut in_jobs = false;
    for line in text.lines() {
        if line.starts_with("jobs:") {
            in_jobs = true;
            continue;
        }
        if !in_jobs {
            continue;
        }
        let is_job_key = line.starts_with("  ")
            && !line.starts_with("   ")
            && line.trim_end().ends_with(':')
            && !line.trim_start().starts_with('#');
        if is_job_key {
            out.push((line.trim().trim_end_matches(':').to_string(), String::new()));
        } else if let Some((_, body)) = out.last_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    out
}

/// Every workflow must pin its GITHUB_TOKEN grant in the file itself: either a
/// workflow-level `permissions:` block, or one on every single job. A job with
/// neither runs on the repo-level default, which no commit reviews.
#[test]
fn every_workflow_job_has_an_explicit_github_token_grant() {
    for (name, text) in workflows() {
        if top_level_permissions(&text).is_some() {
            continue;
        }
        let jobs = jobs(&text);
        assert!(
            !jobs.is_empty(),
            "{name}: no workflow-level `permissions:` block and no jobs found — \
             the job-scan heuristic no longer matches this file's layout"
        );
        for (job, body) in jobs {
            assert!(
                body.lines()
                    .any(|line| line.starts_with("    permissions:")),
                "{name}: job `{job}` has no `permissions:` block and the file has \
                 no workflow-level one, so its GITHUB_TOKEN grant is whatever the \
                 repo settings page currently says instead of what this commit \
                 says; add a least-privilege block (see coverage.yml)"
            );
        }
    }
}

/// Outside the named allowlist, no workflow-level grant may exceed
/// `contents: read`. This is what turns a quiet widening (say
/// `packages: write` slipped into publish.yml) into a red PR.
#[test]
fn only_allowlisted_workflows_grant_more_than_contents_read() {
    for (name, text) in workflows() {
        if WRITE_GRANT_ALLOWLIST
            .iter()
            .any(|(allowed, _)| *allowed == name)
        {
            continue;
        }
        let Some(scopes) = top_level_permissions(&text) else {
            // Job-level blocks only: each job's block is held to the same
            // read-only bar.
            for (job, body) in jobs(&text) {
                let job_scopes: Vec<&str> = body
                    .lines()
                    .skip_while(|line| !line.starts_with("    permissions:"))
                    .skip(1)
                    .take_while(|line| line.starts_with("      "))
                    .map(str::trim)
                    .collect();
                for scope in job_scopes {
                    assert_eq!(
                        scope, "contents: read",
                        "{name}: job `{job}` grants `{scope}` — beyond \
                         `contents: read` and not on the allowlist"
                    );
                }
            }
            continue;
        };
        assert_eq!(
            scopes,
            vec!["contents: read".to_string()],
            "{name}: the workflow-level `permissions:` block grants more (or less) \
             than `contents: read` and is not on the allowlist; if the widening is \
             deliberate, add it to WRITE_GRANT_ALLOWLIST with its reason"
        );
    }
}
