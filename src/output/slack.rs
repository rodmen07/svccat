use crate::drift::{DriftReport, Severity};
use anyhow::Result;
use serde_json::{json, Value};

/// Render a `svccat check` result as Slack Block Kit JSON.
///
/// The output is a complete Slack `blocks` payload suitable for posting via
/// the Slack API (`chat.postMessage`) or an incoming webhook.
pub fn render_check(report: &DriftReport) -> Result<()> {
    let payload = build_check_payload(report);
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn build_check_payload(report: &DriftReport) -> Value {
    let errors = report.error_count();
    let warnings = report.warning_count();

    let header_text = if report.drifts.is_empty() {
        "svccat: No drift detected".to_string()
    } else {
        format!(
            "svccat: {errors} error{}, {warnings} warning{}",
            plural(errors),
            plural(warnings)
        )
    };

    let mut blocks: Vec<Value> = vec![
        json!({
            "type": "header",
            "text": {"type": "plain_text", "text": header_text, "emoji": true}
        }),
        json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": format!(
                    "Manifest: `{}`\nDeclared: *{}*  |  Discovered: *{}*",
                    report.manifest, report.declared, report.discovered
                )
            }
        }),
    ];

    if report.drifts.is_empty() {
        blocks.push(json!({
            "type": "section",
            "text": {"type": "mrkdwn", "text": ":white_check_mark: All services are in sync."}
        }));
    } else {
        blocks.push(json!({"type": "divider"}));
        for item in &report.drifts {
            let emoji = match item.severity {
                Severity::Error => ":x:",
                Severity::Warning => ":warning:",
            };
            let kind_str = format!("{:?}", item.kind);
            let text = format!(
                "{emoji} *{}* - {}\n{}",
                item.service, kind_str, item.message
            );
            blocks.push(json!({
                "type": "section",
                "text": {"type": "mrkdwn", "text": text}
            }));
        }
    }

    json!({"blocks": blocks})
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::drift::{DriftItem, DriftKind};

    #[test]
    fn clean_payload_contains_success_text() {
        let report = DriftReport {
            manifest: "services.yaml".to_string(),
            declared: 1,
            discovered: 1,
            drifts: vec![],
        };

        let payload = build_check_payload(&report);
        let blocks = payload["blocks"].as_array().unwrap();
        assert!(!blocks.is_empty());
        assert!(blocks
            .iter()
            .any(|b| b.to_string().contains("No drift detected")));
    }

    #[test]
    fn drift_payload_contains_error_and_warning_items() {
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
                    kind: DriftKind::UndeclaredInRepo,
                    severity: Severity::Warning,
                    service: "worker".to_string(),
                    message: "extra".to_string(),
                    detail: None,
                },
            ],
        };

        let payload = build_check_payload(&report);
        let text = payload.to_string();
        assert!(text.contains("api"));
        assert!(text.contains("worker"));
        assert!(text.contains("error"));
        assert!(text.contains("warning"));
    }
}
