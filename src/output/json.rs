use crate::drift::DriftReport;
use crate::manifest::Manifest;
use crate::ping::PingResult;
use anyhow::Result;
use serde_json::json;

pub fn render_check(report: &DriftReport, ping_results: &[PingResult]) -> Result<()> {
    let out = json!({
        "manifest": report.manifest,
        "summary": {
            "declared": report.declared,
            "discovered": report.discovered,
            "drift_count": report.drifts.len(),
            "errors": report.error_count(),
            "warnings": report.warning_count(),
        },
        "drift": report.drifts,
        "ping": ping_results,
    });
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}

pub fn render_export(manifest: &Manifest, report: &DriftReport) -> Result<()> {
    let out = json!({
        "version": manifest.version,
        "manifest": report.manifest,
        "summary": {
            "declared": report.declared,
            "discovered": report.discovered,
            "drift_count": report.drifts.len(),
            "errors": report.error_count(),
            "warnings": report.warning_count(),
        },
        "services": manifest.services,
        "drift": report.drifts,
    });
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}
