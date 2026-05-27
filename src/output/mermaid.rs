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

/// Render a self-contained HTML file with a D3.js v7 force-directed dependency graph.
///
/// Nodes are coloured by platform; edges represent `depends_on` links.
/// The HTML file has no external CDN dependencies: D3 is embedded via a CDN
/// `<script>` tag so the file requires an internet connection to render.
pub fn render_html_graph(manifest: &Manifest, team: Option<&str>) -> String {
    use std::fmt::Write;

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

    // Build JSON node and link arrays for D3.
    let mut nodes_json = String::from("[\n");
    for svc in manifest
        .services
        .iter()
        .filter(|s| in_scope.contains(s.name.as_str()))
    {
        let platform = svc.platform.as_deref().unwrap_or("unknown");
        let team_str = svc.team.as_deref().unwrap_or("");
        let lang_str = svc.language.as_deref().unwrap_or("");
        writeln!(
            nodes_json,
            "  {{\"id\":{id:?},\"platform\":{plat:?},\"team\":{team:?},\"language\":{lang:?}}},",
            id = svc.name,
            plat = platform,
            team = team_str,
            lang = lang_str,
        )
        .unwrap();
    }
    nodes_json.push(']');

    let mut links_json = String::from("[\n");
    for svc in manifest
        .services
        .iter()
        .filter(|s| in_scope.contains(s.name.as_str()))
    {
        for dep in &svc.depends_on {
            writeln!(
                links_json,
                "  {{\"source\":{src:?},\"target\":{tgt:?}}},",
                src = svc.name,
                tgt = dep,
            )
            .unwrap();
        }
    }
    links_json.push(']');

    let title = format!(
        "svccat graph - {} service(s)",
        manifest
            .services
            .iter()
            .filter(|s| in_scope.contains(s.name.as_str()))
            .count()
    );

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
<script>
const nodes = {nodes_json};
const links = {links_json};

// Assign a stable colour per platform using D3 ordinal scale.
const platforms = [...new Set(nodes.map(d => d.platform))];
const colour = d3.scaleOrdinal(d3.schemeTableau10).domain(platforms);

const svg = d3.select("#graph");
const width = window.innerWidth;
const height = window.innerHeight - 60;
svg.attr("viewBox", [0, 0, width, height]);

// Arrow marker
svg.append("defs").append("marker")
  .attr("id", "arrow")
  .attr("viewBox", "0 -5 10 10")
  .attr("refX", 22).attr("refY", 0)
  .attr("markerWidth", 6).attr("markerHeight", 6)
  .attr("orient", "auto")
  .append("path").attr("fill", "#aaa").attr("d", "M0,-5L10,0L0,5");

const sim = d3.forceSimulation(nodes)
  .force("link", d3.forceLink(links).id(d => d.id).distance(120))
  .force("charge", d3.forceManyBody().strength(-300))
  .force("center", d3.forceCenter(width / 2, height / 2))
  .force("collide", d3.forceCollide(40));

const link = svg.append("g")
  .selectAll("line")
  .data(links).join("line")
  .attr("class", "link")
  .attr("marker-end", "url(#arrow)");

const node = svg.append("g")
  .selectAll("g")
  .data(nodes).join("g")
  .attr("class", "node")
  .call(d3.drag()
    .on("start", (e, d) => {{ if (!e.active) sim.alphaTarget(0.3).restart(); d.fx = d.x; d.fy = d.y; }})
    .on("drag",  (e, d) => {{ d.fx = e.x; d.fy = e.y; }})
    .on("end",   (e, d) => {{ if (!e.active) sim.alphaTarget(0); d.fx = null; d.fy = null; }}));

node.append("circle")
  .attr("r", 16)
  .attr("fill", d => colour(d.platform));

node.append("text")
  .attr("dy", 28).attr("text-anchor", "middle")
  .text(d => d.id);

const tip = document.getElementById("tooltip");
node.on("mouseover", (e, d) => {{
  tip.style.display = "block";
  tip.innerHTML = `<b>${{d.id}}</b><br>platform: ${{d.platform}}<br>team: ${{d.team || "-"}}<br>lang: ${{d.language || "-"}}`;
}}).on("mousemove", e => {{
  tip.style.left = (e.pageX + 12) + "px";
  tip.style.top  = (e.pageY - 28) + "px";
}}).on("mouseout", () => {{ tip.style.display = "none"; }});

sim.on("tick", () => {{
  link.attr("x1", d => d.source.x).attr("y1", d => d.source.y)
      .attr("x2", d => d.target.x).attr("y2", d => d.target.y);
  node.attr("transform", d => `translate(${{d.x}},${{d.y}})`);
}});
</script>
</body>
</html>
"##,
        title = title,
        nodes_json = nodes_json,
        links_json = links_json,
    )
}
