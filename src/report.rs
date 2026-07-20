use crate::discovery::DiscoveredService;
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

/// Base stylesheet shared by every self-contained HTML report svccat emits
/// (the single-repo report here, and the multi-repo workspace report in
/// [`crate::output::workspace_html`]). Kept as one constant so the two reports
/// read as one visual family instead of drifting apart independently.
pub(crate) const REPORT_STYLE: &str = r#"
  body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
         max-width: 1100px; margin: 40px auto; padding: 0 20px; color: #24292e; line-height: 1.5; }
  h1 { border-bottom: 2px solid #e1e4e8; padding-bottom: 10px; }
  h2 { border-bottom: 1px solid #e1e4e8; padding-bottom: 6px; margin-top: 2em; }
  h3 { color: #586069; margin-top: 1.5em; }
  table { border-collapse: collapse; width: 100%; margin-bottom: 1.5em; }
  th { background: #f6f8fa; font-weight: 600; }
  th, td { border: 1px solid #d1d5da; padding: 8px 14px; text-align: left; }
  tr:nth-child(even) td { background: #fafbfc; }
  ul { padding-left: 1.8em; }
  li { margin-bottom: 4px; }
  strong { font-weight: 600; }
"#;

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
{REPORT_STYLE}
</style>
</head>
<body>
{body}
</body>
</html>"#
    )
}

pub(crate) fn push_tr(buf: &mut String, label: &str, value: &str) {
    writeln!(buf, "<tr><td>{}</td><td>{}</td></tr>", label, value).unwrap();
}

/// HTML-escape a string for safe embedding in HTML text content or a
/// double-quoted attribute value.
///
/// This is the escaping mechanism for every repo-sourced string (service,
/// team, and repo names; drift messages) that lands in an HTML report:
/// [`render_html`] here and [`crate::output::workspace_html::render_html`]
/// both route every such string through this function before writing it into
/// the document. It is a manual implementation (no external crate) matching
/// the four characters meaningful to an HTML parser in text/attribute
/// position; embedding untrusted data inside a `<script>` element instead
/// needs different handling — see [`crate::output::json_script`].
pub(crate) fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ── History renderer ──────────────────────────────────────────────────────────

/// Render a Markdown table showing drift evolution across the last `n` git commits.
///
/// Uses `git log` to enumerate commits, loads the manifest at each commit via
/// `git show`, then runs drift analysis against the current discovered services.
pub fn render_history_markdown(
    root: &std::path::Path,
    manifest_path: &std::path::Path,
    discovered: &[DiscoveredService],
    n: usize,
) -> anyhow::Result<String> {
    let log_output = std::process::Command::new("git")
        .args([
            "-C",
            &root.to_string_lossy(),
            "log",
            "--format=%H %s",
            &format!("-{}", n),
        ])
        .output()?;

    if !log_output.status.success() {
        anyhow::bail!(
            "git log failed: {}",
            String::from_utf8_lossy(&log_output.stderr).trim()
        );
    }

    let log = String::from_utf8(log_output.stdout)?;
    let commits: Vec<(String, String)> = log
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| {
            let (hash, msg) = l.split_once(' ').unwrap_or((l, ""));
            (hash.to_string(), msg.to_string())
        })
        .collect();

    if commits.is_empty() {
        anyhow::bail!("no git commits found");
    }

    let mut out = String::new();
    writeln!(
        out,
        "## Drift History (last {} commit{})",
        commits.len(),
        plural(commits.len())
    )
    .unwrap();
    writeln!(out).unwrap();
    writeln!(out, "| Commit | Summary | Errors | Warnings | Total |").unwrap();
    writeln!(out, "|--------|---------|--------|----------|-------|").unwrap();

    for (hash, summary) in &commits {
        let short = &hash[..hash.len().min(7)];
        // Truncate summary at 60 chars (character-safe)
        let display_summary: String = summary.chars().take(60).collect();
        let display_summary = if summary.chars().count() > 60 {
            format!("{}…", display_summary)
        } else {
            display_summary
        };

        match crate::since::load_at_ref(root, manifest_path, hash) {
            Ok(m) => {
                let r = crate::drift::analyze(&m, discovered, root);
                let errors = r.error_count();
                let warnings = r.warning_count();
                let total = r.drifts.len();
                let status = if errors > 0 {
                    "❌"
                } else if warnings > 0 {
                    "⚠️"
                } else {
                    "✅"
                };
                writeln!(
                    out,
                    "| `{}` | {} | {} | {} | {} {} |",
                    short, display_summary, errors, warnings, status, total
                )
                .unwrap();
            }
            Err(_) => {
                // Manifest may not exist at older commits — mark as unavailable.
                writeln!(out, "| `{}` | {} | — | — | — |", short, display_summary).unwrap();
            }
        }
    }

    Ok(out)
}

// ── Badge renderer ────────────────────────────────────────────────────────────

/// Return a Shields.io badge URL reflecting the current drift status.
///
/// - 0 errors, 0 warnings → `brightgreen`, label `drift: clean`
/// - 0 errors, N warnings → `yellow`, label `N warning(s)`
/// - N errors             → `red`, label `N error(s)`
fn badge_url(report: &DriftReport) -> String {
    let errors = report.error_count();
    let warnings = report.warning_count();

    let (label, color) = if errors > 0 {
        (
            format!("{} error{}", errors, if errors == 1 { "" } else { "s" }),
            "red",
        )
    } else if warnings > 0 {
        (
            format!(
                "{} warning{}",
                warnings,
                if warnings == 1 { "" } else { "s" }
            ),
            "yellow",
        )
    } else {
        ("drift: clean".to_string(), "brightgreen")
    };

    // Shields.io static badge: /badge/<left>-<right>-<color>
    // Spaces encoded as %20, colon as %3A
    let encoded_label = label.replace(' ', "%20").replace(':', "%3A");
    format!(
        "https://img.shields.io/badge/svccat-{}-{}",
        encoded_label, color
    )
}

/// Render a Markdown badge snippet for embedding in a README.
///
/// Output example:
/// ```text
/// [![svccat drift: clean](https://img.shields.io/badge/...)](https://crates.io/crates/svccat)
/// ```
pub fn render_badge(report: &DriftReport) -> String {
    let errors = report.error_count();
    let warnings = report.warning_count();
    let alt = if errors > 0 {
        format!(
            "svccat: {} error{}",
            errors,
            if errors == 1 { "" } else { "s" }
        )
    } else if warnings > 0 {
        format!(
            "svccat: {} warning{}",
            warnings,
            if warnings == 1 { "" } else { "s" }
        )
    } else {
        "svccat: drift clean".to_string()
    };

    format!(
        "[![{}]({})](https://crates.io/crates/svccat)",
        alt,
        badge_url(report)
    )
}

/// Render the full ownership and drift report as machine-readable JSON.
pub fn render_json(manifest: &Manifest, report: &DriftReport) -> anyhow::Result<String> {
    let team_groups: BTreeMap<String, Vec<serde_json::Value>> = {
        let by_team = group_by_team(manifest);
        let drift_map = drift_by_service(report);
        by_team
            .into_iter()
            .map(|(team, svcs)| {
                let entries: Vec<serde_json::Value> = svcs
                    .iter()
                    .map(|svc| {
                        let drifts: Vec<serde_json::Value> = drift_map
                            .get(svc.name.as_str())
                            .map(|items| {
                                items
                                    .iter()
                                    .map(|d| {
                                        serde_json::json!({
                                            "severity": format!("{:?}", d.severity).to_lowercase(),
                                            "kind": format!("{:?}", d.kind),
                                            "message": d.message,
                                        })
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        serde_json::json!({
                            "name": svc.name,
                            "language": svc.language,
                            "platform": svc.platform,
                            "role": svc.role,
                            "url": svc.url,
                            "oncall": svc.oncall,
                            "drift": drifts,
                        })
                    })
                    .collect();
                (team, entries)
            })
            .collect()
    };

    let out = serde_json::json!({
        "manifest": report.manifest,
        "summary": {
            "declared": report.declared,
            "discovered": report.discovered,
            "errors": report.error_count(),
            "warnings": report.warning_count(),
        },
        "teams": team_groups,
    });

    Ok(serde_json::to_string_pretty(&out)?)
}
