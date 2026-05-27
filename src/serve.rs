use crate::{discovery, drift, manifest, report};
use anyhow::{Context, Result};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::time::Duration;

/// Start a local HTTP server on `127.0.0.1:{port}` that serves the live
/// HTML drift report.  Each GET request regenerates the report from disk so
/// the page always reflects the current state of the repository.
///
/// When `refresh_secs > 0` a `<meta http-equiv="refresh">` tag is injected
/// so the browser polls automatically.
pub fn serve(root: &Path, port: u16, refresh_secs: u32) -> Result<()> {
    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&addr).with_context(|| format!("failed to bind to {addr}"))?;

    let url = format!("http://localhost:{port}");
    println!("svccat serving at {url}");
    println!("Press Ctrl-C to stop.");

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(_) => continue,
        };

        // Set a short read timeout so we never block indefinitely on a slow client.
        let _ = stream.set_read_timeout(Some(Duration::from_millis(300)));

        // Consume the HTTP request; we only serve one response regardless of path/method.
        let mut req_buf = [0u8; 4096];
        let _ = stream.read(&mut req_buf);

        let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));

        let body = match build_html(root, refresh_secs) {
            Ok(h) => h,
            Err(e) => format!(
                "<html><head><title>svccat error</title></head><body><pre>Error: {e:#}</pre></body></html>"
            ),
        };

        let body_bytes = body.as_bytes();
        let headers = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body_bytes.len()
        );
        let _ = stream.write_all(headers.as_bytes());
        let _ = stream.write_all(body_bytes);
    }

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn build_html(root: &Path, refresh_secs: u32) -> Result<String> {
    let manifest_path = manifest::find_default(root);
    let m = manifest::Manifest::load(&manifest_path)?;
    let discovered = discovery::discover_services_with_ignore(root, &m, &[]);
    let mut r = drift::analyze(&m, &discovered, root);
    r.manifest = manifest_path.display().to_string();

    let mut html = report::render_html(&m, &r);

    if refresh_secs > 0 {
        let meta = format!(r#"<meta http-equiv="refresh" content="{refresh_secs}">"#);
        // Inject after the opening <head> tag.
        html = html.replacen("<head>", &format!("<head>{meta}"), 1);
    }

    Ok(html)
}
