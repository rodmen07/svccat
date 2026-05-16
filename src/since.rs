use anyhow::Result;
use std::path::Path;

use crate::manifest::Manifest;

/// Load the manifest as it existed at `git_ref` using `git show <ref>:<path>`.
///
/// `manifest_path` may be absolute; it will be made relative to `root` before
/// passing to git.
pub fn load_at_ref(root: &Path, manifest_path: &Path, git_ref: &str) -> Result<Manifest> {
    let rel = manifest_path.strip_prefix(root).unwrap_or(manifest_path);

    let output = std::process::Command::new("git")
        .args([
            "-C",
            &root.to_string_lossy(),
            "show",
            &format!("{}:{}", git_ref, rel.to_string_lossy()),
        ])
        .output()?;

    if !output.status.success() {
        anyhow::bail!(
            "git show {}:{} failed: {}",
            git_ref,
            rel.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }

    let text = String::from_utf8(output.stdout)?;
    let m: Manifest = serde_yaml::from_str(&text)
        .map_err(|e| anyhow::anyhow!("cannot parse manifest at {}: {}", git_ref, e))?;
    Ok(m)
}
