use crate::manifest::Manifest;
use crate::urlvalidation;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "status")]
pub enum PingStatus {
    /// Got an HTTP response (any status code — the service is reachable).
    Reachable { code: u16 },
    /// Connection failed (timeout, DNS error, TLS error, etc.).
    Unreachable { reason: String },
    /// URL failed validation (e.g., private IP address).
    Invalid { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResult {
    pub service: String,
    pub url: String,
    pub ping: PingStatus,
}

impl PingResult {
    pub fn is_ok(&self) -> bool {
        matches!(self.ping, PingStatus::Reachable { .. })
    }
}

/// Ping every service that has a `url` field and return the results.
///
/// URLs are validated to prevent SSRF attacks:
/// - Private/internal IP addresses are rejected
/// - URLs must have http:// or https:// scheme
pub fn ping_services(manifest: &Manifest) -> Vec<PingResult> {
    manifest
        .services
        .iter()
        .filter_map(|svc| svc.url.as_deref().map(|url| (svc, url.to_string())))
        .map(|(svc, url)| {
            // Validate URL to prevent SSRF attacks (allow http for this context)
            let ping = if let Err(e) = urlvalidation::validate_url(&url, false) {
                PingStatus::Invalid {
                    reason: e.to_string(),
                }
            } else {
                match ureq::get(&url).timeout(Duration::from_secs(5)).call() {
                    Ok(resp) => PingStatus::Reachable {
                        code: resp.status(),
                    },
                    // 4xx/5xx: got a response, service is up
                    Err(ureq::Error::Status(code, _)) => PingStatus::Reachable { code },
                    Err(e) => PingStatus::Unreachable {
                        reason: e.to_string(),
                    },
                }
            };
            PingResult {
                service: svc.name.clone(),
                url,
                ping,
            }
        })
        .collect()
}
