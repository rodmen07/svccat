use std::fs;
use std::path::Path;

/// Create an empty file at the given path, creating directories as needed.
pub fn touch(root: &Path, rel_path: &str) {
    let full = root.join(rel_path);
    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::File::create(&full).unwrap();
}

/// Write a manifest to the default location (services.yaml).
pub fn write_manifest(root: &Path, content: &str) {
    fs::write(root.join("services.yaml"), content).unwrap();
}
