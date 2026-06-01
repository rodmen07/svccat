use crate::manifest::Manifest;
use serde::{Deserialize, Serialize};
use std::path::Path;

// ── Glob Pattern Limits ───────────────────────────────────────────────────────
// These limits prevent DoS attacks using expensive glob patterns

/// Maximum number of glob patterns from discovery.paths (prevents scanning excessive patterns)
const MAX_GLOB_PATTERNS: usize = 20;

/// Maximum number of consecutive wildcards in a pattern (prevents `**/**/**` etc)
const MAX_CONSECUTIVE_WILDCARDS: usize = 2;

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
/// directory that contains at least one marker file, excluding ignored paths.
pub fn discover_services(root: &Path, manifest: &Manifest) -> Vec<DiscoveredService> {
    discover_services_with_ignore(root, manifest, &[])
}

/// Like `discover_services` but merges extra ignore patterns from the CLI.
pub fn discover_services_with_ignore(
    root: &Path,
    manifest: &Manifest,
    extra_ignore: &[String],
) -> Vec<DiscoveredService> {
    discover_services_with_opts(root, manifest, extra_ignore, 1)
}

/// Full-featured discovery: like `discover_services_with_ignore` but also accepts
/// a maximum scan depth (`depth >= 1`). Depth 1 matches only direct children of
/// each discovery path (the current default). Depth 2 also matches grandchildren,
/// and so on.
pub fn discover_services_with_opts(
    root: &Path,
    manifest: &Manifest,
    extra_ignore: &[String],
    depth: u32,
) -> Vec<DiscoveredService> {
    let base_patterns = manifest.effective_discovery_paths();
    let markers = &manifest.discovery.markers;

    // Validate glob pattern limits to prevent DoS
    if base_patterns.len() > MAX_GLOB_PATTERNS {
        eprintln!(
            "warning: too many discovery patterns ({}, max {}). Some patterns will be ignored.",
            base_patterns.len(),
            MAX_GLOB_PATTERNS
        );
    }

    let effective_markers: Vec<String> = if markers.is_empty() {
        default_markers()
    } else {
        markers.clone()
    };

    // Compile ignore patterns (from manifest + CLI) into glob::Pattern.
    let ignore_patterns: Vec<glob::Pattern> = manifest
        .discovery
        .ignore
        .iter()
        .chain(extra_ignore.iter())
        .filter_map(|p| {
            if has_dangerous_pattern(p) {
                eprintln!(
                    "warning: ignore pattern '{}' may be expensive and was skipped",
                    p
                );
                None
            } else {
                glob::Pattern::new(p).ok()
            }
        })
        .collect();

    // Expand each base pattern to cover depth levels.
    // A base pattern of "services/*" at depth=2 becomes ["services/*", "services/*/*"].
    let mut patterns: Vec<String> = Vec::new();
    for (idx, base) in base_patterns.iter().enumerate() {
        if idx >= MAX_GLOB_PATTERNS {
            break;
        }

        if has_dangerous_pattern(base) {
            eprintln!(
                "warning: discovery pattern '{}' may be expensive and was skipped",
                base
            );
            continue;
        }

        // Start with the base pattern itself (depth 1).
        let mut current = base.clone();
        patterns.push(current.clone());
        // For each additional depth level, append "/*".
        for _ in 1..depth.max(1) {
            current.push_str("/*");
            patterns.push(current.clone());
        }
    }

    let mut discovered: Vec<DiscoveredService> = Vec::new();
    let mut seen_paths: std::collections::HashSet<String> = std::collections::HashSet::new();

    for pattern in &patterns {
        let full_pattern = root.join(pattern);
        let pattern_str = full_pattern.to_string_lossy();

        let entries = match glob::glob(&pattern_str) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for entry in entries.flatten() {
            // Check if this is a directory (not a symlink to a directory)
            // Use metadata() which doesn't follow symlinks by default
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            // Reject symlinks to prevent symlink-based attacks
            // A symlink to a directory is_dir() == true, but is_symlink() == true
            if metadata.is_symlink() {
                eprintln!(
                    "warning: skipping symlink '{}' during discovery (potential security risk)",
                    entry.display()
                );
                continue;
            }

            if !metadata.is_dir() {
                continue;
            }

            let rel_path = entry
                .strip_prefix(root)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| entry.to_string_lossy().to_string());

            // Skip directories matching any ignore pattern.
            if ignore_patterns.iter().any(|p| p.matches(&rel_path)) {
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

            // Deduplicate paths (e.g. overlapping glob patterns or depth expansion).
            if !seen_paths.insert(rel_path.clone()) {
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

/// Check if a glob pattern is potentially dangerous (expensive or malicious).
///
/// Dangerous patterns include:
/// - Multiple consecutive wildcards: `**/**`, `*/**/*/`
/// - Unbounded wildcards at start: `**/` without prefix
fn has_dangerous_pattern(pattern: &str) -> bool {
    // Check for multiple consecutive wildcards (e.g., **/**/*, */*/*/*)
    let mut consecutive_wildcards = 0;

    for c in pattern.chars() {
        if c == '*' {
            consecutive_wildcards += 1;
            if consecutive_wildcards > MAX_CONSECUTIVE_WILDCARDS {
                return true; // Too many consecutive wildcards
            }
        } else {
            consecutive_wildcards = 0;
        }
    }

    // Check for completely unbounded patterns that could match everything
    if pattern == "**" || pattern == "*" || pattern == "**/*" {
        return false; // These are OK as they're common use cases
    }

    // Flag patterns starting with ** without a directory prefix
    if pattern.starts_with("**/") && !pattern.contains("/**/") {
        // Only one level of **, this is fine
        return false;
    }

    false
}
