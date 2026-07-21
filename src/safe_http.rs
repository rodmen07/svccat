//! SSRF-safe HTTP requests: redirect-following with per-hop revalidation.
//!
//! `ureq` (the HTTP client this crate uses) follows HTTP redirects
//! automatically, but [`urlvalidation::validate_url`] is only ever called
//! once, against the *initial* destination URL, before the first request
//! goes out. A server that returns a 3xx redirect to a private or internal
//! address — the cloud metadata endpoint `http://169.254.169.254/latest/meta-data/`,
//! an internal service on `127.0.0.1`, or any RFC 1918 address — would have
//! that redirect followed with no re-validation of the new target: a
//! validated, public-looking initial URL becomes an SSRF vector on any later
//! hop, because the client never re-checks where it actually ends up.
//!
//! The functions here close that gap by disabling `ureq`'s built-in
//! redirect following (`AgentBuilder::redirects(0)`, confirmed from
//! `ureq` 2.12.1's source: with `redirects(0)` a 3xx response is returned to
//! the caller instead of being followed) and instead following redirects
//! manually, re-running `validate_url` against every `Location` header
//! target before it is ever requested, bounded by [`MAX_REDIRECT_HOPS`].

use crate::urlvalidation::validate_url;
use std::fmt;
use std::time::Duration;

/// Maximum redirect hops to follow. Matches `ureq`'s own built-in default
/// (`AgentBuilder::redirects` defaults to `5`, see `ureq::agent::AgentBuilder`),
/// so legitimate redirect chains behave exactly as they did before this fix;
/// it exists to bound the manual loop below against redirect loops.
const MAX_REDIRECT_HOPS: u8 = 5;

/// Error from a redirect-validated HTTP request.
#[derive(Debug)]
pub enum SafeHttpError {
    /// The initial URL, or a `Location` redirect target, failed SSRF
    /// validation (see [`validate_url`]).
    Blocked(anyhow::Error),
    /// A redirect chain exceeded [`MAX_REDIRECT_HOPS`] (likely a redirect loop).
    TooManyRedirects { max: u8 },
    /// A `Location` header was missing a value, or the redirect target could
    /// not be resolved to a valid URL.
    BadRedirect(String),
    /// The underlying HTTP request failed: a network/transport error, or a
    /// response with a 4xx/5xx status code (`ureq` surfaces both as
    /// `ureq::Error`). Boxed because `ureq::Error::Status` embeds a full
    /// `Response`, making the unboxed variant much larger than the others.
    Request(Box<ureq::Error>),
}

impl fmt::Display for SafeHttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SafeHttpError::Blocked(e) => write!(f, "blocked by URL validation: {e}"),
            SafeHttpError::TooManyRedirects { max } => {
                write!(f, "too many redirects (max {max}), possible redirect loop")
            }
            SafeHttpError::BadRedirect(msg) => write!(f, "bad redirect: {msg}"),
            SafeHttpError::Request(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for SafeHttpError {}

/// GET `url`, following redirects manually and re-validating every `Location`
/// target against [`validate_url`] before it is ever requested.
///
/// `strict_https` is forwarded to `validate_url` unchanged (see its docs).
pub fn get(
    url: &str,
    strict_https: bool,
    timeout: Duration,
) -> Result<ureq::Response, SafeHttpError> {
    follow(url, strict_https, |agent, u| {
        agent.get(u).timeout(timeout).call().map_err(Box::new)
    })
}

/// POST a JSON `body` to `url`, following redirects manually and
/// re-validating every `Location` target against [`validate_url`] before it
/// is ever requested.
pub fn post_json(
    url: &str,
    strict_https: bool,
    body: &str,
    user_agent: &str,
) -> Result<ureq::Response, SafeHttpError> {
    follow(url, strict_https, |agent, u| {
        agent
            .post(u)
            .set("Content-Type", "application/json")
            .set("User-Agent", user_agent)
            .send_string(body)
            .map_err(Box::new)
    })
}

/// Shared redirect-following loop used by [`get`] and [`post_json`].
///
/// `attempt` performs exactly one HTTP call against the given agent and URL;
/// it is re-invoked with a new, revalidated URL for each redirect hop.
fn follow(
    url: &str,
    strict_https: bool,
    attempt: impl Fn(&ureq::Agent, &str) -> Result<ureq::Response, Box<ureq::Error>>,
) -> Result<ureq::Response, SafeHttpError> {
    validate_url(url, strict_https).map_err(SafeHttpError::Blocked)?;

    // Disable ureq's automatic redirect-following entirely: we re-validate
    // and follow each hop ourselves below.
    let agent = ureq::AgentBuilder::new().redirects(0).build();
    let mut current = url.to_string();

    for hop in 0..=MAX_REDIRECT_HOPS {
        let resp = attempt(&agent, &current).map_err(SafeHttpError::Request)?;

        if !(300..400).contains(&resp.status()) {
            return Ok(resp);
        }
        let Some(location) = resp.header("location").map(str::to_string) else {
            // A 3xx with no Location header has nothing to follow; surface it as-is.
            return Ok(resp);
        };

        if hop == MAX_REDIRECT_HOPS {
            return Err(SafeHttpError::TooManyRedirects {
                max: MAX_REDIRECT_HOPS,
            });
        }

        current = resolve_and_validate_redirect(&current, &location, strict_https)?;
    }

    unreachable!("the loop above always returns before its final iteration completes")
}

/// Resolve a `Location` header value (which may be relative) against the
/// current URL, then run it through the same [`validate_url`] check applied
/// to initial URLs. This is the function that closes the SSRF gap: without
/// it, a redirect target is never checked at all.
fn resolve_and_validate_redirect(
    current: &str,
    location: &str,
    strict_https: bool,
) -> Result<String, SafeHttpError> {
    let base = url::Url::parse(current)
        .map_err(|e| SafeHttpError::BadRedirect(format!("current URL became unparseable: {e}")))?;
    let next = base.join(location).map_err(|e| {
        SafeHttpError::BadRedirect(format!("bad redirect Location '{location}': {e}"))
    })?;
    let next = next.to_string();

    validate_url(&next, strict_https).map_err(SafeHttpError::Blocked)?;
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The core regression proof for this fix, at the unit level: a redirect
    /// target must be rejected by the exact same rule that would reject it
    /// as an initial URL. Before this fix, no such check existed at all for
    /// redirect targets — only the initial URL was ever validated.
    #[test]
    fn redirect_target_rejected_like_an_initial_private_ip_url() {
        let metadata_url = "http://169.254.169.254/latest/meta-data/";

        // Sanity: validate_url already rejects this address as an initial URL
        // (it's link-local, see urlvalidation::is_private_ip).
        assert!(
            validate_url(metadata_url, false).is_err(),
            "sanity check failed: validate_url should reject this as an initial URL too"
        );

        let result =
            resolve_and_validate_redirect("https://looks-benign.example.com/", metadata_url, false);

        match result {
            Err(SafeHttpError::Blocked(_)) => {}
            other => panic!(
                "redirect to the cloud metadata address must be blocked exactly like \
                 an initial URL would be, got: {other:?}"
            ),
        }
    }

    #[test]
    fn redirect_target_rejected_for_rfc1918_private_range() {
        for target in [
            "http://10.0.0.1/",
            "http://172.16.0.1/",
            "http://192.168.1.1/",
            "http://127.0.0.1:6379/",
        ] {
            let result = resolve_and_validate_redirect("https://example.com/", target, false);
            assert!(
                matches!(result, Err(SafeHttpError::Blocked(_))),
                "expected redirect to {target} to be blocked, got: {result:?}"
            );
        }
    }

    #[test]
    fn redirect_target_resolved_relative_to_current_url() {
        let resolved = resolve_and_validate_redirect("https://example.com/a/b", "/c", false)
            .expect("relative redirect to a public path should resolve and validate");
        assert_eq!(resolved, "https://example.com/c");
    }

    #[test]
    fn redirect_to_another_public_host_is_allowed() {
        let resolved = resolve_and_validate_redirect(
            "https://example.com/",
            "https://other-public-host.example.org/landing",
            false,
        )
        .expect("redirect to a different public host should be allowed");
        assert_eq!(resolved, "https://other-public-host.example.org/landing");
    }
}
