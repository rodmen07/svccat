use crate::drift::DriftReport;
use crate::manifest::{Manifest, ServiceEntry};
use crate::policy::{PolicyConfig, PolicyReport, PolicySeverity};
use anyhow::Result;
use colored::Colorize;
use serde::Serialize;
use std::fmt::Write as _;
use std::path::Path;

// ── Score dimensions ──────────────────────────────────────────────────────────

/// Scores for a single service (0-100 each).
#[derive(Debug, Clone, Serialize)]
pub struct ServiceScore {
    pub name: String,
    /// Percentage of optional metadata fields that are populated.
    pub completeness: u8,
    /// 100 when the service has no drift errors, 0 when it has errors.
    pub drift: u8,
    /// 100 when no policy errors; reduced per violation.
    pub policy: u8,
    /// Composite score: weighted average (completeness 40%, drift 40%, policy 20%).
    pub total: u8,
}

/// Catalog-wide scorecard.
#[derive(Debug, Clone, Serialize)]
pub struct Scorecard {
    pub services: Vec<ServiceScore>,
    pub avg_total: u8,
    pub avg_completeness: u8,
    pub avg_drift: u8,
    pub avg_policy: u8,
}

// ── Scored fields ─────────────────────────────────────────────────────────────

// These are the optional fields we track for completeness scoring.
const SCORED_FIELDS: &[&str] = &[
    "language", "platform", "url", "role", "team", "oncall", "docs", "ci", "path",
];

fn completeness_score(svc: &ServiceEntry) -> u8 {
    let filled = SCORED_FIELDS.iter().filter(|&&f| field_set(svc, f)).count();
    ((filled * 100) / SCORED_FIELDS.len()) as u8
}

fn field_set(svc: &ServiceEntry, field: &str) -> bool {
    match field {
        "language" => svc.language.is_some(),
        "platform" => svc.platform.is_some(),
        "url" => svc.url.is_some(),
        "role" => svc.role.is_some(),
        "team" => svc.team.is_some(),
        "oncall" => svc.oncall.is_some(),
        "docs" => svc.docs.is_some(),
        "ci" => svc.ci.is_some(),
        "path" => svc.path.is_some(),
        _ => false,
    }
}

// ── Main scoring logic ────────────────────────────────────────────────────────

/// Build a full scorecard for all services in `manifest`.
///
/// `drift_report` is from `drift::analyze`.
/// `policy_report` is `None` when no policy file is configured.
pub fn build(
    manifest: &Manifest,
    drift_report: &DriftReport,
    policy_report: Option<&PolicyReport>,
) -> Scorecard {
    let mut services: Vec<ServiceScore> = manifest
        .services
        .iter()
        .map(|svc| score_service(svc, drift_report, policy_report))
        .collect();

    // Sort highest score first, then alphabetically.
    services.sort_by(|a, b| b.total.cmp(&a.total).then(a.name.cmp(&b.name)));

    let n = services.len().max(1) as u32;
    let avg = |f: fn(&ServiceScore) -> u8| -> u8 {
        (services.iter().map(|s| f(s) as u32).sum::<u32>() / n) as u8
    };

    Scorecard {
        avg_total: avg(|s| s.total),
        avg_completeness: avg(|s| s.completeness),
        avg_drift: avg(|s| s.drift),
        avg_policy: avg(|s| s.policy),
        services,
    }
}

fn score_service(
    svc: &ServiceEntry,
    drift_report: &DriftReport,
    policy_report: Option<&PolicyReport>,
) -> ServiceScore {
    let completeness = completeness_score(svc);

    // Drift: 100 if no errors for this service, deduct 20 per error (floor 0).
    let drift_errors = drift_report
        .drifts
        .iter()
        .filter(|d| d.service == svc.name && matches!(d.severity, crate::drift::Severity::Error))
        .count();
    let drift = (100u8).saturating_sub((drift_errors * 20).min(100) as u8);

    // Policy: 100 if no policy, else deduct 25 per error violation.
    let policy = if let Some(pr) = policy_report {
        let policy_errors = pr
            .violations
            .iter()
            .filter(|v| v.service == svc.name && matches!(v.severity, PolicySeverity::Error))
            .count();
        (100u8).saturating_sub((policy_errors * 25).min(100) as u8)
    } else {
        100
    };

    // Weighted composite: completeness 40%, drift 40%, policy 20%.
    let total = ((completeness as u32 * 40 + drift as u32 * 40 + policy as u32 * 20) / 100) as u8;

    ServiceScore {
        name: svc.name.clone(),
        completeness,
        drift,
        policy,
        total,
    }
}

// ── Run helper ────────────────────────────────────────────────────────────────

/// Convenience: load manifest, run drift + policy, return a scored scorecard.
pub fn run(manifest: &Manifest, root: &Path, ignore: &[String], depth: u32) -> Scorecard {
    use crate::discovery;
    use crate::drift;

    let discovered = discovery::discover_services_with_opts(root, manifest, ignore, depth);
    let drift_report = drift::analyze(manifest, &discovered, root);

    let policy_cfg = PolicyConfig::load(root);
    let policy_report = policy_cfg
        .as_ref()
        .map(|cfg| crate::policy::check(manifest, cfg));

    build(manifest, &drift_report, policy_report.as_ref())
}

// ── Renderers ─────────────────────────────────────────────────────────────────

fn score_color(score: u8) -> colored::Color {
    if score >= 80 {
        colored::Color::Green
    } else if score >= 50 {
        colored::Color::Yellow
    } else {
        colored::Color::Red
    }
}

fn bar(score: u8, width: usize) -> String {
    let filled = ((score as usize) * width) / 100;
    let empty = width - filled;
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

pub fn render_terminal(sc: &Scorecard) {
    println!("{}", "Service Scorecard".bold());
    println!("{}", "─".repeat(80).dimmed());

    // Header
    println!(
        "{:<28}  {:>6}  {:>6}  {:>6}  {:>6}  {}",
        "SERVICE".bold(),
        "TOTAL".bold(),
        "COMPL".bold(),
        "DRIFT".bold(),
        "POLCY".bold(),
        "SCORE BAR".bold(),
    );
    println!("{}", "─".repeat(80).dimmed());

    for svc in &sc.services {
        let color = score_color(svc.total);
        let bar_str = bar(svc.total, 20);
        println!(
            "{:<28}  {:>6}  {:>6}  {:>6}  {:>6}  {}",
            svc.name,
            format!("{:>5}%", svc.total).color(color),
            format!("{:>5}%", svc.completeness),
            format!("{:>5}%", svc.drift),
            format!("{:>5}%", svc.policy),
            bar_str.color(color),
        );
    }

    println!("{}", "─".repeat(80).dimmed());
    let avg_color = score_color(sc.avg_total);
    println!(
        "{:<28}  {:>6}  {:>6}  {:>6}  {:>6}",
        "AVERAGE".bold(),
        format!("{:>5}%", sc.avg_total).color(avg_color).bold(),
        format!("{:>5}%", sc.avg_completeness),
        format!("{:>5}%", sc.avg_drift),
        format!("{:>5}%", sc.avg_policy),
    );
}

pub fn render_json(sc: &Scorecard) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(sc)?);
    Ok(())
}

pub fn render_json_to_string(sc: &Scorecard) -> Result<String> {
    Ok(serde_json::to_string_pretty(sc)?)
}

pub fn render_markdown(sc: &Scorecard) -> String {
    let mut out = String::new();
    writeln!(out, "# Service Scorecard").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "| Service | Total | Completeness | Drift | Policy |").unwrap();
    writeln!(out, "|---------|------:|-------------:|------:|-------:|").unwrap();
    for svc in &sc.services {
        writeln!(
            out,
            "| {} | {}% | {}% | {}% | {}% |",
            svc.name, svc.total, svc.completeness, svc.drift, svc.policy
        )
        .unwrap();
    }
    writeln!(out).unwrap();
    writeln!(
        out,
        "**Average** - Total: {}% | Completeness: {}% | Drift: {}% | Policy: {}%",
        sc.avg_total, sc.avg_completeness, sc.avg_drift, sc.avg_policy
    )
    .unwrap();
    out
}
