use crate::drift::{DriftKind, DriftReport, Severity};
use crate::ping::{PingResult, PingStatus};
use anyhow::Result;
use serde_json::{json, Value};

/// Emit a SARIF 2.1.0 document to stdout.
///
/// Drift items become SARIF results.  Each `DriftKind` is a separate rule so
/// GitHub Code Scanning can surface them as inline PR annotations.
///
/// `--ping` failures become results too, under their own rules: an unreachable
/// or SSRF-blocked service URL is a finding a CI consumer of this format needs
/// to see, exactly as it is in the `json`, `junit`, `markdown` and terminal
/// renderers.  A *reachable* service produces no result, because SARIF results
/// are problems, not a per-URL health log.
pub fn render_check(report: &DriftReport, ping_results: &[PingResult]) -> Result<()> {
    let doc = build_sarif(report, ping_results);
    println!("{}", serde_json::to_string_pretty(&doc)?);
    Ok(())
}

// ── Internal ──────────────────────────────────────────────────────────────────

fn build_sarif(report: &DriftReport, ping_results: &[PingResult]) -> Value {
    let rules = sarif_rules();
    let mut results: Vec<Value> = report
        .drifts
        .iter()
        .map(|item| sarif_result(item, &report.manifest))
        .collect();

    // Ping results keep the manifest order `ping::ping_services` walked, and
    // are appended after the drift results, so the document is deterministic
    // for a given (report, ping_results) pair.
    results.extend(
        ping_results
            .iter()
            .filter_map(|ping| sarif_ping_result(ping, &report.manifest)),
    );

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
        sarif_rule(
            "unreachable_service",
            "UnreachableService",
            "Service url did not answer when pinged with --ping",
            "error"
        ),
        sarif_rule(
            "invalid_service_url",
            "InvalidServiceUrl",
            "Service url was rejected before it was requested (bad scheme, or a private/internal address)",
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

    sarif_finding(rule_id, level, &item.message, &item.service, manifest_path)
}

/// A ping outcome as a SARIF result, or `None` when there is nothing to report.
///
/// `Reachable` deliberately yields `None`: SARIF `results` are findings, and
/// emitting one per healthy URL would turn a clean run into a wall of Code
/// Scanning alerts.  This mirrors the junit renderer, where a reachable
/// service is a passing testcase and only the other two states are failures.
fn sarif_ping_result(ping: &PingResult, manifest_path: &str) -> Option<Value> {
    let (rule_id, message) = match &ping.ping {
        PingStatus::Reachable { .. } => return None,
        PingStatus::Unreachable { reason } => (
            "unreachable_service",
            format!(
                "{} is unreachable at {}: {}",
                ping.service, ping.url, reason
            ),
        ),
        PingStatus::Invalid { reason } => (
            "invalid_service_url",
            format!(
                "{} has an unusable url {}: {}",
                ping.service, ping.url, reason
            ),
        ),
    };

    Some(sarif_finding(
        rule_id,
        "error",
        &message,
        &ping.service,
        manifest_path,
    ))
}

/// The one shape every result in this document has, so a new finding kind
/// cannot invent a different location layout.
fn sarif_finding(
    rule_id: &str,
    level: &str,
    message: &str,
    service: &str,
    manifest_path: &str,
) -> Value {
    json!({
        "ruleId": rule_id,
        "level": level,
        "message": { "text": message },
        "locations": [{
            "physicalLocation": {
                "artifactLocation": { "uri": uri_from_path(manifest_path) }
            },
            "logicalLocations": [{
                "name": service,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drift::DriftItem;

    // ── Fixtures ────────────────────────────────────────────────────────────

    fn report_with(drifts: Vec<DriftItem>) -> DriftReport {
        report_at("./services.yaml", drifts)
    }

    fn report_at(manifest: &str, drifts: Vec<DriftItem>) -> DriftReport {
        DriftReport {
            manifest: manifest.to_string(),
            declared: drifts.len(),
            discovered: 0,
            drifts,
        }
    }

    fn drift(kind: DriftKind, severity: Severity) -> DriftItem {
        DriftItem {
            kind,
            severity,
            service: "billing".to_string(),
            message: "something drifted".to_string(),
            detail: None,
        }
    }

    fn ping(service: &str, status: PingStatus) -> PingResult {
        PingResult {
            service: service.to_string(),
            url: format!("https://{service}.example.com/health"),
            ping: status,
        }
    }

    /// Every `DriftKind`, listed once.
    ///
    /// The wildcard-free `match` is what keeps this list honest: adding a
    /// variant to `DriftKind` stops this module compiling, so the list can
    /// never silently fall behind the enum it claims to enumerate.
    fn all_drift_kinds() -> Vec<DriftKind> {
        let all = vec![
            DriftKind::DeclaredMissingFromRepo,
            DriftKind::UndeclaredInRepo,
            DriftKind::MissingField,
            DriftKind::MissingReferencedFile,
            DriftKind::PolicyViolation,
            DriftKind::DanglingDependency,
            DriftKind::CircularDependency,
        ];
        for kind in &all {
            match kind {
                DriftKind::DeclaredMissingFromRepo
                | DriftKind::UndeclaredInRepo
                | DriftKind::MissingField
                | DriftKind::MissingReferencedFile
                | DriftKind::PolicyViolation
                | DriftKind::DanglingDependency
                | DriftKind::CircularDependency => {}
            }
        }
        all
    }

    /// Every `PingStatus`, listed once, under the same compile-time guard.
    fn all_ping_statuses() -> Vec<PingStatus> {
        let all = vec![
            PingStatus::Reachable { code: 200 },
            PingStatus::Unreachable {
                reason: "connection refused".to_string(),
            },
            PingStatus::Invalid {
                reason: "URL uses private/internal IP address: 10.0.0.1".to_string(),
            },
        ];
        for status in &all {
            match status {
                PingStatus::Reachable { .. }
                | PingStatus::Unreachable { .. }
                | PingStatus::Invalid { .. } => {}
            }
        }
        all
    }

    fn declared_rule_ids(doc: &Value) -> Vec<String> {
        doc["runs"][0]["tool"]["driver"]["rules"]
            .as_array()
            .expect("rules array")
            .iter()
            .map(|r| r["id"].as_str().expect("rule id").to_string())
            .collect()
    }

    fn result_rule_ids(doc: &Value) -> Vec<String> {
        doc["runs"][0]["results"]
            .as_array()
            .expect("results array")
            .iter()
            .map(|r| r["ruleId"].as_str().expect("ruleId").to_string())
            .collect()
    }

    // ── The document a consumer actually receives ───────────────────────────

    #[test]
    fn document_carries_the_sarif_version_and_tool_driver_a_consumer_keys_off() {
        let doc = build_sarif(&report_with(vec![]), &[]);

        assert_eq!(doc["version"], "2.1.0");
        assert!(doc["$schema"]
            .as_str()
            .expect("$schema")
            .ends_with("sarif-schema-2.1.0.json"));

        let driver = &doc["runs"][0]["tool"]["driver"];
        assert_eq!(driver["name"], "svccat");
        assert_eq!(driver["version"], env!("CARGO_PKG_VERSION"));

        // A clean run is still a well-formed document with both arrays present.
        assert_eq!(doc["runs"][0]["results"].as_array().unwrap().len(), 0);
        assert_eq!(
            doc["runs"][0]["artifacts"][0]["location"]["uri"],
            "services.yaml"
        );
    }

    #[test]
    fn drift_severity_maps_onto_the_sarif_level_enum() {
        let doc = build_sarif(
            &report_with(vec![
                drift(DriftKind::PolicyViolation, Severity::Error),
                drift(DriftKind::MissingField, Severity::Warning),
            ]),
            &[],
        );

        let results = doc["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results[0]["level"], "error");
        assert_eq!(results[1]["level"], "warning");
        // SARIF 2.1.0 only permits none/note/warning/error.
        for r in results {
            let level = r["level"].as_str().unwrap();
            assert!(
                ["none", "note", "warning", "error"].contains(&level),
                "level {level} is not in the SARIF level enum"
            );
        }
    }

    // ── The drift guard: rules and results must agree ───────────────────────

    /// Reads BOTH sources that have to agree: the hand-written `sarif_rules()`
    /// array, and every rule id the two result emitters can actually produce.
    ///
    /// A result whose `ruleId` is not declared in `tool.driver.rules` is the
    /// failure mode this guards: GitHub Code Scanning rejects such a document,
    /// and the two lists are maintained by hand in different functions.
    #[test]
    fn every_rule_id_a_result_can_emit_is_declared_in_the_rules_array() {
        let declared = declared_rule_ids(&build_sarif(&report_with(vec![]), &[]));

        for kind in all_drift_kinds() {
            let doc = build_sarif(
                &report_with(vec![drift(kind.clone(), Severity::Error)]),
                &[],
            );
            let emitted = result_rule_ids(&doc);
            assert_eq!(emitted.len(), 1, "{kind:?} should emit exactly one result");
            assert!(
                declared.contains(&emitted[0]),
                "{kind:?} emits ruleId {:?}, which sarif_rules() does not declare",
                emitted[0]
            );
        }

        for status in all_ping_statuses() {
            let doc = build_sarif(&report_with(vec![]), &[ping("billing", status.clone())]);
            for emitted in result_rule_ids(&doc) {
                assert!(
                    declared.contains(&emitted),
                    "{status:?} emits ruleId {emitted:?}, which sarif_rules() does not declare"
                );
            }
        }
    }

    #[test]
    fn declared_rule_ids_are_unique() {
        let declared = declared_rule_ids(&build_sarif(&report_with(vec![]), &[]));
        let mut sorted = declared.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            declared.len(),
            "duplicate rule id in {declared:?}"
        );
    }

    // ── The defect this coverage found: --ping was dropped on the floor ─────

    #[test]
    fn unreachable_and_invalid_pings_become_results() {
        let doc = build_sarif(
            &report_with(vec![]),
            &[
                ping(
                    "billing",
                    PingStatus::Unreachable {
                        reason: "connection refused".to_string(),
                    },
                ),
                ping(
                    "auth",
                    PingStatus::Invalid {
                        reason: "URL uses private/internal IP address: 10.0.0.1".to_string(),
                    },
                ),
            ],
        );

        assert_eq!(
            result_rule_ids(&doc),
            vec!["unreachable_service", "invalid_service_url"]
        );

        let results = doc["runs"][0]["results"].as_array().unwrap();
        assert_eq!(results[0]["level"], "error");
        assert_eq!(
            results[0]["locations"][0]["logicalLocations"][0]["name"],
            "billing"
        );
        assert!(results[0]["message"]["text"]
            .as_str()
            .unwrap()
            .contains("connection refused"));
        assert_eq!(
            results[1]["locations"][0]["logicalLocations"][0]["name"],
            "auth"
        );
        assert!(results[1]["message"]["text"]
            .as_str()
            .unwrap()
            .contains("10.0.0.1"));
    }

    /// The discriminator that stops the test above from passing for the wrong
    /// reason ("any ping produces a result"): a healthy service must stay out
    /// of `results`, or every clean `--ping` run becomes a wall of alerts.
    #[test]
    fn reachable_pings_produce_no_result() {
        let doc = build_sarif(
            &report_with(vec![]),
            &[
                ping("billing", PingStatus::Reachable { code: 200 }),
                ping("auth", PingStatus::Reachable { code: 503 }),
            ],
        );

        assert!(
            result_rule_ids(&doc).is_empty(),
            "reachable services must not become findings"
        );
    }

    #[test]
    fn ping_results_are_appended_after_drift_results_in_manifest_order() {
        let doc = build_sarif(
            &report_with(vec![drift(DriftKind::MissingField, Severity::Warning)]),
            &[
                ping("billing", PingStatus::Reachable { code: 200 }),
                ping(
                    "auth",
                    PingStatus::Unreachable {
                        reason: "timed out".to_string(),
                    },
                ),
                ping(
                    "search",
                    PingStatus::Invalid {
                        reason: "unsupported URL scheme: ftp".to_string(),
                    },
                ),
            ],
        );

        assert_eq!(
            result_rule_ids(&doc),
            vec![
                "missing_field",
                "unreachable_service",
                "invalid_service_url"
            ]
        );
    }

    /// Compatibility: with no ping results the document is exactly what it was
    /// before ping support existed, so the fix cannot have moved drift output.
    #[test]
    fn a_run_without_ping_is_unchanged_by_ping_support() {
        let report = report_with(vec![
            drift(DriftKind::DeclaredMissingFromRepo, Severity::Error),
            drift(DriftKind::MissingField, Severity::Warning),
        ]);

        assert_eq!(
            result_rule_ids(&build_sarif(&report, &[])),
            vec!["declared_missing_from_repo", "missing_field"]
        );
        assert_eq!(
            build_sarif(&report, &[]),
            build_sarif(
                &report,
                &[ping("billing", PingStatus::Reachable { code: 200 })]
            ),
            "a reachable ping must not perturb the document at all"
        );
    }

    // ── uri_from_path ───────────────────────────────────────────────────────

    #[test]
    fn uri_from_path_strips_the_leading_dot_slash_in_both_separators() {
        assert_eq!(uri_from_path("./services.yaml"), "services.yaml");
        assert_eq!(uri_from_path(".\\services.yaml"), "services.yaml");
        assert_eq!(uri_from_path("services.yaml"), "services.yaml");
    }

    #[test]
    fn uri_from_path_normalises_windows_separators_so_the_uri_is_not_os_dependent() {
        assert_eq!(
            uri_from_path(".\\catalog\\services.yaml"),
            "catalog/services.yaml"
        );
        assert_eq!(
            uri_from_path("catalog\\nested\\services.yaml"),
            "catalog/nested/services.yaml"
        );
    }

    #[test]
    fn the_artifact_uri_and_every_result_location_name_the_same_file() {
        let doc = build_sarif(
            &report_at(
                ".\\catalog\\services.yaml",
                vec![drift(DriftKind::MissingField, Severity::Warning)],
            ),
            &[ping(
                "billing",
                PingStatus::Unreachable {
                    reason: "timed out".to_string(),
                },
            )],
        );

        let artifact = doc["runs"][0]["artifacts"][0]["location"]["uri"]
            .as_str()
            .unwrap()
            .to_string();
        assert_eq!(artifact, "catalog/services.yaml");

        for result in doc["runs"][0]["results"].as_array().unwrap() {
            assert_eq!(
                result["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                    .as_str()
                    .unwrap(),
                artifact
            );
        }
    }
}
