use crate::manifest::Manifest;
use serde::{Deserialize, Serialize};
use std::path::Path;

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredService {
    /// Directory name — used as the service name when matching.
    pub name: String,
    /// Path relative to the repo root.
    pub path: String,
    /// Marker files found in this directory.
    pub markers_found: Vec<String>,
}

// ── Discovery ─────────────────────────────────────────────────────────────────

/// Walk the repo using the effective discovery patterns and return every
/// directory that contains at least one marker file.
pub fn discover_services(root: &Path, manifest: &Manifest) -> Vec<DiscoveredService> {
    let patterns = manifest.effective_discovery_paths();
    let markers = &manifest.discovery.markers;

    let effective_markers: Vec<String> = if markers.is_empty() {
        default_markers()
    } else {
        markers.clone()
    };

    let mut discovered: Vec<DiscoveredService> = Vec::new();

    for pattern in &patterns {
        let full_pattern = root.join(pattern);
        let pattern_str = full_pattern.to_string_lossy();

        let entries = match glob::glob(&pattern_str) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            if !entry.is_dir() {
                continue;
            }

            let found_markers = markers_in_dir(&entry, &effective_markers);
            if found_markers.is_empty() {
                continue;
            }

            let name = entry
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let rel_path = entry
                .strip_prefix(root)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| entry.to_string_lossy().to_string());

            // Deduplicate paths (e.g. overlapping glob patterns).
            if discovered.iter().any(|d| d.path == rel_path) {
                continue;
            }

            discovered.push(DiscoveredService {
                name,
                path: rel_path,
                markers_found: found_markers,
            });
        }
    }

    discovered
}

/// Returns true if `rel_path` (relative to `root`) is a directory containing
/// at least one of the given marker files.
pub fn dir_has_markers(root: &Path, rel_path: &str, markers: &[String]) -> bool {
    let full = root.join(rel_path);
    if !full.is_dir() {
        return false;
    }
    !markers_in_dir(&full, markers).is_empty()
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn markers_in_dir(dir: &Path, markers: &[String]) -> Vec<String> {
    markers
        .iter()
        .filter(|m| dir.join(m.as_str()).exists())
        .cloned()
        .collect()
}

fn default_markers() -> Vec<String> {
    [
        "Cargo.toml",
        "Dockerfile",
        "go.mod",
        "package.json",
        "pyproject.toml",
        "requirements.txt",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}
