use crate::drift::{DriftItem, DriftKind, DriftReport, Severity};
use crate::ping::{PingResult, PingStatus};
use colored::Colorize;

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

    let kind_label = match item.kind {
        DriftKind::DeclaredMissingFromRepo => "[MISSING]   ".red(),
        DriftKind::UndeclaredInRepo => "[UNDECLARED]".yellow(),
        DriftKind::MissingField => "[FIELD]     ".cyan(),
        DriftKind::MissingReferencedFile => "[REF]       ".yellow(),
        DriftKind::PolicyViolation => "[POLICY]    ".red(),
    };

    println!("  {}  {}  {}", icon, kind_label, item.message);
}
