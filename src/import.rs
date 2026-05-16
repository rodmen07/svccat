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

// ── Docker Compose types ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct DockerComposeFile {
    services: std::collections::HashMap<String, DockerComposeService>,
}

#[derive(Debug, Deserialize, Default)]
struct DockerComposeService {
    build: Option<DockerBuild>,
    #[allow(dead_code)]
    image: Option<String>,
    #[serde(default)]
    depends_on: serde_yaml::Value,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DockerBuild {
    Simple(String),
    Extended { context: String },
}

impl DockerBuild {
    fn context_path(&self) -> &str {
        match self {
            Self::Simple(s) => s.as_str(),
            Self::Extended { context } => context.as_str(),
        }
    }
}

fn parse_compose_depends_on(val: &serde_yaml::Value) -> Vec<String> {
    match val {
        serde_yaml::Value::Sequence(seq) => seq
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect(),
        serde_yaml::Value::Mapping(map) => map
            .iter()
            .filter_map(|(k, _)| k.as_str().map(str::to_owned))
            .collect(),
        _ => vec![],
    }
}

// ── Docker Compose public API ─────────────────────────────────────────────────

/// Parse all services from a `docker-compose.yml` / `compose.yaml` file found
/// in `root` and return them as `ServiceEntry` values.
pub fn import_docker_compose(root: &Path) -> Result<Vec<(ServiceEntry, String)>> {
    let compose_path = find_compose_file(root)
        .ok_or_else(|| anyhow::anyhow!(
            "no docker-compose.yml / compose.yaml found in {}",
            root.display()
        ))?;

    let text = std::fs::read_to_string(&compose_path)
        .with_context(|| format!("cannot read {}", compose_path.display()))?;

    let compose: DockerComposeFile = serde_yaml::from_str(&text)
        .with_context(|| format!("cannot parse {}", compose_path.display()))?;

    let source = compose_path.display().to_string();
    let mut entries: Vec<(ServiceEntry, String)> = Vec::new();

    let mut names: Vec<&String> = compose.services.keys().collect();
    names.sort();

    for name in names {
        let svc = &compose.services[name];

        let path = svc.build.as_ref().map(|b| {
            // Normalise "./" prefix and backslashes.
            b.context_path()
                .trim_start_matches("./")
                .replace('\\', "/")
        });

        let depends_on = parse_compose_depends_on(&svc.depends_on);

        entries.push((
            ServiceEntry {
                name: name.clone(),
                path,
                language: None,
                platform: None,
                url: None,
                role: None,
                team: None,
                oncall: None,
                submodule: None,
                docs: None,
                ci: None,
                depends_on,
            },
            source.clone(),
        ));
    }

    Ok(entries)
}

/// Merge docker-compose services into an existing or new manifest.
pub fn run_docker_compose(root: &Path, output_path: PathBuf, force: bool) -> Result<()> {
    let imported = import_docker_compose(root)?;

    if imported.is_empty() {
        println!("No services found in the compose file.");
        return Ok(());
    }

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

    let existing_names: std::collections::HashSet<String> =
        manifest.services.iter().map(|s| s.name.clone()).collect();

    let mut added = 0usize;
    let mut skipped = 0usize;

    for (svc, source) in &imported {
        if existing_names.contains(&svc.name) {
            eprintln!("  skip  '{}' already in manifest  (from {})", svc.name, source);
            skipped += 1;
        } else {
            eprintln!("  add   '{}'  (from {})", svc.name, source);
            manifest.services.push(svc.clone());
            added += 1;
        }
    }

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

fn find_compose_file(root: &Path) -> Option<PathBuf> {
    for name in &[
        "docker-compose.yml",
        "docker-compose.yaml",
        "compose.yml",
        "compose.yaml",
    ] {
        let p = root.join(name);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

// ── OpenAPI / Swagger types ───────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct OpenApiFile {
    info: OpenApiInfo,
    /// OpenAPI 3.x server list.
    servers: Option<Vec<OpenApiServer>>,
    /// Swagger 2.x host field.
    host: Option<String>,
    /// Swagger 2.x basePath field.
    #[serde(rename = "basePath")]
    base_path: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenApiInfo {
    title: String,
    #[serde(rename = "x-team")]
    x_team: Option<String>,
    #[serde(rename = "x-oncall")]
    x_oncall: Option<String>,
    #[serde(rename = "x-language")]
    x_language: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenApiServer {
    url: String,
}

// ── OpenAPI public API ────────────────────────────────────────────────────────

/// Walk `root` for OpenAPI / Swagger spec files and return `ServiceEntry` values.
///
/// Recognised file names: `openapi.yaml`, `openapi.yml`, `swagger.yaml`, `swagger.yml`.
/// The service name is derived from `info.title` (slugified to lowercase kebab-case).
pub fn import_openapi(root: &Path) -> Result<Vec<(ServiceEntry, String)>> {
    let mut entries: Vec<(ServiceEntry, String)> = Vec::new();

    let patterns = [
        "**/openapi.yaml",
        "**/openapi.yml",
        "**/swagger.yaml",
        "**/swagger.yml",
    ];

    let mut paths: Vec<PathBuf> = Vec::new();
    for pattern in &patterns {
        let full_pattern = root.join(pattern);
        let matches = glob::glob(&full_pattern.to_string_lossy())
            .into_iter()
            .flatten()
            .flatten();
        paths.extend(matches);
    }
    paths.sort();
    paths.dedup();

    for path in &paths {
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let spec: OpenApiFile = match serde_yaml::from_str(&text) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let name = slugify(&spec.info.title);
        if name.is_empty() {
            continue;
        }

        let url = spec
            .servers
            .as_ref()
            .and_then(|s| s.first())
            .map(|s| s.url.clone())
            .or_else(|| {
                // Swagger 2.x: build URL from host + basePath.
                spec.host.as_ref().map(|h| {
                    let base = spec.base_path.as_deref().unwrap_or("");
                    format!("https://{h}{base}")
                })
            });

        let rel_dir = path
            .parent()
            .and_then(|p| p.strip_prefix(root).ok())
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();

        let svc = ServiceEntry {
            name,
            language: spec.info.x_language,
            platform: None,
            url,
            role: Some("api".to_string()),
            team: spec.info.x_team,
            oncall: spec.info.x_oncall,
            submodule: None,
            path: if rel_dir.is_empty() { None } else { Some(rel_dir) },
            docs: None,
            ci: None,
            depends_on: vec![],
        };

        entries.push((svc, path.display().to_string()));
    }

    Ok(entries)
}

/// Merge OpenAPI-imported services into an existing or new manifest.
pub fn run_openapi(root: &Path, output_path: PathBuf, force: bool) -> Result<()> {
    let imported = import_openapi(root)?;

    if imported.is_empty() {
        println!(
            "No OpenAPI / Swagger specs found in {}.",
            root.display()
        );
        println!("Searched for: openapi.yaml, openapi.yml, swagger.yaml, swagger.yml");
        return Ok(());
    }

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

    let existing_names: std::collections::HashSet<String> =
        manifest.services.iter().map(|s| s.name.clone()).collect();

    let mut added = 0usize;
    let mut skipped = 0usize;

    for (svc, source) in &imported {
        if existing_names.contains(&svc.name) {
            eprintln!("  skip  '{}' already in manifest  (from {})", svc.name, source);
            skipped += 1;
        } else {
            eprintln!("  add   '{}'  (from {})", svc.name, source);
            manifest.services.push(svc.clone());
            added += 1;
        }
    }

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

// ── OpenAPI helper ────────────────────────────────────────────────────────────

/// Convert an API title to a lowercase kebab-case slug suitable for service names.
///
/// Example: "User Service API" -> "user-service-api"
fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
