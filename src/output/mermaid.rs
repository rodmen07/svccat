use crate::drift::DriftReport;
use crate::manifest::Manifest;
use std::collections::BTreeMap;

pub fn render_graph(manifest: &Manifest) {
    // Group services by platform, using BTreeMap for deterministic order.
    let mut groups: BTreeMap<String, Vec<&crate::manifest::ServiceEntry>> = BTreeMap::new();
    for svc in &manifest.services {
        let platform = svc
            .platform
            .clone()
            .unwrap_or_else(|| "Undeployed".to_string());
        groups.entry(platform).or_default().push(svc);
    }

    println!("```mermaid");
    println!("graph TD");

    for (platform, services) in &groups {
        let safe_plat = safe_id(platform);
        println!("  subgraph {}[\"{}\"]", safe_plat, platform);
        for svc in services {
            let node_id = safe_id(&svc.name);
            let label = build_label(svc);
            println!("    {}[\"{}\"]", node_id, label);
        }
        println!("  end");
    }

    // Render dependency edges after all subgraphs.
    for svc in &manifest.services {
        for dep in &svc.depends_on {
            println!("  {} --> {}", safe_id(&svc.name), safe_id(dep));
        }
    }

    println!("```");
}

pub fn render_markdown_table(manifest: &Manifest) {
    println!("# Service Catalog\n");
    println!("| Service | Language | Platform | Role | URL |");
    println!("|---------|----------|----------|------|-----|");
    for svc in &manifest.services {
        println!(
            "| {} | {} | {} | {} | {} |",
            svc.name,
            svc.language.as_deref().unwrap_or("—"),
            svc.platform.as_deref().unwrap_or("—"),
            svc.role.as_deref().unwrap_or("—"),
            svc.url
                .as_deref()
                .map(|u| format!("[link]({})", u))
                .unwrap_or_else(|| "—".to_string()),
        );
    }
}

pub fn render_export_markdown(manifest: &Manifest, report: &DriftReport) {
    render_markdown_table(manifest);

    if !report.drifts.is_empty() {
        println!("\n## Drift Report\n");
        println!("| Severity | Kind | Service | Message |");
        println!("|----------|------|---------|---------|");
        for item in &report.drifts {
            let severity = format!("{:?}", item.severity).to_lowercase();
            let kind = format!("{:?}", item.kind);
            println!(
                "| {} | {} | {} | {} |",
                severity, kind, item.service, item.message
            );
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn safe_id(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn build_label(svc: &crate::manifest::ServiceEntry) -> String {
    match (&svc.language, &svc.role) {
        (Some(lang), Some(role)) => format!("{}\\n{}\\n{}", svc.name, lang, role),
        (Some(lang), None) => format!("{}\\n{}", svc.name, lang),
        (None, Some(role)) => format!("{}\\n{}", svc.name, role),
        (None, None) => svc.name.clone(),
    }
}
