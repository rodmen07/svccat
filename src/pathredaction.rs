use std::path::Path;

/// Redact a file path for display in error messages and logs.
///
/// Converts absolute paths to relative (repo-relative) when possible,
/// preventing information disclosure about system structure.
///
/// # Examples
/// - `/home/user/repo/services/api` → `services/api`
/// - `/usr/local/bin` → `/usr/local/bin` (absolute, kept as-is with warning)
/// - `services.yaml` → `services.yaml` (already relative)
pub fn redact_path(path: &Path, root: &Path) -> String {
    // Try to make path relative to root (repo root)
    if let Ok(rel_path) = path.strip_prefix(root) {
        rel_path.display().to_string()
    } else {
        // Path is outside repo; check if it's absolute
        let path_str = path.to_string_lossy();
        if path.is_absolute() || path_str.starts_with('/') {
            // Log that we're showing an absolute path (potential info leak)
            format!("[absolute path: {}]", path.display())
        } else {
            path.display().to_string()
        }
    }
}

/// Redact a string that may contain file paths or system information.
///
/// Used for error messages from external tools (e.g., git stderr) that may
/// contain absolute paths or other system info.
///
/// Currently a no-op; can be extended to sanitize specific patterns.
pub fn redact_message(message: &str) -> String {
    // TODO: Sanitize git stderr for common absolute paths
    // For now, just return as-is since most paths in git stderr
    // are already relative to the repo
    message.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_relative_path() {
        let root = Path::new("/home/user/repo");
        let path = Path::new("/home/user/repo/services/api");
        assert_eq!(redact_path(path, root), "services/api");
    }

    #[test]
    fn test_redact_already_relative() {
        let root = Path::new("/home/user/repo");
        let path = Path::new("services/api");
        // Relative paths don't have root, so they're shown as-is
        assert_eq!(redact_path(path, root), "services/api");
    }

    #[test]
    fn test_redact_absolute_outside_root() {
        let root = Path::new("/home/user/repo");
        let path = Path::new("/etc/passwd");
        let result = redact_path(path, root);
        assert!(result.contains("absolute path"));
    }
}
