use crate::manifest::Manifest;
use crate::safe_http::{self, SafeHttpError};
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
/// - Every hop of an HTTP redirect chain is re-validated the same way before
///   it is followed, so a validated public URL cannot 302 its way to a
///   private/internal address (see [`crate::safe_http`]).
pub fn ping_services(manifest: &Manifest) -> Vec<PingResult> {
    manifest
        .services
        .iter()
        .filter_map(|svc| svc.url.as_deref().map(|url| (svc, url.to_string())))
        .map(|(svc, url)| {
            let ping = match safe_http::get(&url, false, Duration::from_secs(5)) {
                Ok(resp) => PingStatus::Reachable {
                    code: resp.status(),
                },
                Err(SafeHttpError::Request(e)) => match *e {
                    // 4xx/5xx: got a response, service is up
                    ureq::Error::Status(code, _) => PingStatus::Reachable { code },
                    other => PingStatus::Unreachable {
                        reason: other.to_string(),
                    },
                },
                // Initial URL, or a redirect target, failed SSRF validation.
                Err(SafeHttpError::Blocked(e)) => PingStatus::Invalid {
                    reason: e.to_string(),
                },
                Err(e) => PingStatus::Unreachable {
                    reason: e.to_string(),
                },
            };
            PingResult {
                service: svc.name.clone(),
                url,
                ping,
            }
        })
        .collect()
}
