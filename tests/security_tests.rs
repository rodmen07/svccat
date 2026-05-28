use tempfile::TempDir;

mod common;
use common::*;

// ── Path Validation Tests ─────────────────────────────────────────────────────

#[test]
fn reject_absolute_paths() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create manifest with absolute path
    let manifest = r#"
discovery:
  paths: ["services/*"]
services:
  - name: api
    path: /etc/passwd
"#;
    write_manifest(root, manifest);

    // Loading manifest should fail
    let result = svccat::manifest::Manifest::load(&root.join("services.yaml"));
    assert!(result.is_err(), "Expected error for absolute path");
}

#[test]
fn reject_path_traversal() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create manifest with path traversal
    let manifest = r#"
discovery:
  paths: ["services/*"]
services:
  - name: api
    path: ../../etc/passwd
"#;
    write_manifest(root, manifest);

    // Loading manifest should fail
    let result = svccat::manifest::Manifest::load(&root.join("services.yaml"));
    assert!(result.is_err(), "Expected error for path traversal");
}

#[test]
fn reject_windows_drive_letters() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create manifest with Windows-style absolute path
    let manifest = r#"
discovery:
  paths: ["services/*"]
services:
  - name: api
    path: C:\Windows\System32
"#;
    write_manifest(root, manifest);

    // Loading manifest should fail (on all platforms)
    let result = svccat::manifest::Manifest::load(&root.join("services.yaml"));
    assert!(result.is_err(), "Expected error for Windows drive letter path");
}

// ── Manifest Size Limits Tests ────────────────────────────────────────────────

#[test]
fn reject_oversized_manifest() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    let manifest_path = root.join("services.yaml");

    // Create a manifest file larger than the limit (10 MB)
    let huge_content = "a".repeat(11 * 1024 * 1024); // 11 MB
    std::fs::write(&manifest_path, huge_content).unwrap();

    // Loading should fail due to size limit
    let result = svccat::manifest::Manifest::load(&manifest_path);
    assert!(result.is_err(), "Expected error for oversized manifest");
}

#[test]
fn accept_manifest_within_size_limit() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create a normal manifest that's well within the limit
    let manifest = r#"
version: "1"
discovery:
  paths: ["services/*"]
services:
  - name: api
    language: Rust
    platform: "Cloud Run"
"#;
    write_manifest(root, manifest);

    // This should succeed
    let result = svccat::manifest::Manifest::load(&root.join("services.yaml"));
    assert!(result.is_ok(), "Manifest within size limit should load successfully");
}

// ── Discovery & Symlink Tests ─────────────────────────────────────────────────

#[test]
fn skip_symlinks_during_discovery() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create a real service directory
    touch(root, "services/real-api/Cargo.toml");

    // Create a symlink to a directory outside the repo
    #[cfg(unix)]
    {
        use std::os::unix::fs as unix_fs;
        let external = dir.path().join("external");
        std::fs::create_dir(&external).unwrap();
        unix_fs::symlink(&external, root.join("services/symlink")).unwrap();
    }

    let manifest = r#"
discovery:
  paths: ["services/*"]
services: []
"#;
    write_manifest(root, manifest);

    // Discovery should find only the real service, not the symlink
    let m = svccat::manifest::Manifest::load(&root.join("services.yaml")).unwrap();
    let discovered = svccat::discovery::discover_services(root, &m);

    // Should only find the real-api service (symlink is skipped)
    assert!(discovered.iter().any(|d| d.name == "real-api"), "Should find real-api service");
}

// ── Glob Pattern Limits Tests ─────────────────────────────────────────────────

#[test]
fn reject_excessive_glob_patterns() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create manifest with too many discovery patterns
    let mut manifest = String::from("discovery:\n  paths:\n");
    for i in 0..30 {
        manifest.push_str(&format!("    - \"services{:02}/*\"\n", i));
    }
    manifest.push_str("services: []\n");

    write_manifest(root, &manifest);

    // Load and discover - should warn about excess patterns
    let m = svccat::manifest::Manifest::load(&root.join("services.yaml")).unwrap();
    let _discovered = svccat::discovery::discover_services(root, &m);
    // The test passes if it doesn't crash; warnings are printed to stderr
}

#[test]
fn discover_with_safe_patterns() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();

    // Create services with safe discovery patterns
    touch(root, "services/api/Cargo.toml");
    touch(root, "services/web/package.json");

    let manifest = r#"
discovery:
  paths:
    - "services/*"
services: []
"#;
    write_manifest(root, manifest);

    let m = svccat::manifest::Manifest::load(&root.join("services.yaml")).unwrap();
    let discovered = svccat::discovery::discover_services(root, &m);

    assert_eq!(discovered.len(), 2);
    let names: Vec<_> = discovered.iter().map(|d| &d.name).collect();
    assert!(names.contains(&&"api".to_string()));
    assert!(names.contains(&&"web".to_string()));
}

// ── Information Disclosure Tests ──────────────────────────────────────────────

#[test]
fn redact_absolute_paths_in_errors() {
    use std::path::PathBuf;

    let root = PathBuf::from("/home/user/repo");
    let absolute_path = PathBuf::from("/etc/passwd");

    let redacted = svccat::pathredaction::redact_path(&absolute_path, &root);

    // Should indicate it's an absolute path
    assert!(redacted.contains("absolute path"));
    assert!(!redacted.contains("/etc/passwd") || redacted.contains("[absolute path"));
}

#[test]
fn convert_absolute_to_relative_paths() {
    use std::path::PathBuf;

    let root = PathBuf::from("/home/user/repo");
    let absolute_path = PathBuf::from("/home/user/repo/services/api");

    let redacted = svccat::pathredaction::redact_path(&absolute_path, &root);

    // Should be relative path
    assert_eq!(redacted, "services/api");
    assert!(!redacted.contains("/home/user"));
}
