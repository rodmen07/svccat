use crate::drift::{DriftReport, Severity};
use anyhow::Result;
use serde_json::{json, Value};

/// Render a `svccat check` result as a Microsoft Teams Incoming Webhook payload.
///
/// The output is an Adaptive Card (version 1.4) wrapped in the `attachments`
/// envelope expected by Teams incoming webhooks and the `chat.postMessage` API.
pub fn render_check(report: &DriftReport) -> Result<()> {
    let errors = report.error_count();
    let warnings = report.warning_count();

    let (title, title_color) = if report.drifts.is_empty() {
        ("svccat: No drift detected".to_string(), "Good")
    } else {
        (
            format!(
                "svccat: {errors} error{}, {warnings} warning{}",
                plural(errors),
                plural(warnings)
            ),
            if errors > 0 { "Attention" } else { "Warning" },
        )
    };

    let mut body: Vec<Value> = vec![
        json!({
            "type": "TextBlock",
            "size": "Large",
            "weight": "Bolder",
            "color": title_color,
            "text": title,
            "wrap": true
        }),
        json!({
            "type": "FactSet",
            "facts": [
                {"title": "Manifest", "value": report.manifest},
                {"title": "Declared", "value": report.declared.to_string()},
                {"title": "Discovered", "value": report.discovered.to_string()},
            ]
        }),
    ];

    if report.drifts.is_empty() {
        body.push(json!({
            "type": "TextBlock",
            "text": "All services are in sync.",
            "color": "Good",
            "wrap": true
        }));
    } else {
        body.push(json!({"type": "Container", "separator": true, "items": []}));
        for item in &report.drifts {
            let icon = match item.severity {
                Severity::Error => "🔴",
                Severity::Warning => "🟡",
            };
            let kind_str = format!("{:?}", item.kind);
            body.push(json!({
                "type": "TextBlock",
                "text": format!("{icon} **{}** - {}\n{}", item.service, kind_str, item.message),
                "wrap": true
            }));
        }
    }

    let card = json!({
        "$schema": "http://adaptivecards.io/schemas/adaptive-card.json",
        "type": "AdaptiveCard",
        "version": "1.4",
        "msteams": {"width": "Full"},
        "body": body
    });

    let payload = json!({
        "type": "message",
        "attachments": [{
            "contentType": "application/vnd.microsoft.card.adaptive",
            "contentUrl": null,
            "content": card
        }]
    });

    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}
