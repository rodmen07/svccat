use crate::discovery::discover_services_with_opts;
use crate::drift::{analyze, DriftKind};
use crate::init::infer_language;
use crate::manifest::{Manifest, ServiceEntry};
use anyhow::{Context, Result};
use colored::Colorize;
use std::path::Path;

pub struct FixSummary {
    pub added: Vec<String>,
    pub pruned: Vec<String>,
}

/// Auto-remediate simple drift in the manifest.
///
/// - Adds `UndeclaredInRepo` services as new entries.
/// - When `prune` is `true`, removes `DeclaredMissingFromRepo` entries.
/// - When `dry_run` is `true`, only prints what would change without writing.
pub fn run(
    manifest_path: &Path,
    root: &Path,
    extra_ignore: &[String],
    depth: u32,
    prune: bool,
    dry_run: bool,
) -> Result<FixSummary> {
    let mut manifest = Manifest::load(manifest_path)?;
    let discovered = discover_services_with_opts(root, &manifest, extra_ignore, depth);
    let report = analyze(&manifest, &discovered, root);

    // Collect UndeclaredInRepo items to add.
    let to_add: Vec<_> = report
        .drifts
        .iter()
        .filter(|d| d.kind == DriftKind::UndeclaredInRepo)
        .filter_map(|d| discovered.iter().find(|s| s.name == d.service))
        .collect();

    // Collect DeclaredMissingFromRepo names to optionally prune.
    let to_prune: Vec<String> = if prune {
        report
            .drifts
            .iter()
            .filter(|d| d.kind == DriftKind::DeclaredMissingFromRepo)
            .map(|d| d.service.clone())
            .collect()
    } else {
        Vec::new()
    };

    if to_add.is_empty() && to_prune.is_empty() {
        println!("{}", "  OK  No fixable drift found.".green().bold());
        return Ok(FixSummary {
            added: vec![],
            pruned: vec![],
        });
    }

    let mut added: Vec<String> = Vec::new();
    let mut pruned: Vec<String> = Vec::new();

    for svc in &to_add {
        let lang = infer_language(root, &svc.path);
        let lang_str = lang.as_deref().unwrap_or("unknown");
        let label = if dry_run { "would add" } else { "+ add    " };
        println!(
            "  {}  {} ({}, {})",
            label.green(),
            svc.name.bold(),
            svc.path,
            lang_str
        );
        added.push(svc.name.clone());
        if !dry_run {
            manifest.services.push(ServiceEntry {
                name: svc.name.clone(),
                path: Some(svc.path.clone()),
                language: lang,
                platform: None,
                url: None,
                role: None,
                team: None,
                oncall: None,
                submodule: None,
                docs: None,
                ci: None,
                tags: Vec::new(),
                depends_on: Vec::new(),
            });
        }
    }

    for name in &to_prune {
        let label = if dry_run { "would prune" } else { "- prune     " };
        println!("  {}  {} (directory not found)", label.red(), name.bold());
        pruned.push(name.clone());
    }

    if !dry_run {
        manifest.services.retain(|s| !to_prune.contains(&s.name));
        let yaml =
            serde_yaml::to_string(&manifest).context("failed to serialize manifest")?;
        std::fs::write(manifest_path, &yaml)
            .with_context(|| format!("failed to write {}", manifest_path.display()))?;
        println!("\n{}", format!("  wrote {}", manifest_path.display()).bold());
    } else {
        println!(
            "\n{}",
            "  (dry-run: no files were modified)".dimmed()
        );
    }

    Ok(FixSummary { added, pruned })
}
