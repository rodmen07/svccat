use crate::drift::DriftReport;
use crate::manifest::Manifest;
use crate::output::d3_force_graph::{self, D3GraphConfig, TooltipField};
use crate::output::json_script;
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
            println!(
                "    \"{}\" [label=\"{}\"];",
                dot_escape(&svc.name),
                dot_escape(&svc.name)
            );
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
            println!(
                "  \"{}\" -> \"{}\";",
                dot_escape(&svc.name),
                dot_escape(dep)
            );
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
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

// ── String-building helpers for --output support ───────────────────────────────

/// Macro to build graph content into a String instead of printing to stdout.
macro_rules! sbuf {
    ($buf:ident, $($arg:tt)*) => {
        writeln!($buf, $($arg)*).unwrap()
    };
}

pub fn render_graph_filtered_string(manifest: &Manifest, team: Option<&str>) -> String {
    use std::fmt::Write;
    let mut buf = String::new();

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

    sbuf!(buf, "```mermaid");
    sbuf!(buf, "graph TD");
    for (platform, services) in &groups {
        let safe_plat = safe_id(platform);
        sbuf!(buf, "  subgraph {}[\"{}\"]", safe_plat, platform);
        for svc in services {
            sbuf!(buf, "    {}[\"{}\"]", safe_id(&svc.name), build_label(svc));
        }
        sbuf!(buf, "  end");
    }
    if !external.is_empty() {
        sbuf!(buf, "  subgraph External[\"External (other teams)\"]");
        for name in &external {
            sbuf!(buf, "    {}[\"{}\\n[ext]\"]", safe_id(name), name);
        }
        sbuf!(buf, "  end");
    }
    for svc in &manifest.services {
        if !in_scope.contains(svc.name.as_str()) {
            continue;
        }
        for dep in &svc.depends_on {
            if in_scope.contains(dep.as_str()) || external.contains(dep.as_str()) {
                sbuf!(buf, "  {} --> {}", safe_id(&svc.name), safe_id(dep));
            }
        }
    }
    sbuf!(buf, "```");
    buf
}

pub fn render_markdown_table_string(manifest: &Manifest) -> String {
    use std::fmt::Write;
    let mut buf = String::new();
    sbuf!(buf, "# Service Catalog\n");
    sbuf!(buf, "| Service | Language | Platform | Role | Team | URL |");
    sbuf!(buf, "|---------|----------|----------|------|------|-----|");
    for svc in &manifest.services {
        sbuf!(
            buf,
            "| {} | {} | {} | {} | {} | {} |",
            svc.name,
            svc.language.as_deref().unwrap_or(""),
            svc.platform.as_deref().unwrap_or(""),
            svc.role.as_deref().unwrap_or(""),
            svc.team.as_deref().unwrap_or(""),
            svc.url.as_deref().unwrap_or("")
        );
    }
    buf
}

pub fn render_dot_string(manifest: &Manifest, team: Option<&str>) -> String {
    use std::fmt::Write;
    let mut buf = String::new();

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

    sbuf!(buf, "digraph services {{");
    sbuf!(buf, "  rankdir=LR;");
    sbuf!(buf, "  node [shape=box fontname=Helvetica];");
    sbuf!(buf, "");
    for (idx, (platform, services)) in groups.iter().enumerate() {
        sbuf!(buf, "  subgraph cluster_{idx} {{");
        sbuf!(buf, "    label=\"{}\";", dot_escape(platform));
        sbuf!(buf, "    style=filled;");
        sbuf!(buf, "    color=lightgrey;");
        for svc in services {
            sbuf!(
                buf,
                "    \"{}\" [label=\"{}\"];",
                dot_escape(&svc.name),
                dot_escape(&svc.name)
            );
        }
        sbuf!(buf, "  }}");
        sbuf!(buf, "");
    }
    for svc in &manifest.services {
        if !in_scope.contains(svc.name.as_str()) {
            continue;
        }
        for dep in &svc.depends_on {
            sbuf!(
                buf,
                "  \"{}\" -> \"{}\";",
                dot_escape(&svc.name),
                dot_escape(dep)
            );
        }
    }
    sbuf!(buf, "}}");
    buf
}

pub fn render_plantuml_string(manifest: &Manifest, team: Option<&str>) -> String {
    use std::fmt::Write;
    let mut buf = String::new();

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

    sbuf!(buf, "@startuml");
    sbuf!(buf, "skinparam componentStyle rectangle");
    sbuf!(buf, "");
    for (platform, services) in &groups {
        sbuf!(buf, "package \"{}\" {{", plantuml_escape(platform));
        for svc in services {
            let label = match &svc.language {
                Some(lang) => format!("{} ({})", svc.name, lang),
                None => svc.name.clone(),
            };
            sbuf!(
                buf,
                "  component [{}] as {}",
                plantuml_escape(&label),
                plantuml_id(&svc.name)
            );
        }
        sbuf!(buf, "}}");
        sbuf!(buf, "");
    }
    for svc in &manifest.services {
        if !in_scope.contains(svc.name.as_str()) {
            continue;
        }
        for dep in &svc.depends_on {
            sbuf!(
                buf,
                "{} ..> {} : depends",
                plantuml_id(&svc.name),
                plantuml_id(dep)
            );
        }
    }
    sbuf!(buf, "");
    sbuf!(buf, "@enduml");
    buf
}

// ── HTML interactive graph ─────────────────────────────────────────────────────

/// A D3 node for the single-repo graph: one entry per in-scope service,
/// carrying the fields the tooltip and colour scale read.
#[derive(serde::Serialize)]
struct D3Node<'a> {
    id: &'a str,
    platform: &'a str,
    team: &'a str,
    language: &'a str,
}

#[derive(serde::Serialize)]
struct D3Link<'a> {
    source: &'a str,
    target: &'a str,
}

#[derive(serde::Serialize)]
struct D3Graph<'a> {
    nodes: Vec<D3Node<'a>>,
    links: Vec<D3Link<'a>>,
}

/// Build the D3 node/link arrays from the in-scope services.
fn build_d3_graph<'a>(
    manifest: &'a Manifest,
    in_scope: &std::collections::HashSet<&str>,
) -> D3Graph<'a> {
    let nodes = manifest
        .services
        .iter()
        .filter(|s| in_scope.contains(s.name.as_str()))
        .map(|svc| D3Node {
            id: svc.name.as_str(),
            platform: svc.platform.as_deref().unwrap_or("unknown"),
            team: svc.team.as_deref().unwrap_or(""),
            language: svc.language.as_deref().unwrap_or(""),
        })
        .collect();

    let mut links = Vec::new();
    for svc in manifest
        .services
        .iter()
        .filter(|s| in_scope.contains(s.name.as_str()))
    {
        for dep in &svc.depends_on {
            links.push(D3Link {
                source: svc.name.as_str(),
                target: dep.as_str(),
            });
        }
    }

    D3Graph { nodes, links }
}

/// Render a self-contained HTML file with a D3.js v7 force-directed dependency graph.
///
/// Nodes are coloured by platform; edges represent `depends_on` links.
/// The HTML file has no external CDN dependencies: D3 is embedded via a CDN
/// `<script>` tag so the file requires an internet connection to render.
///
/// Two escaping mechanisms are load-bearing here, the same split
/// `workspace_html.rs` documents for its own report:
///
/// - The D3 mechanics (drag physics, arrow marker, tick handler, and the
///   tooltip's HTML-escaping of untrusted service names) are shared with
///   `workspace_html::render_graph_panel` via [`crate::output::d3_force_graph`]
///   — see that module's docs.
/// - The node/link data itself is embedded inside a `<script>` element, a
///   different trust boundary: HTML-escaping alone does not stop a value
///   containing a literal `</script>` sequence from closing the element
///   early. That data is routed through [`crate::output::json_script::embed`]
///   instead — see that module's docs for why plain `serde_json::to_string`
///   (or, worse, raw `{:?}` Debug-format interpolation) is not sufficient.
pub fn render_html_graph(manifest: &Manifest, team: Option<&str>) -> String {
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

    let graph = build_d3_graph(manifest, &in_scope);
    let graph_json =
        json_script::embed(&graph).expect("D3Node/D3Link are plain strings and always serialize");

    let title = format!(
        "svccat graph - {} service(s)",
        manifest
            .services
            .iter()
            .filter(|s| in_scope.contains(s.name.as_str()))
            .count()
    );

    let script = d3_force_graph::render_script(&D3GraphConfig {
        svg_selector: "#graph",
        tooltip_id: "tooltip",
        arrow_id: "arrow",
        width_expr: "window.innerWidth",
        height_expr: "window.innerHeight - 60",
        color_field: "platform",
        node_radius: 16,
        text_dy: 28,
        link_distance: 120,
        charge_strength: -300,
        collide_radius: 40,
        tooltip_header_expr: "d.id",
        tooltip_fields: &[
            TooltipField {
                label: "platform",
                value_expr: "d.platform",
            },
            TooltipField {
                label: "team",
                value_expr: "d.team || \"-\"",
            },
            TooltipField {
                label: "lang",
                value_expr: "d.language || \"-\"",
            },
        ],
    });

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
<script src="https://cdn.jsdelivr.net/npm/d3@7/dist/d3.min.js"></script>
<style>
  body {{ margin: 0; background: #1a1a2e; color: #eee; font-family: sans-serif; }}
  h1 {{ text-align: center; padding: 1rem 0 0; font-size: 1.1rem; opacity: 0.7; }}
  svg {{ width: 100vw; height: calc(100vh - 60px); }}
  .node circle {{ stroke: #fff; stroke-width: 1.5px; cursor: pointer; }}
  .node text {{ font-size: 11px; fill: #eee; pointer-events: none; }}
  .link {{ stroke: #aaa; stroke-opacity: 0.5; stroke-width: 1.5px; }}
  #tooltip {{
    position: absolute; background: rgba(0,0,0,0.8); color: #fff;
    padding: 8px 12px; border-radius: 4px; font-size: 12px;
    pointer-events: none; display: none;
  }}
</style>
</head>
<body>
<h1>{title}</h1>
<div id="tooltip"></div>
<svg id="graph"></svg>
<script type="application/json" id="graph-data">{graph_json}</script>
<script>
const graphData = JSON.parse(document.getElementById("graph-data").textContent);
const nodes = graphData.nodes;
const links = graphData.links;

{script}
</script>
</body>
</html>
"##,
        title = title,
        graph_json = graph_json,
        script = script,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_id_normalizes_non_alphanumeric_chars() {
        assert_eq!(safe_id("api-gateway.v1"), "api_gateway_v1");
        assert_eq!(safe_id("service/name"), "service_name");
    }

    #[test]
    fn dot_escape_escapes_quotes_and_backslashes() {
        assert_eq!(dot_escape("a\\b"), "a\\\\b");
        assert_eq!(dot_escape("a\"b"), "a\\\"b");
    }

    #[test]
    fn build_label_includes_optional_fields() {
        let svc = crate::manifest::ServiceEntry {
            name: "api".to_string(),
            language: Some("Rust".to_string()),
            role: Some("Gateway".to_string()),
            ..Default::default()
        };
        assert_eq!(build_label(&svc), "api\\nRust\\nGateway");
    }

    #[test]
    fn malicious_service_name_in_graph_data_cannot_close_the_script_tag() {
        // Same vulnerability class PR #6 (commit 07b0485) fixed in
        // workspace_html.rs's D3 data island, and which json_script.rs's own
        // `script_breakout_attempt_is_neutralized` test proves the `embed`
        // helper neutralizes: this renderer used to build
        // nodes_json/links_json via raw `{:?}` Debug-format string
        // interpolation, which does NOT escape `<`, `>`, or `&`. A service,
        // team, platform, or language name containing a literal `</script>`
        // would close the surrounding `<script>` element early and inject
        // live markup into `svccat graph --format html` output.
        let manifest = Manifest {
            services: vec![crate::manifest::ServiceEntry {
                name: "</script><script>alert(1)</script>".to_string(),
                platform: Some("</script><script>alert(2)</script>".to_string()),
                team: Some("</script><script>alert(3)</script>".to_string()),
                language: Some("</script><script>alert(4)</script>".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };

        let html = render_html_graph(&manifest, None);

        // The raw, unescaped payload (as `{:?}` Debug-format interpolation
        // would have produced pre-fix) must not appear anywhere: that is
        // exactly the literal script-close/open sequence that terminates
        // the JSON `<script>` element early and starts a new, live one.
        assert!(
            !html
                .to_lowercase()
                .contains("</script><script>alert"),
            "a raw, unescaped payload must not survive as a literal script-close/open sequence: {html}"
        );

        // The escaped `<`/`>` form is present in the JSON data island for
        // every field, proving each survived intact but inert (mirrors
        // json_script.rs's script_breakout_attempt_is_neutralized and
        // workspace_html.rs's
        // malicious_service_name_in_graph_data_cannot_close_the_script_tag).
        for n in 1..=4 {
            assert!(
                html.contains(&format!("\\u003cscript\\u003ealert({n})")),
                "escaped payload {n} must survive intact in the JSON island: {html}"
            );
        }

        // The dependency graph's own D3 script now reads nodes/links from a
        // parsed JSON island rather than a raw string interpolation.
        assert!(html.contains(r#"id="graph-data">"#));
        assert!(html.contains(r#"JSON.parse(document.getElementById("graph-data").textContent)"#));
    }

    #[test]
    fn graph_data_json_island_round_trips_through_json_parse() {
        // Proves the new embed-based data path actually carries real node
        // and link data end to end, not just that it is "reachable" — the
        // escaping fix must not silently drop or corrupt legitimate data.
        let manifest = Manifest {
            services: vec![
                crate::manifest::ServiceEntry {
                    name: "api".to_string(),
                    platform: Some("Cloud Run".to_string()),
                    depends_on: vec!["db".to_string()],
                    ..Default::default()
                },
                crate::manifest::ServiceEntry {
                    name: "db".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let html = render_html_graph(&manifest, None);
        let marker = r#"id="graph-data">"#;
        let start = html.find(marker).unwrap() + marker.len();
        let end = start + html[start..].find("</script>").unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&html[start..end]).unwrap();

        assert_eq!(parsed["nodes"][0]["id"], "api");
        assert_eq!(parsed["nodes"][0]["platform"], "Cloud Run");
        assert_eq!(parsed["links"][0]["source"], "api");
        assert_eq!(parsed["links"][0]["target"], "db");
    }
}
