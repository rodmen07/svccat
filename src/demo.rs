//! `svccat demo` - a zero-setup walkthrough.
//!
//! Writes a small sample monorepo (with deliberate drift) to a temporary
//! directory and runs `check`, `graph`, and `stats` against it, so a new user
//! can see svccat in action immediately after `cargo install svccat`.

use anyhow::Result;
use colored::Colorize;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{discovery, drift, manifest, output, stats};

/// Sample manifest. `legacy-worker` has no directory (declared-missing error)
/// and `frontend` omits `platform` (field warning).
const SERVICES_YAML: &str = r#"version: "1"

discovery:
  paths:
    - "services/*"
  markers:
    - Cargo.toml
    - Dockerfile
    - go.mod
    - package.json
    - pyproject.toml

services:
  - name: api-gateway
    language: Go
    platform: Cloud Run
    role: Rate-limiting reverse proxy
    url: https://gateway.example.com
    depends_on:
      - auth-service
  - name: auth-service
    language: Python
    platform: Cloud Run
    role: JWT issuance
    url: https://auth.example.com
  - name: frontend
    language: TypeScript
    role: Single-page application
    url: https://example.github.io/app/
  - name: legacy-worker
    language: Rust
    platform: Fly.io
    role: Background jobs
"#;

/// Marker files that make directories look like real services. `experimental-api`
/// is intentionally not declared in the manifest (undeclared-in-repo warning).
const MARKERS: &[(&str, &str)] = &[
    (
        "services/api-gateway/go.mod",
        "module example.com/api-gateway\n",
    ),
    (
        "services/auth-service/pyproject.toml",
        "[project]\nname = \"auth-service\"\n",
    ),
    (
        "services/frontend/package.json",
        "{ \"name\": \"frontend\" }\n",
    ),
    ("services/experimental-api/Dockerfile", "FROM scratch\n"),
];

pub fn run(keep: bool) -> Result<i32> {
    let root = make_sample_dir()?;

    // Write the sample monorepo to disk.
    std::fs::write(root.join("services.yaml"), SERVICES_YAML)?;
    for (rel, body) in MARKERS {
        let full = root.join(rel);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(full, body)?;
    }

    println!();
    println!("{}", "svccat demo".bold());
    println!(
        "A sample monorepo with deliberate drift was generated at:\n  {}",
        root.display()
    );

    // Load, discover, and analyze once; reuse the same building blocks the real
    // subcommands use (see `Commands::Check` in main.rs).
    let manifest_path = root.join("services.yaml");
    let m = manifest::Manifest::load(&manifest_path)?;
    let discovered = discovery::discover_services(&root, &m);

    section(
        "svccat check",
        "Compare the declared manifest against the directories in the repo:",
    );
    let mut report = drift::analyze(&m, &discovered, &root);
    report.manifest = manifest_path.display().to_string();
    output::terminal::render_check(&report, &[]);

    section(
        "svccat graph",
        "Render a Mermaid diagram of the catalog, grouped by platform:",
    );
    output::mermaid::render_graph(&m);

    section(
        "svccat stats",
        "Show field-coverage across every declared service:",
    );
    stats::run(&m);

    println!();
    println!("{}", "Next steps".bold());
    println!("  cd <your-repo>");
    println!("  svccat init       # scaffold services.yaml from your repo");
    println!("  svccat check      # detect drift (add --fail-on-drift in CI)");

    if keep {
        println!("\nSample kept at: {}", root.display());
    } else {
        let _ = std::fs::remove_dir_all(&root);
        println!("\n(Sample directory removed; re-run with --keep to inspect it.)");
    }

    Ok(0)
}

/// Create a unique demo directory under the system temp dir.
fn make_sample_dir() -> Result<std::path::PathBuf> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("svccat-demo-{}-{}", std::process::id(), nanos));
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Print a labelled header for one step of the walkthrough.
fn section(cmd: &str, desc: &str) {
    println!();
    println!("{}", format!("$ {cmd}").bold().cyan());
    println!("{desc}");
    println!();
}
