//! Proves the SSRF-via-redirect fix in `svccat::safe_http`.
//!
//! Before this fix, `svccat::ping::ping_services` and `svccat::webhook::post`
//! each called `urlvalidation::validate_url` exactly once, against the
//! initial destination URL, then handed the request to `ureq`, which follows
//! HTTP redirects automatically without re-validating the target host on
//! each hop. A server that responded to an initially-valid request with a
//! redirect to a private/internal address (e.g. the cloud metadata endpoint
//! `169.254.169.254`, or a service on `127.0.0.1`) would have that redirect
//! followed with no re-check.
//!
//! These tests spin up real local HTTP servers (hand-rolled `TcpListener`
//! loops, the same style already used by `src/serve.rs`) to prove, against
//! actual HTTP traffic rather than just unit-level assertions, that:
//! 1. a redirect to a private IP literal is refused, and the forbidden
//!    target never receives a connection at all, and
//! 2. a redirect to another `validate_url`-allowed target (localhost) is
//!    still followed correctly, so the fix does not break legitimate
//!    redirect chains.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

/// Spawn a one-shot local HTTP server: it accepts at most one connection
/// within a bounded deadline, sends back `response` verbatim, and reports
/// (via the returned `AtomicBool`) whether a connection was ever received.
/// Bounding the accept loop with a deadline (rather than a plain blocking
/// `accept()`) means a test whose fix works correctly — and therefore never
/// dials this server — cannot hang indefinitely; the thread simply exits
/// once the deadline passes.
///
/// Listens on both the IPv4 (`127.0.0.1`) and, best-effort, the IPv6 (`::1`)
/// loopback at the same port. `validate_url` only exempts the literal
/// hostname `localhost` (not IP literals), and some environments resolve
/// `localhost` to `[::1]` before `127.0.0.1` and then hang rather than
/// instantly refuse a connection attempt to an address nothing is listening
/// on, which would make an IPv4-only test server slow or flaky depending on
/// DNS/OS resolution order. Listening on both removes that dependency.
fn spawn_one_shot_server(response: String) -> (u16, Arc<AtomicBool>, thread::JoinHandle<()>) {
    let v4 = TcpListener::bind("127.0.0.1:0").expect("bind test server (IPv4 loopback)");
    v4.set_nonblocking(true)
        .expect("set IPv4 test server nonblocking");
    let port = v4.local_addr().expect("IPv4 test server local_addr").port();

    let v6 = TcpListener::bind(format!("[::1]:{port}")).ok();
    if let Some(l) = &v6 {
        l.set_nonblocking(true)
            .expect("set IPv6 test server nonblocking");
    }

    let hit = Arc::new(AtomicBool::new(false));
    let hit_for_thread = Arc::clone(&hit);

    let handle = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            for listener in std::iter::once(&v4).chain(v6.as_ref()) {
                if let Ok((mut stream, _)) = listener.accept() {
                    hit_for_thread.store(true, Ordering::SeqCst);
                    let mut buf = [0u8; 4096];
                    let _ = stream.read(&mut buf); // consume the request, ignore its content
                    let _ = stream.write_all(response.as_bytes());
                    return;
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
    });

    (port, hit, handle)
}

/// The attack scenario from the finding: an initially-valid, reachable URL
/// (`http://localhost:{port}/`, allowed by `validate_url`'s development
/// exception) responds with a 302 redirect to a private IP literal
/// (`127.0.0.1` on a different port — the same class of address as the
/// cloud metadata endpoint `169.254.169.254` or an internal service on
/// `localhost:6379`, but using an address `validate_url` already rejects for
/// *initial* URLs, so the test is self-contained without a real attacker
/// server). The fix must refuse to follow it, and the forbidden target must
/// never see a connection at all.
#[test]
fn redirect_to_private_ip_literal_is_never_followed() {
    let (guard_port, guard_hit, guard_handle) = spawn_one_shot_server(
        "HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string(),
    );

    let forbidden_location = format!("http://127.0.0.1:{guard_port}/");
    let (redirect_port, _redirect_hit, redirect_handle) = spawn_one_shot_server(format!(
        "HTTP/1.1 302 Found\r\nLocation: {forbidden_location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    ));

    let initial_url = format!("http://localhost:{redirect_port}/");
    let result = svccat::safe_http::get(&initial_url, false, Duration::from_secs(5));

    redirect_handle
        .join()
        .expect("redirect server thread panicked");
    guard_handle.join().expect("guard server thread panicked");

    let err = result
        .expect_err("a redirect to a private IP literal must be refused, not followed silently");
    let msg = err.to_string();
    assert!(
        msg.to_lowercase().contains("private") || msg.to_lowercase().contains("internal"),
        "expected a private/internal-IP validation error, got: {msg}"
    );
    assert!(
        !guard_hit.load(Ordering::SeqCst),
        "the redirect target must never receive a connection at all: an SSRF-safe \
         client rejects it before dialing, it does not dial and then discard the response"
    );
}

/// Companion positive-path test: a redirect to another target `validate_url`
/// already allows (localhost) must still be followed and the final response
/// returned, proving the fix only closes the SSRF gap and does not also
/// break ordinary redirect chains.
#[test]
fn legitimate_localhost_redirect_chain_still_succeeds() {
    let (dest_port, dest_hit, dest_handle) = spawn_one_shot_server(
        "HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok".to_string(),
    );

    let allowed_location = format!("http://localhost:{dest_port}/");
    let (redirect_port, _redirect_hit, redirect_handle) = spawn_one_shot_server(format!(
        "HTTP/1.1 302 Found\r\nLocation: {allowed_location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    ));

    let initial_url = format!("http://localhost:{redirect_port}/");
    let result = svccat::safe_http::get(&initial_url, false, Duration::from_secs(5));

    redirect_handle
        .join()
        .expect("redirect server thread panicked");
    dest_handle
        .join()
        .expect("destination server thread panicked");

    let resp =
        result.expect("a redirect to another validate_url-allowed target must still be followed");
    assert_eq!(resp.status(), 200);
    assert!(
        dest_hit.load(Ordering::SeqCst),
        "the destination server should have received the followed redirect"
    );
}
