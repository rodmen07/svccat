use crate::drift::{DriftReport, Severity};
use anyhow::Result;
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

/// Render a `svccat check` result as Datadog Events API JSON.
///
/// The output is a JSON array of event objects compatible with the Datadog
/// Events API v1 (`POST /api/v1/events`).  Each drifting service becomes one
/// event; a clean run produces a single "all clear" event.
///
/// Typical pipeline usage:
/// ```text
/// svccat check --format datadog | curl -s -X POST \
///   "https://api.datadoghq.com/api/v1/events?api_key=$DD_API_KEY" \
///   -H "Content-Type: application/json" -d @-
/// ```
pub fn render_check(report: &DriftReport) -> Result<()> {
    let summary = build_check_payload(report);
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}

fn build_check_payload(report: &DriftReport) -> Value {
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let errors = report.error_count();
    let warnings = report.warning_count();

    let events: Vec<Value> = if report.drifts.is_empty() {
        vec![json!({
            "title": "svccat: No drift detected",
            "text": format!(
                "%%% \nAll {} declared service{} are in sync with the repo.\n%%%",
                report.declared,
                if report.declared == 1 { "" } else { "s" }
            ),
            "date_happened": now_secs,
            "alert_type": "success",
            "source_type_name": "svccat",
            "tags": ["svccat", "drift:clean"]
        })]
    } else {
        // One event per drifting service.
        let mut grouped: std::collections::BTreeMap<String, Vec<&crate::drift::DriftItem>> =
            std::collections::BTreeMap::new();
        for item in &report.drifts {
            grouped.entry(item.service.clone()).or_default().push(item);
        }

        grouped
            .into_iter()
            .map(|(service, items)| {
                let has_error = items.iter().any(|i| matches!(i.severity, Severity::Error));
                let alert_type = if has_error { "error" } else { "warning" };

                let lines: Vec<String> = items
                    .iter()
                    .map(|i| {
                        let prefix = match i.severity {
                            Severity::Error => "[ERROR]",
                            Severity::Warning => "[WARN]",
                        };
                        format!("{prefix} {:?}: {}", i.kind, i.message)
                    })
                    .collect();

                let body = format!("%%% \n{}\n%%%", lines.join("\n"));

                json!({
                    "title": format!("svccat drift: {service}"),
                    "text": body,
                    "date_happened": now_secs,
                    "alert_type": alert_type,
                    "source_type_name": "svccat",
                    "tags": [
                        "svccat",
                        format!("service:{service}"),
                        format!("drift:{}", if has_error { "error" } else { "warning" })
                    ]
                })
            })
            .collect()
    };

    json!({
        "manifest": report.manifest,
        "declared": report.declared,
        "discovered": report.discovered,
        "errors": errors,
        "warnings": warnings,
        "events": events
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drift::{DriftItem, DriftKind};

    #[test]
    fn clean_payload_emits_single_success_event() {
        let report = DriftReport {
            manifest: "services.yaml".to_string(),
            declared: 1,
            discovered: 1,
            drifts: vec![],
        };

        let payload = build_check_payload(&report);
        let events = payload["events"].as_array().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["alert_type"], "success");
    }

    #[test]
    fn drift_payload_groups_events_by_service() {
        let report = DriftReport {
            manifest: "services.yaml".to_string(),
            declared: 2,
            discovered: 2,
            drifts: vec![
                DriftItem {
                    kind: DriftKind::DeclaredMissingFromRepo,
                    severity: Severity::Error,
                    service: "api".to_string(),
                    message: "missing".to_string(),
                    detail: None,
                },
                DriftItem {
                    kind: DriftKind::MissingField,
                    severity: Severity::Warning,
                    service: "api".to_string(),
                    message: "no platform".to_string(),
                    detail: Some("platform".to_string()),
                },
                DriftItem {
                    kind: DriftKind::UndeclaredInRepo,
                    severity: Severity::Warning,
                    service: "worker".to_string(),
                    message: "extra".to_string(),
                    detail: None,
                },
            ],
        };

        let payload = build_check_payload(&report);
        let events = payload["events"].as_array().unwrap();
        assert_eq!(events.len(), 2);

        let text = payload.to_string();
        assert!(text.contains("service:api"));
        assert!(text.contains("service:worker"));
    }
}
