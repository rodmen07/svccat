use anyhow::Result;
use std::path::Path;

use crate::manifest::Manifest;
use crate::pathredaction;

/// Validate git reference format to prevent injection attacks.
/// Allows: branch names, tags, commit SHAs, and git special refs like HEAD, HEAD~1, etc.
/// Rejects: paths with special characters or suspicious patterns.
fn validate_git_ref(git_ref: &str) -> Result<()> {
    if git_ref.is_empty() {
        anyhow::bail!("git reference cannot be empty");
    }
    if git_ref.len() > 256 {
        anyhow::bail!("git reference too long (max 256 chars)");
    }

    // Check for null bytes and dangerous patterns
    if git_ref.contains('\0') || git_ref.contains("..") {
        anyhow::bail!("invalid git reference '{}': dangerous pattern detected", git_ref);
    }

    // Whitelist allowed characters: alphanumerics, common git special chars, separators
    let is_valid = git_ref.chars().all(|c| {
        c.is_alphanumeric() || matches!(c, '.' | '_' | '/' | '-' | ':' | '@' | '^' | '~' | '!' | '+')
    });

    if !is_valid {
        anyhow::bail!("invalid git reference '{}': contains illegal characters", git_ref);
    }

    Ok(())
}

/// Validate that a manifest path doesn't escape the repo root.
fn validate_manifest_path(path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy();

    // Reject absolute paths
    if path.is_absolute() {
        anyhow::bail!("manifest path must be relative to repo root");
    }

    // Reject parent directory traversal
    if path_str.contains("..") {
        anyhow::bail!("manifest path cannot contain '..' (attempted path traversal)");
    }

    // Reject suspicious patterns
    if path_str.starts_with('/') || path_str.contains('\0') {
        anyhow::bail!("manifest path contains invalid characters");
    }

    Ok(())
}

/// Load the manifest as it existed at `git_ref` using `git show <ref>:<path>`.
///
/// `manifest_path` may be absolute; it will be made relative to `root` before
/// passing to git.
///
/// # Security
/// - Validates git ref against a strict allowlist pattern
/// - Validates manifest path to prevent directory traversal
/// - Uses `--` separator to prevent git from misinterpreting arguments
pub fn load_at_ref(root: &Path, manifest_path: &Path, git_ref: &str) -> Result<Manifest> {
    validate_git_ref(git_ref)?;

    let rel = manifest_path.strip_prefix(root).unwrap_or(manifest_path);
    validate_manifest_path(rel)?;

    let spec = format!("{}:{}", git_ref, rel.to_string_lossy());

    let output = std::process::Command::new("git")
        .args([
            "-C",
            &root.to_string_lossy(),
            "show",
            "--",  // Separator: prevent git from interpreting spec as a ref/path
            &spec,
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let redacted_error = pathredaction::redact_message(stderr.trim());
        anyhow::bail!("git show failed at {}: {}", git_ref, redacted_error);
    }

    let text = String::from_utf8(output.stdout)?;
    let m: Manifest = serde_yaml::from_str(&text)
        .map_err(|e| anyhow::anyhow!("cannot parse manifest at {}: {}", git_ref, e))?;
    Ok(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_git_refs() {
        assert!(validate_git_ref("main").is_ok());
        assert!(validate_git_ref("HEAD").is_ok());
        assert!(validate_git_ref("HEAD~1").is_ok());
        assert!(validate_git_ref("v1.0.0").is_ok());
        assert!(validate_git_ref("feature/auth-service").is_ok());
        assert!(validate_git_ref("release-2024-01-15").is_ok());
    }

    #[test]
    fn test_invalid_git_refs() {
        assert!(validate_git_ref("").is_err());
        assert!(validate_git_ref("..").is_err());
        assert!(validate_git_ref("HEAD; echo hacked").is_err());
        assert!(validate_git_ref("$(whoami)").is_err());
        assert!(validate_git_ref("HEAD\0hidden").is_err());
    }

    #[test]
    fn test_valid_manifest_paths() {
        assert!(validate_manifest_path(Path::new("services.yaml")).is_ok());
        assert!(validate_manifest_path(Path::new("config/services.yaml")).is_ok());
    }

    #[test]
    fn test_invalid_manifest_paths() {
        assert!(validate_manifest_path(Path::new("../etc/passwd")).is_err());
        assert!(validate_manifest_path(Path::new("/etc/passwd")).is_err());
        assert!(validate_manifest_path(Path::new("foo\0bar")).is_err());
    }
}
