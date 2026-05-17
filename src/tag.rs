use anyhow::{bail, Context, Result};
use std::path::Path;

// ── Public API ────────────────────────────────────────────────────────────────

/// Add a tag to a service in the manifest file.
///
/// Reads and re-writes the manifest YAML in-place.
/// No-ops if the tag already exists.
pub fn add(manifest_path: &Path, service_name: &str, tag: &str) -> Result<()> {
    let text = std::fs::read_to_string(manifest_path)
        .with_context(|| format!("cannot read {}", manifest_path.display()))?;

    let mut doc: serde_yaml::Value =
        serde_yaml::from_str(&text).with_context(|| "cannot parse manifest YAML")?;

    let services = doc
        .get_mut("services")
        .and_then(|v| v.as_sequence_mut())
        .context("manifest has no 'services' list")?;

    let svc = services
        .iter_mut()
        .find(|s| s.get("name").and_then(|n| n.as_str()) == Some(service_name))
        .with_context(|| format!("service '{}' not found in manifest", service_name))?;

    let tags = svc
        .as_mapping_mut()
        .context("service entry is not a mapping")?
        .entry(serde_yaml::Value::String("tags".to_string()))
        .or_insert(serde_yaml::Value::Sequence(vec![]));

    let seq = tags.as_sequence_mut().context("'tags' field is not a list")?;
    let tag_val = serde_yaml::Value::String(tag.to_string());
    if seq.contains(&tag_val) {
        eprintln!("tag '{}' already present on '{}'", tag, service_name);
        return Ok(());
    }
    seq.push(tag_val);

    write_yaml(manifest_path, &doc)?;
    eprintln!("added tag '{}' to '{}'", tag, service_name);
    Ok(())
}

/// Remove a tag from a service in the manifest file.
///
/// Errors if the tag is not present.
pub fn remove(manifest_path: &Path, service_name: &str, tag: &str) -> Result<()> {
    let text = std::fs::read_to_string(manifest_path)
        .with_context(|| format!("cannot read {}", manifest_path.display()))?;

    let mut doc: serde_yaml::Value =
        serde_yaml::from_str(&text).with_context(|| "cannot parse manifest YAML")?;

    let services = doc
        .get_mut("services")
        .and_then(|v| v.as_sequence_mut())
        .context("manifest has no 'services' list")?;

    let svc = services
        .iter_mut()
        .find(|s| s.get("name").and_then(|n| n.as_str()) == Some(service_name))
        .with_context(|| format!("service '{}' not found in manifest", service_name))?;

    let mapping = svc.as_mapping_mut().context("service entry is not a mapping")?;

    let tags_key = serde_yaml::Value::String("tags".to_string());
    let tags = mapping
        .get_mut(&tags_key)
        .and_then(|v| v.as_sequence_mut())
        .with_context(|| format!("service '{}' has no tags", service_name))?;

    let tag_val = serde_yaml::Value::String(tag.to_string());
    let before = tags.len();
    tags.retain(|t| t != &tag_val);

    if tags.len() == before {
        bail!("tag '{}' not found on '{}'", tag, service_name);
    }

    // Remove the empty tags list to keep the YAML tidy.
    if tags.is_empty() {
        mapping.remove(&tags_key);
    }

    write_yaml(manifest_path, &doc)?;
    eprintln!("removed tag '{}' from '{}'", tag, service_name);
    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn write_yaml(path: &Path, doc: &serde_yaml::Value) -> Result<()> {
    let out = serde_yaml::to_string(doc).context("serialising YAML")?;
    std::fs::write(path, out).with_context(|| format!("writing {}", path.display()))
}
