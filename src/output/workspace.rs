use crate::workspace::WorkspaceDriftReport;
use anyhow::Result;
use serde_json::json;

/// Render workspace report in terminal format with per-repo sections.
pub fn render_terminal(report: &WorkspaceDriftReport) {
    println!();
    println!("╔═══════════════════════════════════════════════════════════════════╗");
    println!("║              WORKSPACE DRIFT REPORT                              ║");
    println!("╚═══════════════════════════════════════════════════════════════════╝");
    println!();

    for (idx, analysis) in report.repos.iter().enumerate() {
        if idx > 0 {
            println!();
        }

        println!("📦 Repository: {}", analysis.name);
        println!("   Path: {}", analysis.path.display());
        println!(
            "   Services: {} declared, {} discovered",
            analysis.drift.declared, analysis.drift.discovered
        );
        println!(
            "   Issues: {} error(s), {} warning(s)",
            analysis.drift.error_count(),
            analysis.drift.warning_count()
        );

        if !analysis.drift.drifts.is_empty() {
            println!();
            for item in &analysis.drift.drifts {
                let icon = match item.severity {
                    crate::drift::Severity::Error => "x ",
                    crate::drift::Severity::Warning => "!",
                };
                println!("   {} [{}] {}", icon, item.kind.to_string(), item.message);
                if let Some(detail) = &item.detail {
                    println!("      Detail: {}", detail);
                }
            }
        }
    }

    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!(
        "Summary: {} total services, {} errors, {} warnings across {} repositories",
        report.total_declared,
        report.total_errors,
        report.total_warnings,
        report.repos.len()
    );
    println!();

    if report.has_errors() {
        println!("❌ Drift detected. Use --fail-on-drift to gate in CI.");
    } else if report.has_warnings() {
        println!("⚠️  Warnings found. All declared services match repo, but issues detected.");
    } else {
        println!("✅ All services are in sync across workspace.");
    }
    println!();
}

/// Render workspace report in JSON format.
pub fn render_json(report: &WorkspaceDriftReport) -> Result<String> {
    let json_report = json!({
        "total_declared": report.total_declared,
        "total_discovered": report.total_discovered,
        "total_errors": report.total_errors,
        "total_warnings": report.total_warnings,
        "repositories": report.repos.iter().map(|analysis| {
            json!({
                "name": analysis.name,
                "path": analysis.path.display().to_string(),
                "manifest": analysis.drift.manifest,
                "declared": analysis.drift.declared,
                "discovered": analysis.drift.discovered,
                "errors": analysis.drift.error_count(),
                "warnings": analysis.drift.warning_count(),
                "drifts": analysis.drift.drifts.iter().map(|item| {
                    json!({
                        "kind": format!("{:?}", item.kind),
                        "severity": format!("{:?}", item.severity),
                        "service": item.service,
                        "message": item.message,
                        "detail": item.detail,
                    })
                }).collect::<Vec<_>>(),
            })
        }).collect::<Vec<_>>(),
    });

    Ok(serde_json::to_string_pretty(&json_report)?)
}

/// Render workspace report in Markdown format.
pub fn render_markdown(report: &WorkspaceDriftReport) -> String {
    let mut md = String::from("# Workspace Drift Report\n\n");

    md.push_str(&format!(
        "**Summary:** {} total services, {} errors, {} warnings across {} repositories\n\n",
        report.total_declared, report.total_errors, report.total_warnings, report.repos.len()
    ));

    for analysis in &report.repos {
        md.push_str(&format!("## 📦 {}\n\n", analysis.name));
        md.push_str(&format!("- **Path:** `{}`\n", analysis.path.display()));
        md.push_str(&format!(
            "- **Services:** {} declared, {} discovered\n",
            analysis.drift.declared, analysis.drift.discovered
        ));
        md.push_str(&format!(
            "- **Issues:** {} error(s), {} warning(s)\n\n",
            analysis.drift.error_count(),
            analysis.drift.warning_count()
        ));

        if !analysis.drift.drifts.is_empty() {
            md.push_str("### Drift Items\n\n");
            for item in &analysis.drift.drifts {
                md.push_str(&format!(
                    "- **{}** [`{}`] — {} ({})\n",
                    match item.severity {
                        crate::drift::Severity::Error => "❌ Error",
                        crate::drift::Severity::Warning => "⚠️  Warning",
                    },
                    item.kind.to_string(),
                    item.message,
                    item.service
                ));
                if let Some(detail) = &item.detail {
                    md.push_str(&format!("  - Detail: `{}`\n", detail));
                }
            }
            md.push('\n');
        }
    }

    md
}

// Helper trait impl for formatting DriftKind
trait DriftKindDisplay {
    fn to_string(&self) -> String;
}

impl DriftKindDisplay for crate::drift::DriftKind {
    fn to_string(&self) -> String {
        match self {
            crate::drift::DriftKind::DeclaredMissingFromRepo => "MISSING".to_string(),
            crate::drift::DriftKind::UndeclaredInRepo => "UNDECLARED".to_string(),
            crate::drift::DriftKind::MissingField => "FIELD".to_string(),
            crate::drift::DriftKind::MissingReferencedFile => "REF".to_string(),
            crate::drift::DriftKind::PolicyViolation => "POLICY".to_string(),
            crate::drift::DriftKind::DanglingDependency => "DEPENDS".to_string(),
            crate::drift::DriftKind::CircularDependency => "CYCLE".to_string(),
        }
    }
}
