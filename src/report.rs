use crate::drift::{DriftItem, DriftReport, Severity};
use crate::manifest::{Manifest, ServiceEntry};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Write;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn group_by_team<'a>(manifest: &'a Manifest) -> BTreeMap<String, Vec<&'a ServiceEntry>> {
    let mut map: BTreeMap<String, Vec<&'a ServiceEntry>> = BTreeMap::new();
    for svc in &manifest.services {
        let team = svc.team.clone().unwrap_or_else(|| "(no team)".to_string());
        map.entry(team).or_default().push(svc);
    }
    map
}

fn drift_by_service(report: &DriftReport) -> HashMap<&str, Vec<&DriftItem>> {
    let mut map: HashMap<&str, Vec<&DriftItem>> = HashMap::new();
    for d in &report.drifts {
        map.entry(d.service.as_str()).or_default().push(d);
    }
    map
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

// ── Markdown renderer ─────────────────────────────────────────────────────────

pub fn render_markdown(manifest: &Manifest, report: &DriftReport) -> String {
    let mut out = String::new();

    let team_count = manifest
        .services
        .iter()
        .filter_map(|s| s.team.as_deref())
        .collect::<HashSet<_>>()
        .len();

    writeln!(out, "# Service Catalog Report").unwrap();
    writeln!(out).unwrap();

    // Summary table
    writeln!(out, "## Summary").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "| Metric | Value |").unwrap();
    writeln!(out, "|--------|-------|").unwrap();
    writeln!(out, "| Services | {} |", manifest.services.len()).unwrap();
    writeln!(out, "| Teams | {} |", team_count).unwrap();
    writeln!(out, "| Drift errors | {} |", report.error_count()).unwrap();
    writeln!(out, "| Drift warnings | {} |", report.warning_count()).unwrap();
    writeln!(out).unwrap();

    // Services by team
    let by_team = group_by_team(manifest);
    let drifts = drift_by_service(report);

    writeln!(out, "## Services by Team").unwrap();
    writeln!(out).unwrap();

    for (team, svcs) in &by_team {
        writeln!(
            out,
            "### {} ({} service{})",
            team,
            svcs.len(),
            plural(svcs.len())
        )
        .unwrap();
        writeln!(out).unwrap();
        writeln!(
            out,
            "| Service | Language | Platform | Role | Oncall | Drift |"
        )
        .unwrap();
        writeln!(
            out,
            "|---------|----------|----------|------|--------|-------|"
        )
        .unwrap();

        for svc in svcs {
            let lang = svc.language.as_deref().unwrap_or("—");
            let platform = svc.platform.as_deref().unwrap_or("—");
            let role = svc.role.as_deref().unwrap_or("—");
            let oncall = svc.oncall.as_deref().unwrap_or("—");

            let items = drifts
                .get(svc.name.as_str())
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let errors: usize = items
                .iter()
                .filter(|d| d.severity == Severity::Error)
                .count();
            let warnings: usize = items
                .iter()
                .filter(|d| d.severity == Severity::Warning)
                .count();

            let status = if errors > 0 {
                format!("❌ {} error(s)", errors)
            } else if warnings > 0 {
                format!("⚠️ {} warning(s)", warnings)
            } else {
                "✅".to_string()
            };

            writeln!(
                out,
                "| {} | {} | {} | {} | {} | {} |",
                svc.name, lang, platform, role, oncall, status
            )
            .unwrap();
        }
        writeln!(out).unwrap();
    }

    // Dependency summary
    let with_deps: Vec<&ServiceEntry> = manifest
        .services
        .iter()
        .filter(|s| !s.depends_on.is_empty())
        .collect();

    if !with_deps.is_empty() {
        writeln!(out, "## Dependency Summary").unwrap();
        writeln!(out).unwrap();
        for svc in with_deps {
            writeln!(out, "- **{}** → {}", svc.name, svc.depends_on.join(", ")).unwrap();
        }
        writeln!(out).unwrap();
    }

    // Drift details
    if !report.drifts.is_empty() {
        writeln!(out, "## Drift Details").unwrap();
        writeln!(out).unwrap();
        for item in &report.drifts {
            let icon = if item.severity == Severity::Error {
                "❌"
            } else {
                "⚠️"
            };
            writeln!(out, "- {} **{}**: {}", icon, item.service, item.message).unwrap();
        }
        writeln!(out).unwrap();
    }

    out
}

// ── HTML renderer ─────────────────────────────────────────────────────────────

pub fn render_html(manifest: &Manifest, report: &DriftReport) -> String {
    let mut body = String::new();

    let team_count = manifest
        .services
        .iter()
        .filter_map(|s| s.team.as_deref())
        .collect::<HashSet<_>>()
        .len();

    body.push_str("<h1>Service Catalog Report</h1>\n");

    // Summary
    body.push_str("<h2>Summary</h2>\n");
    body.push_str("<table><thead><tr><th>Metric</th><th>Value</th></tr></thead><tbody>\n");
    push_tr(&mut body, "Services", &manifest.services.len().to_string());
    push_tr(&mut body, "Teams", &team_count.to_string());
    push_tr(&mut body, "Drift errors", &report.error_count().to_string());
    push_tr(
        &mut body,
        "Drift warnings",
        &report.warning_count().to_string(),
    );
    body.push_str("</tbody></table>\n");

    // Services by team
    let by_team = group_by_team(manifest);
    let drifts = drift_by_service(report);

    body.push_str("<h2>Services by Team</h2>\n");

    for (team, svcs) in &by_team {
        writeln!(
            body,
            "<h3>{} ({} service{})</h3>",
            esc(team),
            svcs.len(),
            plural(svcs.len())
        )
        .unwrap();
        body.push_str("<table><thead><tr><th>Service</th><th>Language</th><th>Platform</th><th>Role</th><th>Oncall</th><th>Drift</th></tr></thead><tbody>\n");

        for svc in svcs {
            let lang = svc.language.as_deref().unwrap_or("—");
            let platform = svc.platform.as_deref().unwrap_or("—");
            let role = svc.role.as_deref().unwrap_or("—");
            let oncall = svc.oncall.as_deref().unwrap_or("—");

            let items = drifts
                .get(svc.name.as_str())
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let errors: usize = items
                .iter()
                .filter(|d| d.severity == Severity::Error)
                .count();
            let warnings: usize = items
                .iter()
                .filter(|d| d.severity == Severity::Warning)
                .count();

            let status = if errors > 0 {
                format!("❌ {} error(s)", errors)
            } else if warnings > 0 {
                format!("⚠️ {} warning(s)", warnings)
            } else {
                "✅".to_string()
            };

            writeln!(
                body,
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                esc(&svc.name),
                esc(lang),
                esc(platform),
                esc(role),
                esc(oncall),
                esc(&status)
            )
            .unwrap();
        }
        body.push_str("</tbody></table>\n");
    }

    // Dependency summary
    let with_deps: Vec<&ServiceEntry> = manifest
        .services
        .iter()
        .filter(|s| !s.depends_on.is_empty())
        .collect();

    if !with_deps.is_empty() {
        body.push_str("<h2>Dependency Summary</h2>\n<ul>\n");
        for svc in with_deps {
            writeln!(
                body,
                "<li><strong>{}</strong> → {}</li>",
                esc(&svc.name),
                esc(&svc.depends_on.join(", "))
            )
            .unwrap();
        }
        body.push_str("</ul>\n");
    }

    // Drift details
    if !report.drifts.is_empty() {
        body.push_str("<h2>Drift Details</h2>\n<ul>\n");
        for item in &report.drifts {
            let icon = if item.severity == Severity::Error {
                "❌"
            } else {
                "⚠️"
            };
            writeln!(
                body,
                "<li>{} <strong>{}</strong>: {}</li>",
                icon,
                esc(&item.service),
                esc(&item.message)
            )
            .unwrap();
        }
        body.push_str("</ul>\n");
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Service Catalog Report</title>
<style>
  body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
         max-width: 1100px; margin: 40px auto; padding: 0 20px; color: #24292e; line-height: 1.5; }}
  h1 {{ border-bottom: 2px solid #e1e4e8; padding-bottom: 10px; }}
  h2 {{ border-bottom: 1px solid #e1e4e8; padding-bottom: 6px; margin-top: 2em; }}
  h3 {{ color: #586069; margin-top: 1.5em; }}
  table {{ border-collapse: collapse; width: 100%; margin-bottom: 1.5em; }}
  th {{ background: #f6f8fa; font-weight: 600; }}
  th, td {{ border: 1px solid #d1d5da; padding: 8px 14px; text-align: left; }}
  tr:nth-child(even) td {{ background: #fafbfc; }}
  ul {{ padding-left: 1.8em; }}
  li {{ margin-bottom: 4px; }}
  strong {{ font-weight: 600; }}
</style>
</head>
<body>
{body}
</body>
</html>"#
    )
}

fn push_tr(buf: &mut String, label: &str, value: &str) {
    writeln!(buf, "<tr><td>{}</td><td>{}</td></tr>", label, value).unwrap();
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
