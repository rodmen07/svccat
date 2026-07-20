//! Self-contained HTML report for `workspace check --format html`.
//!
//! Multi-repo slice 3: this is the "Later" candidate named in
//! `ROADMAP.md` and originally sketched in `docs/RELEASE_PLAN_V1.4.0.md`
//! item 4 (`svccat workspace check --format html`) — a single HTML view
//! covering every repo in the workspace plus the cross-repo dependency
//! topology.
//!
//! Two escaping mechanisms are load-bearing here, matching the two contexts
//! repo-sourced text lands in:
//!
//! - Plain HTML text and attributes (repo/service/team names, drift
//!   messages) go through [`crate::report::esc`], the same manual
//!   `&`/`<`/`>`/`"` escaper the single-repo HTML report already uses.
//! - The dependency graph's node and link data is embedded inside a
//!   `<script>` element, a different trust boundary: HTML-escaping alone does
//!   not stop a value containing a literal `</script>` sequence from closing
//!   the element early. That data is routed through
//!   [`crate::output::json_script::embed`] instead — see that module's docs
//!   for why plain `serde_json::to_string` is not sufficient on its own.
//!
//! The dependency graph itself shares its D3.js v7 force-directed mechanics
//! (nodes, links, drag, tick handler, tooltip-on-hover) with
//! `svccat graph --format html` (`crate::output::mermaid::render_html_graph`)
//! via [`crate::output::d3_force_graph`], restyled to sit inside a bounded
//! panel within this report rather than fill the whole page, and colouring
//! nodes by repo instead of by platform (workspace `GraphNode`s don't carry
//! per-service platform/language metadata the way a single repo's
//! `Manifest` does). See that module's docs for why the tooltip is the one
//! place untrusted node data must be re-escaped before it reaches
//! `innerHTML`, even though it already passed through the JSON-safe
//! [`json_script::embed`] on the way to the client.

use crate::deps_graph::GraphNode;
use crate::drift::Severity;
use crate::output::d3_force_graph::{self, D3GraphConfig, TooltipField};
use crate::output::json_script;
use crate::report::{esc, push_tr, REPORT_STYLE};
use crate::workspace::WorkspaceDriftReport;
use std::collections::HashSet;
use std::fmt::Write;

/// Extra styling for elements the single-repo report doesn't have: the
/// per-repo path caption and the bounded dependency-graph panel.
const WORKSPACE_STYLE: &str = r#"
  .muted { color: #6a737d; font-weight: normal; font-size: 0.85em; }
  tr.err td { background: #ffeef0; }
  tr.warn td { background: #fffbdd; }
  #graph-panel { border: 1px solid #d1d5da; border-radius: 6px; overflow: hidden;
                 background: #1a1a2e; margin-bottom: 1.5em; }
  #graph-panel svg { width: 100%; height: 480px; display: block; }
  #graph-panel .node circle { stroke: #fff; stroke-width: 1.5px; cursor: pointer; }
  #graph-panel .node text { font-size: 11px; fill: #eee; pointer-events: none; }
  #graph-panel .link { stroke: #aaa; stroke-opacity: 0.5; stroke-width: 1.5px; }
  #graph-tooltip { position: absolute; background: rgba(0,0,0,0.85); color: #fff;
                   padding: 8px 12px; border-radius: 4px; font-size: 12px;
                   pointer-events: none; display: none; }
"#;

/// Render the workspace drift report as a self-contained HTML document.
pub fn render_html(report: &WorkspaceDriftReport) -> String {
    let mut body = String::new();

    let title = match &report.workspace_name {
        Some(name) => format!("Workspace Catalog Report — {name}"),
        None => "Workspace Catalog Report".to_string(),
    };
    writeln!(body, "<h1>{}</h1>", esc(&title)).unwrap();

    render_summary(&mut body, report);
    render_repos(&mut body, report);
    render_dependencies(&mut body, report);

    wrap_document(&title, &body)
}

fn render_summary(body: &mut String, report: &WorkspaceDriftReport) {
    body.push_str("<h2>Summary</h2>\n");
    body.push_str("<table><thead><tr><th>Metric</th><th>Value</th></tr></thead><tbody>\n");
    push_tr(body, "Repositories", &report.repos.len().to_string());
    push_tr(
        body,
        "Declared services",
        &report.total_declared.to_string(),
    );
    push_tr(
        body,
        "Discovered services",
        &report.total_discovered.to_string(),
    );
    push_tr(body, "Errors", &report.total_errors.to_string());
    push_tr(body, "Warnings", &report.total_warnings.to_string());
    body.push_str("</tbody></table>\n");
}

fn render_repos(body: &mut String, report: &WorkspaceDriftReport) {
    body.push_str("<h2>Repositories</h2>\n");

    for analysis in &report.repos {
        writeln!(
            body,
            "<h3>{} <span class=\"muted\">({})</span></h3>",
            esc(&analysis.name),
            esc(&analysis.path.display().to_string())
        )
        .unwrap();

        writeln!(
            body,
            "<p>{} declared, {} discovered — {} error(s), {} warning(s)</p>",
            analysis.drift.declared,
            analysis.drift.discovered,
            analysis.drift.error_count(),
            analysis.drift.warning_count(),
        )
        .unwrap();

        if analysis.drift.drifts.is_empty() {
            continue;
        }

        body.push_str(
            "<table><thead><tr><th>Severity</th><th>Kind</th><th>Service</th><th>Message</th></tr></thead><tbody>\n",
        );
        for item in &analysis.drift.drifts {
            let (row_class, icon) = match item.severity {
                Severity::Error => ("err", "❌ error"),
                Severity::Warning => ("warn", "⚠️ warning"),
            };
            writeln!(
                body,
                "<tr class=\"{row_class}\"><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                esc(icon),
                esc(&format!("{:?}", item.kind)),
                esc(&item.service),
                esc(&item.message)
            )
            .unwrap();
        }
        body.push_str("</tbody></table>\n");
    }
}

fn render_dependencies(body: &mut String, report: &WorkspaceDriftReport) {
    let Some(summary) = &report.dependency_summary else {
        return;
    };

    body.push_str("<h2>Dependency Analysis</h2>\n");
    body.push_str("<table><thead><tr><th>Metric</th><th>Value</th></tr></thead><tbody>\n");
    push_tr(body, "Total services", &summary.total_services.to_string());
    push_tr(
        body,
        "Services with dependencies",
        &summary.services_with_dependencies.to_string(),
    );
    push_tr(
        body,
        "Total dependencies",
        &summary.total_dependencies.to_string(),
    );
    push_tr(
        body,
        "Cross-repo dependencies",
        &summary.cross_repo_dependencies.to_string(),
    );
    push_tr(
        body,
        "Circular dependencies",
        &summary.circular_dependencies.to_string(),
    );
    push_tr(
        body,
        "Unresolvable dependencies",
        &summary.unresolvable_dependencies.to_string(),
    );
    body.push_str("</tbody></table>\n");

    if !report.circular_dependencies.is_empty() {
        body.push_str("<h3>⚠️ Circular Dependencies</h3>\n<ul>\n");
        for c in &report.circular_dependencies {
            writeln!(body, "<li>{}</li>", esc(&c.description)).unwrap();
        }
        body.push_str("</ul>\n");
    }

    if !report.unresolvable_dependencies.is_empty() {
        body.push_str("<h3>❌ Unresolvable Dependencies</h3>\n<ul>\n");
        for u in &report.unresolvable_dependencies {
            writeln!(
                body,
                "<li><strong>{}</strong> depends on <strong>{}</strong> — {}</li>",
                esc(&u.service.to_string()),
                esc(&u.dependency.to_string()),
                esc(&u.reason)
            )
            .unwrap();
        }
        body.push_str("</ul>\n");
    }

    if !report.dependency_graph_nodes.is_empty() {
        body.push_str("<h2>Dependency Graph</h2>\n");
        body.push_str(&render_graph_panel(&report.dependency_graph_nodes));
    }
}

/// A D3 node: one entry per cross-repo `GraphNode`, identified by the
/// `repo:service` composite key so same-named services in different repos
/// never collide into a single node.
#[derive(serde::Serialize)]
struct D3Node<'a> {
    id: String,
    repo: &'a str,
    service: &'a str,
    dependencies: usize,
    dependents: usize,
}

#[derive(serde::Serialize)]
struct D3Link {
    source: String,
    target: String,
}

#[derive(serde::Serialize)]
struct D3Graph<'a> {
    nodes: Vec<D3Node<'a>>,
    links: Vec<D3Link>,
}

/// Build the D3 node/link arrays from the cross-repo dependency graph.
///
/// A dependency edge whose target isn't itself a known node (an unresolvable
/// dependency — already surfaced in its own report section) is dropped here
/// rather than passed to D3, which errors at runtime if `forceLink` is given
/// a link referencing a node id that doesn't exist.
fn build_d3_graph(nodes: &[GraphNode]) -> D3Graph<'_> {
    let known: HashSet<String> = nodes.iter().map(|n| n.key.to_string()).collect();

    let d3_nodes = nodes
        .iter()
        .map(|n| D3Node {
            id: n.key.to_string(),
            repo: n.key.repo.as_str(),
            service: n.key.service.as_str(),
            dependencies: n.dependencies.len(),
            dependents: n.dependents.len(),
        })
        .collect();

    let mut d3_links = Vec::new();
    for n in nodes {
        let source = n.key.to_string();
        for edge in &n.dependencies {
            let target = edge.target.to_string();
            if known.contains(&target) {
                d3_links.push(D3Link {
                    source: source.clone(),
                    target,
                });
            }
        }
    }

    D3Graph {
        nodes: d3_nodes,
        links: d3_links,
    }
}

/// Render the bounded D3 force-directed graph panel, or nothing if the graph
/// data fails to serialize (defensive; every field here is a plain String).
///
/// Node radius, link distance, charge strength, and collide radius are
/// intentionally smaller than `mermaid::render_html_graph`'s full-page
/// graph: this panel is bounded to 480px tall rather than filling the
/// viewport, so a tighter layout avoids nodes crowding the edges. The
/// mechanics that must stay identical between the two renderers (drag
/// physics, the arrow marker, the tick handler, and the tooltip's
/// HTML-escaping) live in [`crate::output::d3_force_graph`] instead of being
/// copied here.
fn render_graph_panel(nodes: &[GraphNode]) -> String {
    let graph = build_d3_graph(nodes);
    let Ok(graph_json) = json_script::embed(&graph) else {
        return String::new();
    };

    let script = d3_force_graph::render_script(&D3GraphConfig {
        svg_selector: "#workspace-graph",
        tooltip_id: "graph-tooltip",
        arrow_id: "wg-arrow",
        width_expr: "panel.clientWidth || 800",
        height_expr: "480",
        color_field: "repo",
        node_radius: 14,
        text_dy: 26,
        link_distance: 110,
        charge_strength: -260,
        collide_radius: 36,
        tooltip_header_expr: "d.id",
        tooltip_fields: &[
            TooltipField {
                label: "repo",
                value_expr: "d.repo",
            },
            TooltipField {
                label: "dependencies",
                value_expr: "d.dependencies",
            },
            TooltipField {
                label: "dependents",
                value_expr: "d.dependents",
            },
        ],
    });

    format!(
        r##"<div id="graph-panel">
<div id="graph-tooltip"></div>
<svg id="workspace-graph"></svg>
</div>
<script src="https://cdn.jsdelivr.net/npm/d3@7/dist/d3.min.js"></script>
<script type="application/json" id="workspace-graph-data">{graph_json}</script>
<script>
const graphData = JSON.parse(document.getElementById("workspace-graph-data").textContent);
const nodes = graphData.nodes;
const links = graphData.links;
const panel = document.getElementById("graph-panel");

{script}
</script>
"##
    )
}

fn wrap_document(title: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{escaped_title}</title>
<style>
{REPORT_STYLE}
{WORKSPACE_STYLE}
</style>
</head>
<body>
{body}
</body>
</html>"#,
        escaped_title = esc(title),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deps_graph::{DependencyEdge, ServiceKey};
    use crate::drift::DriftReport;
    use crate::workspace::RepositoryAnalysis;
    use std::path::PathBuf;

    fn node(repo: &str, service: &str, deps: &[(&str, &str)]) -> GraphNode {
        GraphNode {
            key: ServiceKey {
                repo: repo.to_string(),
                service: service.to_string(),
            },
            version: None,
            dependencies: deps
                .iter()
                .map(|(r, s)| DependencyEdge {
                    target: ServiceKey {
                        repo: r.to_string(),
                        service: s.to_string(),
                    },
                    version_constraint: None,
                })
                .collect(),
            dependents: Vec::new(),
        }
    }

    fn sample_report() -> WorkspaceDriftReport {
        WorkspaceDriftReport {
            workspace_name: Some("Platform".to_string()),
            repos: vec![RepositoryAnalysis {
                name: "api-repo".to_string(),
                path: PathBuf::from("api-repo"),
                drift: DriftReport {
                    manifest: "services.yaml".to_string(),
                    declared: 1,
                    discovered: 1,
                    drifts: Vec::new(),
                },
            }],
            total_declared: 1,
            total_discovered: 1,
            total_errors: 0,
            total_warnings: 0,
            dependency_summary: None,
            circular_dependencies: Vec::new(),
            unresolvable_dependencies: Vec::new(),
            dependency_graph_nodes: Vec::new(),
        }
    }

    #[test]
    fn renders_a_self_contained_document_with_summary() {
        let html = render_html(&sample_report());
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Platform"));
        assert!(html.contains("api-repo"));
        assert!(html.contains("<table>"));
    }

    #[test]
    fn no_dependency_section_when_summary_is_absent() {
        let html = render_html(&sample_report());
        assert!(!html.contains("Dependency Analysis"));
        assert!(!html.contains("d3@7"));
    }

    #[test]
    fn dependency_graph_panel_appears_when_nodes_are_present() {
        let mut report = sample_report();
        report.dependency_summary = Some(crate::deps_graph::DependencySummary {
            total_services: 2,
            services_with_dependencies: 1,
            total_dependencies: 1,
            cross_repo_dependencies: 1,
            circular_dependencies: 0,
            unresolvable_dependencies: 0,
        });
        report.dependency_graph_nodes = vec![
            node("api-repo", "api", &[("web-repo", "web")]),
            node("web-repo", "web", &[]),
        ];

        let html = render_html(&report);
        assert!(html.contains("Dependency Graph"));
        assert!(html.contains("d3@7"));
        assert!(html.contains("workspace-graph-data"));
        // Node ids are namespaced by repo so same-named services can't collide.
        assert!(html.contains("api-repo:api"));
        assert!(html.contains("web-repo:web"));
    }

    #[test]
    fn links_to_unresolvable_targets_are_dropped_not_passed_to_d3() {
        // "api" depends on "web-repo:missing", which has no corresponding
        // node. build_d3_graph must drop that edge rather than emit a link
        // referencing a nonexistent node id (which would error in D3 at
        // runtime).
        let nodes = vec![node("api-repo", "api", &[("web-repo", "missing")])];
        let graph = build_d3_graph(&nodes);
        assert_eq!(graph.nodes.len(), 1);
        assert!(graph.links.is_empty());
    }

    #[test]
    fn malicious_repo_and_service_names_render_as_inert_text() {
        // A repo owner controls svccat.toml and services.yaml, but that text
        // is still untrusted from the report's point of view: a repo or
        // service name shaped like a script tag must never become live
        // markup in the generated HTML.
        let mut report = sample_report();
        report.workspace_name = Some("<script>alert(1)</script>".to_string());
        report.repos[0].name = "<img src=x onerror=alert(2)>".to_string();
        report.repos[0].drift.drifts.push(crate::drift::DriftItem {
            kind: crate::drift::DriftKind::UndeclaredInRepo,
            severity: Severity::Warning,
            service: "\"><script>alert(3)</script>".to_string(),
            message: "<b>bold drift</b> & \"quoted\"".to_string(),
            detail: None,
        });

        let html = render_html(&report);

        // No literal, unescaped script/markup breakout survives anywhere in
        // the document.
        assert!(
            !html.contains("<script>alert"),
            "a malicious name must not become a live <script> tag: {html}"
        );
        assert!(
            !html.contains("<img src=x onerror"),
            "a malicious name must not become a live <img> tag: {html}"
        );
        assert!(
            !html.contains("\"><script>"),
            "a malicious name must not break out of an attribute: {html}"
        );

        // The escaped, inert form is present instead: entities for the plain
        // HTML-text contexts.
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("&lt;img src=x onerror=alert(2)&gt;"));
        assert!(html.contains("&lt;b&gt;bold drift&lt;/b&gt; &amp; &quot;quoted&quot;"));
    }

    #[test]
    fn malicious_service_name_in_graph_data_cannot_close_the_script_tag() {
        // Same trust boundary, but this time the malicious text flows through
        // the D3 <script> data island rather than plain HTML text. A naive
        // `serde_json::to_string` embed would let a service named
        // `</script><script>...` terminate the JSON script element early.
        let mut report = sample_report();
        report.dependency_summary = Some(crate::deps_graph::DependencySummary {
            total_services: 1,
            services_with_dependencies: 0,
            total_dependencies: 0,
            cross_repo_dependencies: 0,
            circular_dependencies: 0,
            unresolvable_dependencies: 0,
        });
        report.dependency_graph_nodes =
            vec![node("api-repo", "</script><script>alert(4)</script>", &[])];

        let html = render_html(&report);

        assert!(
            !html.to_lowercase().contains("alert(4)</script><script>"),
            "the payload must not produce a literal script-close sequence: {html}"
        );
        // The escaped < / > form is present in the JSON data
        // island, proving the name survived intact but inert.
        assert!(html.contains("\\u003cscript\\u003ealert(4)"));
    }
}
