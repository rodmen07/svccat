//! PR-time drift guard: every GitHub Action this repo runs must be on a
//! release that declares the Node 24 runtime.
//!
//! GitHub deprecated the Node 20 runtime for Actions. The failure mode this
//! guards is not a red build — it is the opposite. Before this guard, all six
//! workflows pinned Node 20 actions and every one of the 24 jobs on `main`
//! carried the annotation *"Node.js 20 is deprecated. The following actions
//! target Node.js 20 but are being forced to run on Node.js 24"*, i.e. the
//! runner was silently papering over the gap by running the actions on a
//! runtime they do not declare. Everything stayed green, so nothing in CI
//! could ever fail on it, and a yaml-grep in a backlog note had already
//! MISSED one of the offenders (`Swatinem/rust-cache`, flagged on 20 of the
//! 24 jobs) while listing two actions the runner never flagged at all.
//!
//! The floors below are read from each action's own `action.yml` at that
//! exact tag (`runs.using`), not from release prose — for two of them the
//! obvious guess is wrong: `actions/upload-artifact@v5` still declares
//! `node20`, and `Swatinem/rust-cache@v2.8.0` does too, so bumping either to
//! "the next version" would have looked like a fix and changed nothing.
//!
//! Scope, stated because it is the limit of what a static guard can prove:
//! this asserts what the workflows DECLARE. That an action actually stops
//! being flagged is a runtime fact, and it is verified by reading the
//! Node-20 deprecation annotations of a real run
//! (`repos/rodmen07/svccat/check-runs/<job>/annotations`), not here.
//!
//! The two tests fail on disjoint mutations on purpose: downgrading a pin
//! fails only the floor test, while misspelling a table key fails only the
//! coverage test (a misspelled key matches no step, so the floor test would
//! pass vacuously for that action — which is exactly how this class of guard
//! goes quietly blind).

use std::fs;
use std::path::{Path, PathBuf};

use serde_yaml::Value;

/// The first release of an action that declares `runs.using: node24`.
///
/// Every row was read from that tag's own `action.yml` on 2026-08-10 via
/// `repos/<action>/contents/action.yml?ref=<tag>`, and the `evidence` field
/// records the neighbouring version that proves the floor is where it is
/// rather than one release lower.
struct Floor {
    /// The `uses:` path, without the `@version` suffix.
    action: &'static str,
    first_node24: (u64, u64, u64),
    evidence: &'static str,
}

const NODE24_FLOOR: &[Floor] = &[
    Floor {
        action: "actions/checkout",
        first_node24: (5, 0, 0),
        evidence: "v5.0.0 action.yml declares `using: node24`; v4.1.7 declares node20",
    },
    Floor {
        action: "actions/cache/restore",
        first_node24: (5, 0, 0),
        evidence: "actions/cache v5.0.0 restore/action.yml declares `using: 'node24'`",
    },
    Floor {
        action: "actions/cache/save",
        first_node24: (5, 0, 0),
        evidence: "actions/cache v5.0.0 save/action.yml declares `using: 'node24'`",
    },
    Floor {
        action: "actions/upload-artifact",
        first_node24: (6, 0, 0),
        evidence: "v5.0.0 STILL declares `using: 'node20'` despite its release notes \
                   mentioning Node 24; v6.0.0 is the first release declaring node24",
    },
    Floor {
        action: "actions/github-script",
        first_node24: (8, 0, 0),
        evidence: "v8.0.0 action.yml declares `using: node24`; v7.0.1 declares node20",
    },
    Floor {
        action: "Swatinem/rust-cache",
        first_node24: (2, 9, 0),
        evidence: "v2.8.0 still declares `using: \"node20\"`; v2.9.0 is the first \
                   release declaring node24",
    },
    Floor {
        action: "codecov/codecov-action",
        first_node24: (6, 0, 0),
        evidence: "v4 is a node20 JavaScript action; v5+ are composite wrappers and \
                   v6.0.0 is the release that introduced Node 24 support",
    },
];

/// Every workflow file, GLOB-discovered rather than hand-listed, with a
/// zero-match hard failure so a moved directory cannot make the whole guard
/// pass by finding nothing.
fn workflow_files() -> Vec<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".github")
        .join("workflows");
    let pattern = format!("{}/*.y*ml", dir.display().to_string().replace('\\', "/"));
    let mut files: Vec<PathBuf> = glob::glob(&pattern)
        .expect("workflow glob pattern compiles")
        .map(|entry| entry.expect("readable glob entry"))
        .collect();
    files.sort();
    assert!(
        !files.is_empty(),
        "glob `{pattern}` matched no workflow files; every assertion below would \
         pass vacuously"
    );
    files
}

/// Every `uses:` value in one workflow, as `(job name, uses value)`.
///
/// Read through a REAL YAML parse (`serde_yaml_ng`), never a text scan: these
/// workflows carry long prose comments that already name
/// `actions/cache/restore` and `actions/cache/save` in running text, so a
/// regex over the file body cannot tell a step from a sentence about a step.
/// A parsed document has no comments in it at all.
fn uses_entries(path: &Path) -> Vec<(String, String)> {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    let doc: Value = serde_yaml::from_str(&text)
        .unwrap_or_else(|e| panic!("{} is not parseable YAML: {e}", path.display()));

    let mut out = Vec::new();
    let Some(jobs) = doc.get("jobs").and_then(Value::as_mapping) else {
        panic!("{} declares no `jobs:` mapping", path.display());
    };
    for (job_name, job) in jobs {
        let job_name = job_name
            .as_str()
            .unwrap_or("<non-string job key>")
            .to_string();
        // A job may `uses:` a reusable workflow directly instead of having steps.
        if let Some(uses) = job.get("uses").and_then(Value::as_str) {
            out.push((job_name.clone(), uses.to_string()));
        }
        let Some(steps) = job.get("steps").and_then(Value::as_sequence) else {
            continue;
        };
        for step in steps {
            if let Some(uses) = step.get("uses").and_then(Value::as_str) {
                out.push((job_name.clone(), uses.to_string()));
            }
        }
    }
    out
}

/// `v7`, `v7.0.1` and `v2.9.0` -> a comparable triple. A floating major (`v7`)
/// becomes `(7, 0, 0)`, the lowest release it can resolve to, so the check
/// stays conservative. Anything that is not a `v`-prefixed numeric version
/// (a branch, a commit sha) returns `None` and is reported rather than
/// silently skipped.
fn parse_version(reference: &str) -> Option<(u64, u64, u64)> {
    let rest = reference.strip_prefix('v')?;
    let mut parts = rest.split('.');
    let major: u64 = parts.next()?.parse().ok()?;
    let minor: u64 = match parts.next() {
        Some(part) => part.parse().ok()?,
        None => 0,
    };
    let patch: u64 = match parts.next() {
        Some(part) => part.parse().ok()?,
        None => 0,
    };
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

#[test]
fn every_action_runs_on_the_node_24_runtime() {
    let files = workflow_files();
    let mut total_uses = 0usize;
    let mut checked = 0usize;
    let mut violations = Vec::new();

    for path in &files {
        let file = path
            .file_name()
            .expect("workflow has a name")
            .to_string_lossy();
        for (job, uses) in uses_entries(path) {
            total_uses += 1;
            let Some((action, reference)) = uses.split_once('@') else {
                continue; // a local `./path` or docker action; no version to judge
            };
            let Some(floor) = NODE24_FLOOR.iter().find(|f| f.action == action) else {
                continue; // not a known Node-20 family
            };
            checked += 1;
            let (fm, fn_, fp) = floor.first_node24;
            match parse_version(reference) {
                Some(version) if version >= floor.first_node24 => {}
                Some((m, n, p)) => violations.push(format!(
                    "{file}: job `{job}` pins `{uses}` (v{m}.{n}.{p}), below the \
                     Node 24 floor v{fm}.{fn_}.{fp} — {}",
                    floor.evidence
                )),
                None => violations.push(format!(
                    "{file}: job `{job}` pins `{uses}`, whose ref is not a `v`-numbered \
                     version, so this guard cannot prove it is at or above the Node 24 \
                     floor v{fm}.{fn_}.{fp}. Record the resolved version in NODE24_FLOOR \
                     alongside the pin."
                )),
            }
        }
    }

    assert!(
        total_uses > 0,
        "parsed {} workflow file(s) and found no `uses:` steps at all — the parse \
         walked the wrong shape and this guard is inert",
        files.len()
    );
    assert!(
        checked > 0,
        "found {total_uses} `uses:` steps but none matched a NODE24_FLOOR row, so \
         nothing was actually checked"
    );
    assert!(
        violations.is_empty(),
        "{} action pin(s) still target the deprecated Node 20 runtime:\n  {}",
        violations.len(),
        violations.join("\n  ")
    );
}

#[test]
fn every_node24_floor_row_matches_a_real_step() {
    let files = workflow_files();
    let all_uses: Vec<String> = files
        .iter()
        .flat_map(|path| uses_entries(path))
        .map(|(_, uses)| uses)
        .collect();
    assert!(
        !all_uses.is_empty(),
        "no `uses:` steps parsed out of {} workflow file(s)",
        files.len()
    );

    let unused: Vec<&str> = NODE24_FLOOR
        .iter()
        .map(|floor| floor.action)
        .filter(|action| {
            !all_uses
                .iter()
                .any(|uses| uses.split_once('@').is_some_and(|(a, _)| a == *action))
        })
        .collect();

    assert!(
        unused.is_empty(),
        "NODE24_FLOOR names {} action(s) that no workflow step uses: {unused:?}. \
         Either the workflows dropped them (delete the row) or the key is \
         misspelled — a key that matches nothing makes the floor check pass \
         vacuously for that action, which is this guard going blind rather than \
         going green.",
        unused.len()
    );
}
