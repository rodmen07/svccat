use crate::drift::{DriftReport, Severity};
use anyhow::Result;
use serde_json::{json, Value};

/// Render a `svccat check` result as Slack Block Kit JSON.
///
/// The output is a complete Slack `blocks` payload suitable for posting via
/// the Slack API (`chat.postMessage`) or an incoming webhook.
pub fn render_check(report: &DriftReport) -> Result<()> {
    let errors = report.error_count();
    let warnings = report.warning_count();

    let header_text = if report.drifts.is_empty() {
        format!("svccat: No drift detected")
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

    let payload = json!({"blocks": blocks});
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}
