use crate::drift::{DriftItem, DriftKind, DriftReport, Severity};
use crate::ping::{PingResult, PingStatus};
use colored::Colorize;
use std::collections::HashSet;

pub fn render_check(report: &DriftReport, ping_results: &[PingResult]) {
    let errors = report.error_count();
    let warnings = report.warning_count();

    println!(
        "{}",
        format!(
            "svccat: {} declared, {} discovered  [{}]",
            report.declared, report.discovered, report.manifest
        )
        .bold()
    );
    println!();

    if report.drifts.is_empty() {
        println!("{}", "  OK  No drift detected".green().bold());
    } else {
        println!(
            "{}",
            format!(
                "  DRIFT DETECTED  ({} error{}, {} warning{})",
                errors,
                if errors == 1 { "" } else { "s" },
                warnings,
                if warnings == 1 { "" } else { "s" }
            )
            .yellow()
            .bold()
        );
        println!();

        for item in &report.drifts {
            print_drift_item(item);
        }

        println!();

        if errors > 0 {
            println!("{}", format!("  x  {} error(s)", errors).red().bold());
        }
        if warnings > 0 {
            println!("{}", format!("  !  {} warning(s)", warnings).yellow());
        }
    }

    if !ping_results.is_empty() {
        println!();
        render_ping_results(ping_results);
    }
}

/// Renders only the drift that is new or resolved compared to `old_report`.
/// Returns `(new_count, resolved_count)`.
pub fn render_since_diff(
    old_report: &DriftReport,
    new_report: &DriftReport,
    git_ref: &str,
) -> (usize, usize) {
    let old_keys: HashSet<String> = old_report.drifts.iter().map(drift_key).collect();
    let new_keys: HashSet<String> = new_report.drifts.iter().map(drift_key).collect();

    let added: Vec<&DriftItem> = new_report
        .drifts
        .iter()
        .filter(|d| !old_keys.contains(&drift_key(d)))
        .collect();
    let resolved: Vec<&DriftItem> = old_report
        .drifts
        .iter()
        .filter(|d| !new_keys.contains(&drift_key(d)))
        .collect();
    let unchanged = new_report.drifts.len().saturating_sub(added.len());

    println!(
        "{}",
        format!("svccat --since {}  [{}]", git_ref, new_report.manifest).bold()
    );
    println!();

    if added.is_empty() && resolved.is_empty() {
        println!(
            "{}",
            format!(
                "  OK  No change in drift since {}  ({} existing item{})",
                git_ref,
                unchanged,
                if unchanged == 1 { "" } else { "s" }
            )
            .green()
            .bold()
        );
        return (0, 0);
    }

    if !added.is_empty() {
        println!(
            "{}",
            format!("  NEW drift since {} ({}):", git_ref, added.len())
                .yellow()
                .bold()
        );
        println!();
        for item in &added {
            print_drift_item(item);
        }
        println!();
    }

    if !resolved.is_empty() {
        println!(
            "{}",
            format!("  RESOLVED since {} ({}):", git_ref, resolved.len())
                .green()
                .bold()
        );
        println!();
        for item in &resolved {
            print_resolved_drift_item(item);
        }
        println!();
    }

    if unchanged > 0 {
        println!(
            "  {} existing drift item{} unchanged",
            unchanged,
            if unchanged == 1 { "" } else { "s" }
        );
    }

    (added.len(), resolved.len())
}

pub fn render_ping_results(results: &[PingResult]) {
    println!("{}", "  PING RESULTS".bold());
    println!();
    for r in results {
        match &r.ping {
            PingStatus::Reachable { code } if *code < 400 => {
                println!(
                    "  {}  {}  {} ({})",
                    "✓".green().bold(),
                    "[PING]".green(),
                    r.service,
                    code
                );
            }
            PingStatus::Reachable { code } => {
                println!(
                    "  {}  {}  {}  {} ({})",
                    "!".yellow().bold(),
                    "[PING]".yellow(),
                    r.service,
                    r.url,
                    code
                );
            }
            PingStatus::Unreachable { reason } => {
                println!(
                    "  {}  {}  {}  {} — {}",
                    "x".red().bold(),
                    "[PING]".red(),
                    r.service,
                    r.url,
                    reason
                );
            }
        }
    }
}

fn print_drift_item(item: &DriftItem) {
    let icon = match item.severity {
        Severity::Error => "x".red().bold(),
        Severity::Warning => "!".yellow().bold(),
    };

    let kind_label = drift_kind_label(&item.kind);
    println!("  {}  {}  {}", icon, kind_label, item.message);
}

fn print_resolved_drift_item(item: &DriftItem) {
    let kind_label = drift_kind_label(&item.kind);
    println!("  {}  {}  {}", "✓".green().bold(), kind_label, item.message);
}

fn drift_kind_label(kind: &DriftKind) -> colored::ColoredString {
    match kind {
        DriftKind::DeclaredMissingFromRepo => "[MISSING]   ".red(),
        DriftKind::UndeclaredInRepo => "[UNDECLARED]".yellow(),
        DriftKind::MissingField => "[FIELD]     ".cyan(),
        DriftKind::MissingReferencedFile => "[REF]       ".yellow(),
        DriftKind::PolicyViolation => "[POLICY]    ".red(),
        DriftKind::DanglingDependency => "[DEPENDS]   ".red(),
        DriftKind::CircularDependency => "[CYCLE]     ".red(),
    }
}

/// Stable identity key for a drift item (used by --since diffing).
fn drift_key(item: &DriftItem) -> String {
    format!(
        "{:?}|{}|{}",
        item.kind,
        item.service,
        item.detail.as_deref().unwrap_or("")
    )
}
