use crate::drift::{DriftItem, DriftKind, DriftReport, Severity};
use colored::Colorize;

pub fn render_check(report: &DriftReport) {
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
        return;
    }

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
    };

    println!("  {}  {}  {}", icon, kind_label, item.message);
}
