use crate::drift::DriftReport;
use crate::manifest::Manifest;
use std::collections::BTreeMap;

pub fn render_graph(manifest: &Manifest) {
    render_graph_filtered(manifest, None);
}

/// Render the graph optionally filtered to a single team.
///
/// When `team` is `Some`, only services owned by that team are included as
/// primary nodes.  Services referenced via `depends_on` that belong to *other*
/// teams (or have no `team:` set) are rendered as external placeholder nodes
/// outside any subgraph, clearly marked as `[ext]`.
pub fn render_graph_filtered(manifest: &Manifest, team: Option<&str>) {
    // Determine in-scope service names.
    let in_scope: std::collections::HashSet<&str> = manifest
        .services
        .iter()
        .filter(|s| match team {
            None => true,
            Some(t) => s
                .team
                .as_deref()
                .map(|v| v.eq_ignore_ascii_case(t))
                .unwrap_or(false),
        })
        .map(|s| s.name.as_str())
        .collect();

    // Group in-scope services by platform for subgraphs.
    let mut groups: BTreeMap<String, Vec<&crate::manifest::ServiceEntry>> = BTreeMap::new();
    for svc in &manifest.services {
        if in_scope.contains(svc.name.as_str()) {
            let platform = svc
                .platform
                .clone()
                .unwrap_or_else(|| "Undeployed".to_string());
            groups.entry(platform).or_default().push(svc);
        }
    }

    // Collect external dependency targets (cross-team or undeclared).
    let declared_names: std::collections::HashSet<&str> =
        manifest.services.iter().map(|s| s.name.as_str()).collect();
    let mut external: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for svc in &manifest.services {
        if !in_scope.contains(svc.name.as_str()) {
            continue;
        }
        for dep in &svc.depends_on {
            if !in_scope.contains(dep.as_str()) && declared_names.contains(dep.as_str()) {
                external.insert(dep.as_str());
            }
        }
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

    // External nodes (cross-team dependencies): rendered as plain nodes with [ext] tag.
    if !external.is_empty() {
        println!("  subgraph External[\"External (other teams)\"]");
        for name in &external {
            println!("    {}[\"{}\\n[ext]\"]", safe_id(name), name);
        }
        println!("  end");
    }

    // Render dependency edges for in-scope services.
    for svc in &manifest.services {
        if !in_scope.contains(svc.name.as_str()) {
            continue;
        }
        for dep in &svc.depends_on {
            if in_scope.contains(dep.as_str()) || external.contains(dep.as_str()) {
                println!("  {} --> {}", safe_id(&svc.name), safe_id(dep));
            }
        }
    }

    println!("```");
}

pub fn render_markdown_table(manifest: &Manifest) {
    println!("# Service Catalog\n");
    println!("| Service | Language | Platform | Role | Team | URL |");
    println!("|---------|----------|----------|------|------|-----|");
    for svc in &manifest.services {
        println!(
            "| {} | {} | {} | {} | {} | {} |",
            svc.name,
            svc.language.as_deref().unwrap_or("—"),
            svc.platform.as_deref().unwrap_or("—"),
            svc.role.as_deref().unwrap_or("—"),
            svc.team.as_deref().unwrap_or("—"),
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

// ── Graphviz DOT renderer ─────────────────────────────────────────────────────

/// Emit a Graphviz DOT digraph of the service dependency graph.
///
/// Services are grouped into DOT subgraph clusters by platform.  When `team`
/// is `Some`, only services owned by that team are included as primary nodes.
pub fn render_dot(manifest: &Manifest, team: Option<&str>) {
    let in_scope: std::collections::HashSet<&str> = manifest
        .services
        .iter()
        .filter(|s| match team {
            None => true,
            Some(t) => s
                .team
                .as_deref()
                .map(|v| v.eq_ignore_ascii_case(t))
                .unwrap_or(false),
        })
        .map(|s| s.name.as_str())
        .collect();

    // Group in-scope services by platform.
    let mut groups: BTreeMap<String, Vec<&crate::manifest::ServiceEntry>> = BTreeMap::new();
    for svc in &manifest.services {
        if in_scope.contains(svc.name.as_str()) {
            let platform = svc
                .platform
                .clone()
                .unwrap_or_else(|| "undeployed".to_string());
            groups.entry(platform).or_default().push(svc);
        }
    }

    println!("digraph services {{");
    println!("  rankdir=LR;");
    println!("  node [shape=box fontname=Helvetica];");
    println!();

    for (idx, (platform, services)) in groups.iter().enumerate() {
        println!("  subgraph cluster_{idx} {{");
        println!("    label=\"{}\";", dot_escape(platform));
        println!("    style=filled;");
        println!("    color=lightgrey;");
        for svc in services {
            println!("    \"{}\" [label=\"{}\"];", dot_escape(&svc.name), dot_escape(&svc.name));
        }
        println!("  }}");
        println!();
    }

    // Dependency edges.
    for svc in &manifest.services {
        if !in_scope.contains(svc.name.as_str()) {
            continue;
        }
        for dep in &svc.depends_on {
            println!("  \"{}\" -> \"{}\";", dot_escape(&svc.name), dot_escape(dep));
        }
    }

    println!("}}");
}

fn dot_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

// ── PlantUML renderer ─────────────────────────────────────────────────────────

/// Emit a PlantUML component diagram of the service dependency graph.
///
/// Services are grouped as packages by platform.  Pipe the output to
/// `plantuml -pipe` or paste into https://www.plantuml.com/plantuml/uml/
pub fn render_plantuml(manifest: &Manifest, team: Option<&str>) {
    let in_scope: std::collections::HashSet<&str> = manifest
        .services
        .iter()
        .filter(|s| match team {
            None => true,
            Some(t) => s
                .team
                .as_deref()
                .map(|v| v.eq_ignore_ascii_case(t))
                .unwrap_or(false),
        })
        .map(|s| s.name.as_str())
        .collect();

    // Group in-scope services by platform for packages.
    let mut groups: BTreeMap<String, Vec<&crate::manifest::ServiceEntry>> = BTreeMap::new();
    for svc in &manifest.services {
        if in_scope.contains(svc.name.as_str()) {
            let platform = svc
                .platform
                .clone()
                .unwrap_or_else(|| "undeployed".to_string());
            groups.entry(platform).or_default().push(svc);
        }
    }

    println!("@startuml");
    println!("skinparam componentStyle rectangle");
    println!();

    for (platform, services) in &groups {
        println!("package \"{}\" {{", plantuml_escape(platform));
        for svc in services {
            let label = match &svc.language {
                Some(lang) => format!("{} ({})", svc.name, lang),
                None => svc.name.clone(),
            };
            println!(
                "  component [{}] as {}",
                plantuml_escape(&label),
                plantuml_id(&svc.name)
            );
        }
        println!("}}");
        println!();
    }

    // Dependency edges.
    for svc in &manifest.services {
        if !in_scope.contains(svc.name.as_str()) {
            continue;
        }
        for dep in &svc.depends_on {
            println!(
                "{} ..> {} : depends",
                plantuml_id(&svc.name),
                plantuml_id(dep)
            );
        }
    }

    println!();
    println!("@enduml");
}

fn plantuml_escape(s: &str) -> String {
    s.replace('"', "'")
}

fn plantuml_id(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
}
