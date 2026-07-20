//! Shared D3.js v7 force-directed dependency-graph renderer.
//!
//! Two call sites render an interactive force-directed graph with drag, an
//! arrow-marked link, and a hover tooltip: `svccat graph --format html`
//! ([`crate::output::mermaid::render_html_graph`]) and
//! `svccat workspace check --format html`
//! ([`crate::output::workspace_html::render_graph_panel`]). They differ only
//! in cosmetic, per-call-site ways (panel size, the field nodes are coloured
//! by, tooltip content, a handful of layout constants sized for a full page
//! versus a bounded panel) — every field that varies is named in
//! [`D3GraphConfig`] below and its rationale documented at the call site.
//!
//! The actual D3 mechanics (drag physics, the arrow marker, the simulation
//! tick handler, and — critically — how the hover tooltip escapes untrusted
//! node data) live here ONCE, in [`render_script`], so a future fix to any of
//! them lands for both renderers instead of silently drifting between two
//! independent ~70-line copies.
//!
//! ## Security: the tooltip is the one `innerHTML` sink
//!
//! Node ids, repo/service/platform/team names, and similar fields are
//! untrusted input drawn from repo-controlled manifests. They reach the
//! client already embedded as a JSON literal (via
//! [`crate::output::json_script::embed`] or an equivalent), which by design
//! round-trips through `JSON.parse`/inlining back to the exact original,
//! unescaped bytes — that is what makes the JSON safe: it never contains a
//! literal `<`, `>`, or `&`, so it can't break out of the `<script>` element
//! it lives in.
//!
//! But once that value is back in memory as a plain JS string, it is
//! unescaped HTML again. The hover tooltip in [`render_script`] is the only
//! place either renderer writes such a value into `Element.innerHTML`, so it
//! is the only place a payload like `<img src=x onerror=alert(1)>` can come
//! back to life as markup the moment a viewer hovers that node. The
//! `escHtml` JS helper emitted here is the single, always-applied guard: every
//! tooltip field configured via [`TooltipField`] is wrapped in `escHtml(...)`
//! by this function itself, not left to each call site to remember.

use std::fmt::Write;

/// One label/value line rendered in the hover tooltip below the node id,
/// e.g. `TooltipField { label: "platform", value_expr: "d.platform" }`.
///
/// `value_expr` is a JS expression evaluated against the D3 datum `d` (not
/// the value itself) — it is always passed through the `escHtml` helper
/// before reaching `innerHTML`, so callers cannot opt out of escaping by
/// constructing this struct.
pub(crate) struct TooltipField {
    pub label: &'static str,
    pub value_expr: &'static str,
}

/// Per-call-site configuration for the shared force-graph script. Every
/// field is a documented, intentional difference between the full-page
/// graph and the bounded workspace panel; none of them can disable the
/// tooltip's HTML-escaping.
pub(crate) struct D3GraphConfig {
    /// CSS selector for the `<svg>` element the graph renders into.
    pub svg_selector: &'static str,
    /// Element id of the tooltip `<div>`.
    pub tooltip_id: &'static str,
    /// Arrow marker id. Must be unique if more than one graph could ever
    /// appear in the same document.
    pub arrow_id: &'static str,
    /// JS expression evaluating to the drawing area's width in pixels.
    pub width_expr: &'static str,
    /// JS expression evaluating to the drawing area's height in pixels.
    pub height_expr: &'static str,
    /// Node datum field the colour scale is derived from, e.g. `"platform"`
    /// or `"repo"`.
    pub color_field: &'static str,
    /// Node circle radius.
    pub node_radius: u32,
    /// Node label vertical offset (keeps the text below the circle).
    pub text_dy: u32,
    /// `forceLink` distance.
    pub link_distance: u32,
    /// `forceManyBody` strength (negative repels).
    pub charge_strength: i32,
    /// `forceCollide` radius.
    pub collide_radius: u32,
    /// Tooltip header expression, rendered inside `<b>...</b>` — escaped
    /// like every other tooltip field.
    pub tooltip_header_expr: &'static str,
    /// Remaining tooltip lines, in display order.
    pub tooltip_fields: &'static [TooltipField],
}

/// Render the D3 script body shared by both graph renderers.
///
/// Assumes the caller has already defined JS `const nodes = [...]` and
/// `const links = [...]` immediately before this text, inside the same
/// `<script>` element (both current call sites do this by embedding a JSON
/// island and parsing or inlining it ahead of this output).
pub(crate) fn render_script(cfg: &D3GraphConfig) -> String {
    let mut js = String::new();

    // The one escaping choke point: every dynamic tooltip field is routed
    // through this before it reaches innerHTML.
    js.push_str(
        r#"function escHtml(s) {
  return String(s).replace(/[&<>"']/g, c => ({"&":"&amp;","<":"&lt;",">":"&gt;",'"':"&quot;","'":"&#39;"}[c]));
}

"#,
    );

    writeln!(
        js,
        "const colour = d3.scaleOrdinal(d3.schemeTableau10).domain([...new Set(nodes.map(d => d[{color_field:?}]))]);\n",
        color_field = cfg.color_field
    )
    .unwrap();

    writeln!(
        js,
        "const svg = d3.select({svg_selector:?});",
        svg_selector = cfg.svg_selector
    )
    .unwrap();
    writeln!(js, "const width = {};", cfg.width_expr).unwrap();
    writeln!(js, "const height = {};", cfg.height_expr).unwrap();
    js.push_str("svg.attr(\"viewBox\", [0, 0, width, height]);\n\n");

    writeln!(js, "svg.append(\"defs\").append(\"marker\")").unwrap();
    writeln!(js, "  .attr(\"id\", {arrow_id:?})", arrow_id = cfg.arrow_id).unwrap();
    js.push_str(
        "  .attr(\"viewBox\", \"0 -5 10 10\")\n  .attr(\"refX\", 22).attr(\"refY\", 0)\n  .attr(\"markerWidth\", 6).attr(\"markerHeight\", 6)\n  .attr(\"orient\", \"auto\")\n  .append(\"path\").attr(\"fill\", \"#aaa\").attr(\"d\", \"M0,-5L10,0L0,5\");\n\n",
    );

    writeln!(
        js,
        "const sim = d3.forceSimulation(nodes)\n  .force(\"link\", d3.forceLink(links).id(d => d.id).distance({link_distance}))\n  .force(\"charge\", d3.forceManyBody().strength({charge_strength}))\n  .force(\"center\", d3.forceCenter(width / 2, height / 2))\n  .force(\"collide\", d3.forceCollide({collide_radius}));\n",
        link_distance = cfg.link_distance,
        charge_strength = cfg.charge_strength,
        collide_radius = cfg.collide_radius
    )
    .unwrap();
    js.push('\n');

    writeln!(
        js,
        "const link = svg.append(\"g\")\n  .selectAll(\"line\")\n  .data(links).join(\"line\")\n  .attr(\"class\", \"link\")\n  .attr(\"marker-end\", \"url(#{arrow_id})\");\n",
        arrow_id = cfg.arrow_id
    )
    .unwrap();
    js.push('\n');

    js.push_str(
        "const node = svg.append(\"g\")\n  .selectAll(\"g\")\n  .data(nodes).join(\"g\")\n  .attr(\"class\", \"node\")\n  .call(d3.drag()\n    .on(\"start\", (e, d) => { if (!e.active) sim.alphaTarget(0.3).restart(); d.fx = d.x; d.fy = d.y; })\n    .on(\"drag\",  (e, d) => { d.fx = e.x; d.fy = e.y; })\n    .on(\"end\",   (e, d) => { if (!e.active) sim.alphaTarget(0); d.fx = null; d.fy = null; }));\n\n",
    );

    writeln!(
        js,
        "node.append(\"circle\")\n  .attr(\"r\", {node_radius})\n  .attr(\"fill\", d => colour(d[{color_field:?}]));\n",
        node_radius = cfg.node_radius,
        color_field = cfg.color_field
    )
    .unwrap();
    js.push('\n');

    writeln!(
        js,
        "node.append(\"text\")\n  .attr(\"dy\", {text_dy}).attr(\"text-anchor\", \"middle\")\n  .text(d => d.id);\n",
        text_dy = cfg.text_dy
    )
    .unwrap();
    js.push('\n');

    writeln!(
        js,
        "const tip = document.getElementById({tooltip_id:?});",
        tooltip_id = cfg.tooltip_id
    )
    .unwrap();
    js.push_str("node.on(\"mouseover\", (e, d) => {\n  tip.style.display = \"block\";\n");

    let mut pieces: Vec<String> = Vec::with_capacity(cfg.tooltip_fields.len() + 1);
    pieces.push(format!(
        "\"<b>\" + escHtml({header}) + \"</b>\"",
        header = cfg.tooltip_header_expr
    ));
    for field in cfg.tooltip_fields {
        pieces.push(format!(
            "{label:?} + escHtml({value_expr})",
            label = format!("{}: ", field.label),
            value_expr = field.value_expr
        ));
    }
    writeln!(js, "  tip.innerHTML = {};", pieces.join(" + \"<br>\" + ")).unwrap();

    js.push_str(
        "}).on(\"mousemove\", e => {\n  tip.style.left = (e.pageX + 12) + \"px\";\n  tip.style.top  = (e.pageY - 28) + \"px\";\n}).on(\"mouseout\", () => { tip.style.display = \"none\"; });\n\n",
    );

    js.push_str(
        "sim.on(\"tick\", () => {\n  link.attr(\"x1\", d => d.source.x).attr(\"y1\", d => d.source.y)\n      .attr(\"x2\", d => d.target.x).attr(\"y2\", d => d.target.y);\n  node.attr(\"transform\", d => `translate(${d.x},${d.y})`);\n});\n",
    );

    js
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_cfg() -> D3GraphConfig {
        D3GraphConfig {
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
            ],
        }
    }

    #[test]
    fn every_tooltip_field_is_wrapped_in_esc_html() {
        let script = render_script(&sample_cfg());
        assert!(script.contains("function escHtml(s)"));
        assert!(script.contains("escHtml(d.id)"));
        assert!(script.contains("escHtml(d.platform)"));
        assert!(script.contains(r#"escHtml(d.team || "-")"#));
        // The assignment itself must never contain a raw, unescaped `d.`
        // field reference — every one must be wrapped.
        let assign_line = script
            .lines()
            .find(|l| l.trim_start().starts_with("tip.innerHTML ="))
            .expect("tip.innerHTML assignment present");
        for raw in ["${d.id}", "${d.platform}", "${d.team"] {
            assert!(
                !assign_line.contains(raw),
                "tooltip assignment must not interpolate {raw} unescaped: {assign_line}"
            );
        }
    }

    #[test]
    fn config_values_flow_into_the_script_verbatim() {
        let script = render_script(&sample_cfg());
        assert!(script.contains("d3.select(\"#graph\")"));
        assert!(script.contains("getElementById(\"tooltip\")"));
        assert!(script.contains("url(#arrow)"));
        assert!(script.contains(".attr(\"r\", 16)"));
        assert!(script.contains(".attr(\"dy\", 28)"));
        assert!(script.contains(".distance(120)"));
        assert!(script.contains(".strength(-300)"));
        assert!(script.contains("forceCollide(40)"));
        assert!(script.contains("d[\"platform\"]"));
    }

    #[test]
    fn malicious_label_or_field_values_cannot_reach_innerhtml_unescaped() {
        // The config fields themselves are crate-internal &'static str
        // constants, not attacker input — this test's job is to prove the
        // *mechanism* (every value_expr wrapped in escHtml) holds regardless
        // of how many tooltip fields a call site configures, so a future
        // call site can't accidentally add a field that bypasses escaping.
        let cfg = D3GraphConfig {
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
            ..sample_cfg()
        };
        let script = render_script(&cfg);
        let assign_line = script
            .lines()
            .find(|l| l.trim_start().starts_with("tip.innerHTML ="))
            .expect("tip.innerHTML assignment present");
        for expr in ["d.repo", "d.dependencies", "d.dependents", "d.id"] {
            assert!(
                assign_line.contains(&format!("escHtml({expr})")),
                "{expr} must be wrapped in escHtml on the tooltip assignment: {assign_line}"
            );
        }
    }
}
