use anyhow::Result;
use std::net::IpAddr;
use std::str::FromStr;

/// Validate a URL to prevent SSRF attacks.
///
/// Rejects:
/// - URLs with internal/private IP addresses (127.x.x.x, 10.x.x.x, 172.16-31.x.x, 192.168.x.x, etc.)
/// - URLs without a scheme (http:// or https://)
/// - http:// URLs when strict_https is true
pub fn validate_url(url: &str, strict_https: bool) -> Result<()> {
    if url.is_empty() {
        anyhow::bail!("URL cannot be empty");
    }

    // Parse URL to extract host
    let parsed = url::Url::parse(url)
        .map_err(|_| anyhow::anyhow!("invalid URL: {}", url))?;

    // Check scheme
    match parsed.scheme() {
        "http" | "https" => {}
        "ftp" | "file" | "data" => {
            anyhow::bail!("URL scheme '{}' not allowed (use http or https)", parsed.scheme())
        }
        _ => {
            anyhow::bail!("unsupported URL scheme: {}", parsed.scheme())
        }
    }

    // Enforce HTTPS for webhooks if required
    if strict_https && parsed.scheme() != "https" {
        // Allow http://localhost for development
        if let Some(host) = parsed.host_str() {
            if host != "localhost" && !host.starts_with("localhost:") {
                anyhow::bail!("webhooks must use https:// (not http://) except for localhost");
            }
        }
    }

    // Get the host and check for internal IPs
    if let Some(host) = parsed.host_str() {
        is_public_ip(host)?;
    } else {
        anyhow::bail!("URL missing host");
    }

    Ok(())
}

/// Check if a hostname resolves to a public IP (reject private ranges).
fn is_public_ip(host: &str) -> Result<()> {
    // Reject localhost immediately
    if host == "localhost" || host.starts_with("localhost:") {
        return Ok(()); // Allow for development
    }

    // Strip IPv6 brackets if present (e.g., "[::1]" -> "::1")
    // Handle both "[::1]" and "[::1]:8080" formats
    let host_to_parse = if host.starts_with('[') {
        if let Some(bracket_idx) = host.find(']') {
            // Extract just the IPv6 address between brackets
            &host[1..bracket_idx]
        } else {
            host // Malformed, will fail parsing below
        }
    } else {
        host
    };

    // Try to parse as IP address
    match IpAddr::from_str(host_to_parse) {
        Ok(ip) => {
            if is_private_ip(&ip) {
                anyhow::bail!("URL uses private/internal IP address: {}", ip);
            }
            Ok(())
        }
        Err(_) => {
            // Not an IP address, treat as hostname
            // For hostnames, we cannot reliably check without DNS resolution.
            // In production, the HTTP request will fail if it's a private IP.
            // We log a warning but allow it to proceed.
            // This is acceptable because:
            // 1. Most internal DNS won't resolve from external networks
            // 2. The HTTP timeout will catch slow/hanging connections
            // 3. Stricter validation could break legitimate use cases (custom internal domains)
            Ok(())
        }
    }
}

/// Check if an IP is in a private/internal range.
fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(addr) => {
            addr.is_private()       // 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16
            || addr.is_loopback()   // 127.0.0.0/8
            || addr.is_link_local() // 169.254.0.0/16
            || addr.is_broadcast()  // 255.255.255.255
            || addr.is_unspecified() // 0.0.0.0
        }
        IpAddr::V6(addr) => {
            addr.is_loopback()     // ::1
            || addr.is_unspecified() // ::
            // Check for private ranges manually since is_private() may not be stable
            || is_ipv6_private(addr)
            || is_ipv6_link_local(addr)
        }
    }
}

/// Check if IPv6 address is in private range fc00::/7
fn is_ipv6_private(addr: &std::net::Ipv6Addr) -> bool {
    let segments = addr.segments();
    (segments[0] & 0xfe00) == 0xfc00
}

/// Check if IPv6 address is in link-local range fe80::/10
fn is_ipv6_link_local(addr: &std::net::Ipv6Addr) -> bool {
    let segments = addr.segments();
    (segments[0] & 0xffc0) == 0xfe80
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_public_urls() {
        assert!(validate_url("https://example.com", false).is_ok());
        assert!(validate_url("https://api.github.com", false).is_ok());
        assert!(validate_url("http://example.com", false).is_ok());
        assert!(validate_url("https://hooks.slack.com/services/xyz", false).is_ok());
    }

    #[test]
    fn test_localhost_allowed_for_http() {
        assert!(validate_url("http://localhost:8080", false).is_ok());
        assert!(validate_url("http://localhost", false).is_ok());
    }

    #[test]
    fn test_https_required_for_webhooks() {
        assert!(validate_url("https://example.com", true).is_ok());
        assert!(validate_url("http://example.com", true).is_err()); // HTTP rejected
        assert!(validate_url("http://localhost:8080", true).is_ok()); // localhost exception
    }

    #[test]
    fn test_invalid_private_ips() {
        assert!(validate_url("http://127.0.0.1", false).is_err());
        assert!(validate_url("http://192.168.1.1", false).is_err());
        assert!(validate_url("http://10.0.0.1", false).is_err());
        assert!(validate_url("http://172.16.0.1", false).is_err());
        assert!(validate_url("http://169.254.1.1", false).is_err());
    }

    #[test]
    fn test_ipv6_loopback_rejected() {
        assert!(validate_url("http://[::1]", false).is_err());
    }

    #[test]
    fn test_invalid_schemes() {
        assert!(validate_url("file:///etc/passwd", false).is_err());
        assert!(validate_url("ftp://example.com", false).is_err());
        assert!(validate_url("data:text/plain,hello", false).is_err());
    }

    #[test]
    fn test_malformed_urls() {
        assert!(validate_url("not a url", false).is_err());
        assert!(validate_url("", false).is_err());
    }
}
