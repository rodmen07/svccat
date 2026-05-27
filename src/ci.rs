use crate::{discovery, drift, lint, manifest, policy};
use anyhow::Result;
use colored::Colorize;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

// ── Report ────────────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct CiReport {
    pub lint_errors: usize,
    pub lint_warnings: usize,
    pub drift_errors: usize,
    pub drift_warnings: usize,
    pub policy_errors: usize,
    pub policy_warnings: usize,
    pub steps_run: Vec<String>,
}

impl CiReport {
    pub fn total_errors(&self) -> usize {
        self.lint_errors + self.drift_errors + self.policy_errors
    }

    pub fn total_warnings(&self) -> usize {
        self.lint_warnings + self.drift_warnings + self.policy_warnings
    }

    pub fn passed(&self) -> bool {
        self.total_errors() == 0
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Run lint, drift check, and policy check (if configured) in sequence.
///
/// Steps are always lint -> drift -> policy.  Policy is skipped when no
/// `.svccat/policy.yaml` exists.
pub fn run(manifest: &manifest::Manifest, root: &Path, ignore: &[String], depth: u32) -> CiReport {
    let mut report = CiReport::default();

    // Lint
    let lint_result = lint::run(manifest);
    report.lint_errors = lint_result.error_count();
    report.lint_warnings = lint_result.warning_count();
    report.steps_run.push("lint".to_string());

    // Drift
    let discovered = discovery::discover_services_with_opts(root, manifest, ignore, depth);
    let drift_report = drift::analyze(manifest, &discovered, root);
    report.drift_errors = drift_report.error_count();
    report.drift_warnings = drift_report.warning_count();
    report.steps_run.push("drift".to_string());

    // Policy (optional - skip silently when unconfigured)
    if let Some(policy_cfg) = policy::PolicyConfig::load(root) {
        if !policy_cfg.is_empty() {
            let policy_report = policy::check(manifest, &policy_cfg);
            report.policy_errors = policy_report
                .violations
                .iter()
                .filter(|v| matches!(v.severity, policy::PolicySeverity::Error))
                .count();
            report.policy_warnings = policy_report
                .violations
                .iter()
                .filter(|v| matches!(v.severity, policy::PolicySeverity::Warning))
                .count();
            report.steps_run.push("policy".to_string());
        }
    }

    report
}

// ── Renderers ─────────────────────────────────────────────────────────────────

pub fn render_terminal(report: &CiReport) {
    println!("{}", "svccat ci".bold().underline());
    println!("  steps: {}", report.steps_run.join(" -> "));
    println!();

    let lint_icon = if report.lint_errors > 0 {
        "FAIL".red()
    } else {
        "pass".green()
    };
    let drift_icon = if report.drift_errors > 0 {
        "FAIL".red()
    } else {
        "pass".green()
    };
    println!(
        "  {:<8} {}  ({} errors, {} warnings)",
        "lint", lint_icon, report.lint_errors, report.lint_warnings
    );
    println!(
        "  {:<8} {}  ({} errors, {} warnings)",
        "drift", drift_icon, report.drift_errors, report.drift_warnings
    );

    if report.steps_run.contains(&"policy".to_string()) {
        let policy_icon = if report.policy_errors > 0 {
            "FAIL".red()
        } else {
            "pass".green()
        };
        println!(
            "  {:<8} {}  ({} errors, {} warnings)",
            "policy", policy_icon, report.policy_errors, report.policy_warnings
        );
    }

    println!();
    if report.passed() {
        println!("  {} all checks passed", "✓".green().bold());
    } else {
        println!(
            "  {} {} error(s), {} warning(s) total",
            "✗".red().bold(),
            report.total_errors(),
            report.total_warnings()
        );
    }
}

pub fn render_json(report: &CiReport) -> Result<()> {
    let j = serde_json::json!({
        "passed": report.passed(),
        "steps": report.steps_run,
        "lint":   { "errors": report.lint_errors,   "warnings": report.lint_warnings   },
        "drift":  { "errors": report.drift_errors,  "warnings": report.drift_warnings  },
        "policy": { "errors": report.policy_errors, "warnings": report.policy_warnings },
        "total_errors":   report.total_errors(),
        "total_warnings": report.total_warnings(),
    });
    println!("{}", serde_json::to_string_pretty(&j)?);
    Ok(())
}

// ── Watch mode ────────────────────────────────────────────────────────────────

/// Run `ci::run` continuously, re-triggering on file-system changes.
///
/// Watches the manifest file and every service discovery path.  On each
/// detected change (debounced to 500 ms) it reloads the manifest and reruns
/// the full CI pass, printing a timestamped report.
///
/// Returns the initial error count so callers can honour exit-code semantics
/// on the first run.
pub fn watch(
    manifest_path: &Path,
    root: &Path,
    ignore: &[String],
    depth: u32,
    interval: Option<u64>,
) -> Result<usize> {
    let manifest_path = manifest_path.to_path_buf();
    let root = root.to_path_buf();
    let ignore = ignore.to_vec();

    let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();
    let mut watcher = RecommendedWatcher::new(tx.clone(), Config::default())?;
    watcher.watch(&manifest_path, RecursiveMode::NonRecursive)?;

    // Pre-load manifest to register discovery paths.
    let initial_m = manifest::Manifest::load(&manifest_path)?;
    let watch_paths = effective_watch_paths(&initial_m, &root);
    for p in &watch_paths {
        if p.exists() {
            let _ = watcher.watch(p, RecursiveMode::Recursive);
        }
    }

    // Optional polling thread.
    if let Some(secs) = interval {
        let tx2 = tx.clone();
        let path_clone = manifest_path.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(secs));
            use notify::{event::ModifyKind, Event};
            let ev = Event {
                kind: EventKind::Modify(ModifyKind::Any),
                paths: vec![path_clone.clone()],
                attrs: Default::default(),
            };
            if tx2.send(Ok(ev)).is_err() {
                break;
            }
        });
    }

    // First run immediately.
    let initial_errors = run_once(&manifest_path, &root, &ignore, depth);

    let interval_note = interval
        .map(|s| format!(" (polling every {s}s)"))
        .unwrap_or_default();
    eprintln!(
        "\n{} watching for changes{interval_note}. Press Ctrl-C to stop.",
        "svccat ci --watch".bold()
    );

    let mut last_event = Instant::now();
    for res in rx {
        if let Ok(event) = res {
            if matches!(
                event.kind,
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
            ) {
                if last_event.elapsed() < Duration::from_millis(500) {
                    continue;
                }
                last_event = Instant::now();
                let ts = chrono_lite_now();
                eprintln!("\n[{ts}] change detected - re-running ci…");
                run_once(&manifest_path, &root, &ignore, depth);
            }
        }
    }

    Ok(initial_errors)
}

fn run_once(manifest_path: &PathBuf, root: &Path, ignore: &[String], depth: u32) -> usize {
    match manifest::Manifest::load(manifest_path) {
        Err(e) => {
            eprintln!("{} {e:#}", "error:".red().bold());
            1
        }
        Ok(m) => {
            let report = run(&m, root, ignore, depth);
            render_terminal(&report);
            report.total_errors()
        }
    }
}

fn effective_watch_paths(m: &manifest::Manifest, root: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = m
        .effective_discovery_paths()
        .iter()
        .map(|p| root.join(p))
        .collect();
    paths.push(root.join(".svccat"));
    paths
}

fn chrono_lite_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let s = secs % 60;
    let m = (secs / 60) % 60;
    let h = (secs / 3600) % 24;
    format!("{h:02}:{m:02}:{s:02} UTC")
}
