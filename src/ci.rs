use crate::{discovery, drift, lint, manifest, policy};
use anyhow::Result;
use colored::Colorize;
use std::path::Path;

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
pub fn run(
    manifest: &manifest::Manifest,
    root: &Path,
    ignore: &[String],
    depth: u32,
) -> CiReport {
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

    let lint_icon = if report.lint_errors > 0 { "FAIL".red() } else { "pass".green() };
    let drift_icon = if report.drift_errors > 0 { "FAIL".red() } else { "pass".green() };
    println!(
        "  {:<8} {}  ({} errors, {} warnings)",
        "lint", lint_icon, report.lint_errors, report.lint_warnings
    );
    println!(
        "  {:<8} {}  ({} errors, {} warnings)",
        "drift", drift_icon, report.drift_errors, report.drift_warnings
    );

    if report.steps_run.contains(&"policy".to_string()) {
        let policy_icon = if report.policy_errors > 0 { "FAIL".red() } else { "pass".green() };
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
