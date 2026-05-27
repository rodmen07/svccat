use crate::ci::CiReport;
use crate::drift::DriftReport;
use crate::urlvalidation;
use anyhow::{Context, Result};
use serde::Serialize;
use std::path::Path;

// ── Config ─────────────────────────────────────────────────────────────────────

/// Webhook configuration loaded from `svccat.toml` under `[webhook]`.
///
/// Example:
/// ```toml
/// [webhook]
/// url = "https://hooks.slack.com/services/T000/B000/xxxx"
/// on_errors = true
/// on_warnings = false
/// ```
#[derive(Debug, Default, serde::Deserialize, Clone)]
pub struct WebhookConfig {
    /// HTTP endpoint to POST payloads to.
    pub url: Option<String>,
    /// Fire the webhook when there are errors (default: true).
    #[serde(default = "default_true")]
    pub on_errors: bool,
    /// Fire the webhook when there are only warnings (default: false).
    #[serde(default)]
    pub on_warnings: bool,
}

fn default_true() -> bool {
    true
}

// ── Payload ────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    DriftDetected,
    CiFailed,
    Clean,
}

#[derive(Debug, Serialize)]
pub struct WebhookPayload {
    pub kind: EventKind,
    pub repository: String,
    pub errors: usize,
    pub warnings: usize,
    pub summary: String,
    pub timestamp: u64,
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn repo_name(root: &Path) -> String {
    root.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| ".".to_string())
}

// ── Fire helpers ───────────────────────────────────────────────────────────────

/// Fire a webhook from a drift check result.
///
/// Returns `Ok(())` when no webhook is configured, the threshold is not met,
/// or the POST succeeds.  Returns an error only when a URL is configured and
/// the HTTP request fails.
pub fn fire_from_drift(root: &Path, cfg: &WebhookConfig, report: &DriftReport) -> Result<()> {
    let url = match &cfg.url {
        Some(u) => u.clone(),
        None => return Ok(()),
    };

    let errors = report.error_count();
    let warnings = report.warning_count();

    let should_fire =
        (errors > 0 && cfg.on_errors) || (errors == 0 && warnings > 0 && cfg.on_warnings);
    if !should_fire {
        return Ok(());
    }

    let kind = if errors > 0 {
        EventKind::DriftDetected
    } else {
        EventKind::Clean
    };

    let summary = if errors > 0 {
        format!(
            "{} drift error(s), {} warning(s) detected",
            errors, warnings
        )
    } else if warnings > 0 {
        format!("No errors, {} warning(s) detected", warnings)
    } else {
        "No drift detected".to_string()
    };

    let payload = WebhookPayload {
        kind,
        repository: repo_name(root),
        errors,
        warnings,
        summary,
        timestamp: now_secs(),
    };

    post(&url, &payload)
}

/// Fire a webhook from a CI run result.
pub fn fire_from_ci(root: &Path, cfg: &WebhookConfig, report: &CiReport) -> Result<()> {
    let url = match &cfg.url {
        Some(u) => u.clone(),
        None => return Ok(()),
    };

    let errors = report.total_errors();
    let warnings = report.total_warnings();

    let should_fire =
        (errors > 0 && cfg.on_errors) || (errors == 0 && warnings > 0 && cfg.on_warnings);
    if !should_fire {
        return Ok(());
    }

    let kind = if errors > 0 {
        EventKind::CiFailed
    } else {
        EventKind::Clean
    };

    let summary = if errors > 0 {
        format!(
            "CI failed: {} error(s) across {} step(s)",
            errors,
            report.steps_run.len()
        )
    } else {
        format!(
            "CI passed: {} step(s) completed with {} warning(s)",
            report.steps_run.len(),
            warnings
        )
    };

    let payload = WebhookPayload {
        kind,
        repository: repo_name(root),
        errors,
        warnings,
        summary,
        timestamp: now_secs(),
    };

    post(&url, &payload)
}

// ── HTTP POST ──────────────────────────────────────────────────────────────────

fn post(url: &str, payload: &WebhookPayload) -> Result<()> {
    // Validate webhook URL to prevent SSRF attacks.
    // Webhooks should use HTTPS (with localhost exception for development).
    urlvalidation::validate_url(url, true).context("webhook URL validation failed")?;

    let body = serde_json::to_string(payload).context("serialising webhook payload")?;
    ureq::post(url)
        .set("Content-Type", "application/json")
        .set("User-Agent", concat!("svccat/", env!("CARGO_PKG_VERSION")))
        .send_string(&body)
        .with_context(|| format!("webhook POST to {url} failed"))?;
    eprintln!("webhook fired -> {url}");
    Ok(())
}

// ── Config loading ─────────────────────────────────────────────────────────────

/// Maximum config file size (1 MB) to prevent resource exhaustion
const MAX_CONFIG_FILE_SIZE: u64 = 1024 * 1024;

/// Load webhook config from `svccat.toml`.
/// Returns a default (disabled) config if the key is absent.
///
/// # Security
/// Enforces maximum file size to prevent TOML bomb attacks
pub fn load_config(root: &Path) -> WebhookConfig {
    let path = root.join("svccat.toml");
    if !path.exists() {
        return WebhookConfig::default();
    }

    // Check file size to prevent TOML bomb attacks
    if let Ok(metadata) = std::fs::metadata(&path) {
        if metadata.len() > MAX_CONFIG_FILE_SIZE {
            eprintln!(
                "warning: svccat.toml is too large ({}  bytes, max {}). Ignoring config file.",
                metadata.len(),
                MAX_CONFIG_FILE_SIZE
            );
            return WebhookConfig::default();
        }
    }

    #[derive(serde::Deserialize, Default)]
    struct Outer {
        #[serde(default)]
        webhook: WebhookConfig,
    }

    std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| toml::from_str::<Outer>(&text).ok())
        .map(|o| o.webhook)
        .unwrap_or_default()
}
