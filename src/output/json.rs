use crate::drift::DriftReport;
use crate::manifest::Manifest;
use crate::ping::PingResult;
use anyhow::Result;
use serde_json::json;

pub fn render_check(report: &DriftReport, ping_results: &[PingResult]) -> Result<()> {
    println!("{}", render_check_to_string(report, ping_results)?);
    Ok(())
}

pub fn render_check_to_string(report: &DriftReport, ping_results: &[PingResult]) -> Result<String> {
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
    Ok(serde_json::to_string_pretty(&out)?)
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
