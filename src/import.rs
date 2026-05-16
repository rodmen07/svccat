use crate::manifest::{DiscoveryConfig, Manifest, ServiceEntry, DEFAULT_DISCOVERY_PATHS};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

// ── Backstage types (subset we care about) ────────────────────────────────────

#[derive(Debug, Deserialize)]
struct CatalogInfo {
    kind: String,
    metadata: CatalogMetadata,
    spec: Option<CatalogSpec>,
}

#[derive(Debug, Deserialize)]
struct CatalogMetadata {
    name: String,
    #[allow(dead_code)]
    description: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    annotations: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct CatalogSpec {
    #[serde(rename = "type")]
    kind_type: Option<String>,
    owner: Option<String>,
    system: Option<String>,
    #[allow(dead_code)]
    lifecycle: Option<String>,
    #[serde(rename = "dependsOn", default)]
    depends_on: Vec<String>,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Walk `root` for all `catalog-info.yaml` files, parse Backstage Component
/// entities from each, and return them as `ServiceEntry` values ready to merge
/// into a svccat manifest.
pub fn import_backstage(root: &Path) -> Result<Vec<(ServiceEntry, String)>> {
    let mut entries: Vec<(ServiceEntry, String)> = Vec::new();

    for path in find_catalog_files(root) {
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("cannot read {}", path.display()))?;

        // A single file may contain multiple YAML documents separated by "---".
        for doc in text.split("\n---") {
            let doc = doc.trim();
            if doc.is_empty() {
                continue;
            }
            let info: CatalogInfo = match serde_yaml::from_str(doc) {
                Ok(v) => v,
                Err(_) => continue, // skip non-Backstage docs silently
            };

            if !info.kind.eq_ignore_ascii_case("Component") {
                continue;
            }

            let rel_dir = path
                .parent()
                .and_then(|p| p.strip_prefix(root).ok())
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .unwrap_or_default();

            let spec = info.spec.as_ref();

            // Strip "component:<default/>:" or "group:<default/>:" prefixes from
            // depends_on entries that Backstage generates.
            let deps: Vec<String> = spec
                .map(|s| {
                    s.depends_on
                        .iter()
                        .filter_map(|d| {
                            // "component:default/auth-service" -> "auth-service"
                            d.split('/').last().map(str::to_owned)
                        })
                        .collect()
                })
                .unwrap_or_default();

            let svc = ServiceEntry {
                name: info.metadata.name.clone(),
                language: None,
                platform: spec.and_then(|s| s.system.clone()),
                url: None,
                role: spec.and_then(|s| s.kind_type.clone()),
                team: spec.and_then(|s| s.owner.clone()),
                oncall: None,
                submodule: None,
                path: if rel_dir.is_empty() {
                    None
                } else {
                    Some(rel_dir)
                },
                docs: None,
                ci: None,
                depends_on: deps,
            };

            let source = path.display().to_string();
            entries.push((svc, source));
        }
    }

    Ok(entries)
}

/// Merge imported service entries into an existing or new manifest and write it
/// to `output_path`.
///
/// - If `output_path` exists and `force` is false, only new services (by name)
///   are appended; existing entries are left untouched.
/// - If `force` is true the file is overwritten with the merged result.
pub fn run_backstage(root: &Path, output_path: PathBuf, force: bool) -> Result<()> {
    let imported = import_backstage(root)?;

    if imported.is_empty() {
        println!("No Backstage Component entities found in {}.", root.display());
        println!("Make sure your catalog-info.yaml files use `kind: Component`.");
        return Ok(());
    }

    // Load or bootstrap a manifest to merge into.
    let mut manifest = if output_path.exists() && !force {
        Manifest::load(&output_path)?
    } else {
        Manifest {
            version: "1".to_string(),
            discovery: DiscoveryConfig {
                paths: DEFAULT_DISCOVERY_PATHS
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                markers: crate::manifest::default_markers_pub(),
                ignore: vec![],
            },
            policy: Default::default(),
            services: vec![],
        }
    };

    let existing_names: std::collections::HashSet<String> = manifest
        .services
        .iter()
        .map(|s| s.name.clone())
        .collect();

    let mut added = 0usize;
    let mut skipped = 0usize;

    for (svc, source) in &imported {
        if existing_names.contains(&svc.name) {
            eprintln!(
                "  skip  '{}' already in manifest  (from {})",
                svc.name, source
            );
            skipped += 1;
        } else {
            eprintln!("  add   '{}'  (from {})", svc.name, source);
            manifest.services.push(svc.clone());
            added += 1;
        }
    }

    // Serialise back to YAML. serde_yaml produces clean output.
    let yaml = serde_yaml::to_string(&manifest)
        .context("failed to serialise manifest to YAML")?;

    std::fs::write(&output_path, &yaml)
        .with_context(|| format!("cannot write {}", output_path.display()))?;

    println!();
    println!(
        "Wrote {} — added {}, skipped {} (already declared).",
        output_path.display(),
        added,
        skipped
    );
    println!("Run `svccat check` to verify there is no drift.");
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_catalog_files(root: &Path) -> Vec<PathBuf> {
    let pattern = root.join("**/catalog-info.yaml");
    glob::glob(&pattern.to_string_lossy())
        .into_iter()
        .flatten()
        .flatten()
        .collect()
}
