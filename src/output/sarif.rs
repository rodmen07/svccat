use crate::drift::{DriftKind, DriftReport, Severity};
use crate::ping::PingResult;
use anyhow::Result;
use serde_json::{json, Value};

/// Emit a SARIF 2.1.0 document to stdout.
///
/// Drift items become SARIF results.  Each `DriftKind` is a separate rule so
/// GitHub Code Scanning can surface them as inline PR annotations.
pub fn render_check(report: &DriftReport, _ping_results: &[PingResult]) -> Result<()> {
    let doc = build_sarif(report);
    println!("{}", serde_json::to_string_pretty(&doc)?);
    Ok(())
}

// ── Internal ──────────────────────────────────────────────────────────────────

fn build_sarif(report: &DriftReport) -> Value {
    let rules = sarif_rules();
    let results: Vec<Value> = report
        .drifts
        .iter()
        .map(|item| sarif_result(item, &report.manifest))
        .collect();

    json!({
        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "svccat",
                    "version": env!("CARGO_PKG_VERSION"),
                    "informationUri": "https://github.com/rodmen07/svccat",
                    "rules": rules
                }
            },
            "results": results,
            "artifacts": [{
                "location": { "uri": uri_from_path(&report.manifest) },
                "roles": ["analysisTarget"]
            }]
        }]
    })
}

fn sarif_rules() -> Value {
    json!([
        sarif_rule(
            "declared_missing_from_repo",
            "DeclaredMissingFromRepo",
            "Service declared in manifest but directory not found in repo",
            "error"
        ),
        sarif_rule(
            "undeclared_in_repo",
            "UndeclaredInRepo",
            "Service directory discovered in repo but absent from manifest",
            "warning"
        ),
        sarif_rule(
            "missing_field",
            "MissingField",
            "Recommended metadata field absent from service entry",
            "warning"
        ),
        sarif_rule(
            "missing_referenced_file",
            "MissingReferencedFile",
            "File referenced by docs or ci field does not exist",
            "warning"
        ),
        sarif_rule(
            "policy_violation",
            "PolicyViolation",
            "Service is missing a field required by policy.require_fields",
            "error"
        ),
        sarif_rule(
            "dangling_dependency",
            "DanglingDependency",
            "depends_on references a service not declared in the manifest",
            "error"
        ),
        sarif_rule(
            "circular_dependency",
            "CircularDependency",
            "Circular dependency detected in depends_on graph",
            "error"
        ),
    ])
}

fn sarif_rule(id: &str, name: &str, description: &str, default_level: &str) -> Value {
    json!({
        "id": id,
        "name": name,
        "shortDescription": { "text": description },
        "defaultConfiguration": { "level": default_level },
        "helpUri": format!("https://github.com/rodmen07/svccat#{}", id)
    })
}

fn sarif_result(item: &crate::drift::DriftItem, manifest_path: &str) -> Value {
    let rule_id = match item.kind {
        DriftKind::DeclaredMissingFromRepo => "declared_missing_from_repo",
        DriftKind::UndeclaredInRepo => "undeclared_in_repo",
        DriftKind::MissingField => "missing_field",
        DriftKind::MissingReferencedFile => "missing_referenced_file",
        DriftKind::PolicyViolation => "policy_violation",
        DriftKind::DanglingDependency => "dangling_dependency",
        DriftKind::CircularDependency => "circular_dependency",
    };
    let level = match item.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
    };

    json!({
        "ruleId": rule_id,
        "level": level,
        "message": { "text": item.message },
        "locations": [{
            "physicalLocation": {
                "artifactLocation": { "uri": uri_from_path(manifest_path) }
            },
            "logicalLocations": [{
                "name": item.service,
                "kind": "function"
            }]
        }]
    })
}

/// Convert a file-system path to a URI suitable for SARIF artifact locations.
/// Strips a leading "./" for cleaner output; leaves absolute paths as-is.
fn uri_from_path(path: &str) -> String {
    let stripped = path.trim_start_matches("./").trim_start_matches(".\\");
    stripped.replace('\\', "/")
}
