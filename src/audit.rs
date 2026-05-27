use crate::{cost, discovery, drift, lint, manifest, ping};
use anyhow::Result;
use colored::Colorize;
use std::path::Path;

// ── Result type ───────────────────────────────────────────────────────────────

pub struct AuditResult {
    pub manifest_path: String,
    pub lint_errors: usize,
    pub lint_warnings: usize,
    pub drift_errors: usize,
    pub drift_warnings: usize,
    pub ping_failures: usize,
    pub score: u32,
    pub passed: bool,
    pub cost: Option<cost::CostBreakdown>,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Run a full audit: lint + drift + optional ping + optional cost analysis.
///
/// Returns the structured result for the caller to render.  Does not print
/// anything itself - use `render_terminal` or `render_json` after this call.
pub fn run(
    manifest_path: &Path,
    root: &Path,
    ignore: &[String],
    depth: u32,
    do_ping: bool,
    do_cost_estimate: bool,
) -> Result<(
    AuditResult,
    lint::LintResult,
    drift::DriftReport,
    Vec<ping::PingResult>,
)> {
    let m = manifest::Manifest::load(manifest_path)?;

    let lint_result = lint::run(&m);
    let lint_errors = lint_result.error_count();
    let lint_warnings = lint_result.warning_count();

    let discovered = discovery::discover_services_with_opts(root, &m, ignore, depth);
    let mut report = drift::analyze(&m, &discovered, root);
    report.manifest = manifest_path.display().to_string();
    let drift_errors = report.error_count();
    let drift_warnings = report.warning_count();

    let ping_results = if do_ping {
        ping::ping_services(&m)
    } else {
        vec![]
    };
    let ping_failures = ping_results.iter().filter(|p| !p.is_ok()).count();

    let cost_breakdown = if do_cost_estimate {
        Some(cost::analyze(&m))
    } else {
        None
    };

    let deductions = (drift_errors * 10)
        + (drift_warnings * 3)
        + (lint_errors * 5)
        + (lint_warnings * 2)
        + (ping_failures * 5);
    let score = 100u32.saturating_sub(deductions as u32);
    let passed = drift_errors == 0 && lint_errors == 0;

    let result = AuditResult {
        manifest_path: manifest_path.display().to_string(),
        lint_errors,
        lint_warnings,
        drift_errors,
        drift_warnings,
        ping_failures,
        score,
        passed,
        cost: cost_breakdown,
    };

    Ok((result, lint_result, report, ping_results))
}

pub fn render_terminal(
    result: &AuditResult,
    lint_result: &lint::LintResult,
    drift_report: &drift::DriftReport,
    ping_results: &[ping::PingResult],
) {
    let total_lint = result.lint_errors + result.lint_warnings;
    let total_drift = result.drift_errors + result.drift_warnings;

    println!(
        "{}",
        format!("svccat audit  [{}]", result.manifest_path).bold()
    );
    println!();

    // Lint section
    let lint_header = if total_lint == 0 {
        format!("Lint  {}", "OK".green().bold())
    } else {
        format!(
            "Lint  {} error{}, {} warning{}",
            result.lint_errors,
            plural(result.lint_errors),
            result.lint_warnings,
            plural(result.lint_warnings)
        )
    };
    println!("{}", lint_header.bold());
    for issue in &lint_result.issues {
        let (icon, msg) = match issue.severity {
            lint::LintSeverity::Error => ("✗".red().bold(), issue.message.as_str().red()),
            lint::LintSeverity::Warning => ("⚠".yellow(), issue.message.as_str().yellow()),
        };
        println!("  {icon}  {msg}");
    }
    if lint_result.issues.is_empty() {
        println!("  {} No issues found", "✓".green().bold());
    }
    println!();

    // Drift section
    let drift_header = if total_drift == 0 {
        format!("Drift  {}", "OK".green().bold())
    } else {
        format!(
            "Drift  {} error{}, {} warning{}",
            result.drift_errors,
            plural(result.drift_errors),
            result.drift_warnings,
            plural(result.drift_warnings)
        )
    };
    println!("{}", drift_header.bold());
    for item in &drift_report.drifts {
        let (icon, line) = match item.severity {
            drift::Severity::Error => (
                "✗".red().bold(),
                format!("{}: {:?} - {}", item.service, item.kind, item.message).red(),
            ),
            drift::Severity::Warning => (
                "⚠".yellow(),
                format!("{}: {:?} - {}", item.service, item.kind, item.message).yellow(),
            ),
        };
        println!("  {icon}  {line}");
    }
    if drift_report.drifts.is_empty() {
        println!("  {} No drift detected", "✓".green().bold());
    }
    println!();

    // Ping section (only when requested)
    if !ping_results.is_empty() {
        let reachable = ping_results.iter().filter(|p| p.is_ok()).count();
        let unreachable = ping_results.len() - reachable;
        let ping_header = if unreachable == 0 {
            format!(
                "Ping  {} ({}/{} reachable)",
                "OK".green().bold(),
                reachable,
                ping_results.len()
            )
        } else {
            format!("Ping  {}/{} reachable", reachable, ping_results.len())
        };
        println!("{}", ping_header.bold());
        for p in ping_results {
            if !p.is_ok() {
                println!(
                    "  {}  {}  {}  {}",
                    "✗".red().bold(),
                    p.service.as_str().red(),
                    p.url.as_str(),
                    "UNREACHABLE".red()
                );
            }
        }
        if unreachable == 0 {
            println!("  {} All services reachable", "✓".green().bold());
        }
        println!();
    }

    // Cost section (if available)
    if let Some(ref cost) = result.cost {
        println!("{}", "Cost".bold());
        println!("  Estimated monthly: ${:.2}", cost.total_monthly);
        if !cost.by_platform.is_empty() {
            println!("  By platform:");
            let mut platforms: Vec<_> = cost.by_platform.iter().collect();
            platforms.sort_by_key(|(_, &cost)| (cost as i32).wrapping_neg()); // Sort descending by cost
            for (platform, &platform_cost) in platforms {
                println!("    {}: ${:.2}", platform, platform_cost);
            }
        }
        println!();
    }

    // Score bar
    let bar_width = 20usize;
    let filled = (result.score as usize * bar_width) / 100;
    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(bar_width - filled));
    let score_label = format!("Score: {}/100  {}", result.score, bar);
    let verdict = if result.passed {
        "PASS".green().bold()
    } else {
        "FAIL".red().bold()
    };
    println!("  {}  {verdict}", score_label.bold());

    if !result.passed {
        println!();
        if result.drift_errors > 0 {
            println!(
                "  Tip: run {} to auto-remediate drift.",
                "`svccat fix`".cyan()
            );
        }
    }
}

pub fn render_json(result: &AuditResult) -> anyhow::Result<()> {
    let mut payload = serde_json::json!({
        "manifest": result.manifest_path,
        "lint_errors": result.lint_errors,
        "lint_warnings": result.lint_warnings,
        "drift_errors": result.drift_errors,
        "drift_warnings": result.drift_warnings,
        "ping_failures": result.ping_failures,
        "score": result.score,
        "passed": result.passed,
    });

    if let Some(ref cost) = result.cost {
        let by_platform: std::collections::BTreeMap<_, _> = cost.by_platform.iter().collect();
        payload["cost"] = serde_json::json!({
            "estimated_monthly_usd": cost.total_monthly,
            "by_platform": by_platform,
            "services_count": cost.services_count,
        });
    }

    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}
