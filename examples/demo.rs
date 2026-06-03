//! Library example: use svccat as a crate to load a manifest, run drift
//! analysis, and inspect the report programmatically.
//!
//! Run it from the crate root:
//!
//! ```text
//! cargo run --example demo
//! ```
//!
//! For the end-user CLI walkthrough instead, run the binary: `svccat demo`.

use std::path::Path;

use svccat::{discovery, drift, manifest::Manifest};

fn main() -> anyhow::Result<()> {
    // The bundled sample monorepo lives under examples/ in this repo. Resolve it
    // from the crate root (set at compile time) so the example works from any CWD.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/sample-monorepo");
    let manifest_path = root.join("services.yaml");

    // 1. Load the declared catalog.
    let manifest = Manifest::load(&manifest_path)?;
    println!(
        "Loaded {} declared service(s) from {}",
        manifest.services.len(),
        manifest_path.display()
    );

    // 2. Discover the service directories that actually exist in the repo.
    let discovered = discovery::discover_services(&root, &manifest);
    println!(
        "Discovered {} service director(ies) on disk",
        discovered.len()
    );

    // 3. Compute drift between the two.
    let report = drift::analyze(&manifest, &discovered, &root);
    println!(
        "\nDrift: {} error(s), {} warning(s)",
        report.error_count(),
        report.warning_count()
    );
    for item in &report.drifts {
        println!(
            "  [{:?}/{:?}] {} - {}",
            item.severity, item.kind, item.service, item.message
        );
    }

    Ok(())
}
