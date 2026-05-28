use crate::manifest::{Manifest, ServiceEntry};
use colored::Colorize;

// ── Query parsing ─────────────────────────────────────────────────────────────

/// A parsed search query.
///
/// Supports two forms:
/// - `field:value` - match services where `field` contains `value` (case-insensitive)
/// - `value`       - match services where `name` or any string field contains `value`
#[derive(Debug, Clone)]
pub enum Query {
    FieldValue { field: String, value: String },
    AnyField(String),
}

impl Query {
    pub fn parse(raw: &str) -> Self {
        if let Some((field, value)) = raw.split_once(':') {
            let field = field.trim().to_lowercase();
            let value = value.trim().to_lowercase();
            if !field.is_empty() && !value.is_empty() {
                return Query::FieldValue { field, value };
            }
        }
        Query::AnyField(raw.trim().to_lowercase())
    }
}

// ── Search ────────────────────────────────────────────────────────────────────

fn service_field<'a>(svc: &'a ServiceEntry, field: &str) -> Option<&'a str> {
    match field {
        "name" => Some(svc.name.as_str()),
        "language" | "lang" => svc.language.as_deref(),
        "platform" => svc.platform.as_deref(),
        "url" => svc.url.as_deref(),
        "role" => svc.role.as_deref(),
        "team" => svc.team.as_deref(),
        "oncall" => svc.oncall.as_deref(),
        "docs" => svc.docs.as_deref(),
        "ci" => svc.ci.as_deref(),
        "path" => svc.path.as_deref(),
        "tags" => None, // tags are Vec<String>, handled separately
        _ => None,
    }
}

fn service_tags(svc: &ServiceEntry) -> &[String] {
    &svc.tags
}

fn matches_query(svc: &ServiceEntry, query: &Query) -> bool {
    match query {
        Query::FieldValue { field, value } => {
            if field == "tags" || field == "tag" {
                return service_tags(svc)
                    .iter()
                    .any(|t| t.to_lowercase().contains(value.as_str()));
            }
            if let Some(v) = service_field(svc, field) {
                return v.to_lowercase().contains(value.as_str());
            }
            false
        }
        Query::AnyField(term) => {
            // Check all string fields
            let fields = [
                Some(svc.name.as_str()),
                svc.language.as_deref(),
                svc.platform.as_deref(),
                svc.url.as_deref(),
                svc.role.as_deref(),
                svc.team.as_deref(),
                svc.oncall.as_deref(),
                svc.docs.as_deref(),
                svc.ci.as_deref(),
                svc.path.as_deref(),
            ];
            if fields
                .iter()
                .filter_map(|f| *f)
                .any(|v| v.to_lowercase().contains(term.as_str()))
            {
                return true;
            }
            // Also check tags and depends_on
            if service_tags(svc)
                .iter()
                .any(|t| t.to_lowercase().contains(term.as_str()))
            {
                return true;
            }
            svc.depends_on
                .iter()
                .any(|d| d.to_lowercase().contains(term.as_str()))
        }
    }
}

/// Return all services in the manifest that match `query`.
pub fn run<'a>(manifest: &'a Manifest, query: &Query) -> Vec<&'a ServiceEntry> {
    manifest
        .services
        .iter()
        .filter(|svc| matches_query(svc, query))
        .collect()
}

// ── Renderer ──────────────────────────────────────────────────────────────────

pub fn render(matches: &[&ServiceEntry], query_raw: &str, total: usize) {
    println!(
        "{} {} match{} for {:?} (searched {} service{})",
        if matches.is_empty() {
            "0".yellow().to_string()
        } else {
            matches.len().to_string().green().bold().to_string()
        },
        if matches.len() == 1 {
            "result"
        } else {
            "results"
        },
        if matches.len() == 1 { "" } else { "es" },
        query_raw,
        total,
        if total == 1 { "" } else { "s" }
    );

    if matches.is_empty() {
        return;
    }

    println!();

    for svc in matches {
        let tags_str = if svc.tags.is_empty() {
            String::new()
        } else {
            format!("  [{}]", svc.tags.join(", ").dimmed())
        };
        println!("  {}{}", svc.name.bold(), tags_str);

        let mut meta: Vec<String> = Vec::new();
        if let Some(t) = &svc.team {
            meta.push(format!("team:{}", t));
        }
        if let Some(l) = &svc.language {
            meta.push(format!("lang:{}", l));
        }
        if let Some(p) = &svc.platform {
            meta.push(format!("platform:{}", p));
        }
        if let Some(r) = &svc.role {
            meta.push(format!("role:{}", r));
        }
        if !svc.depends_on.is_empty() {
            meta.push(format!("deps:{}", svc.depends_on.join(",")));
        }
        if !meta.is_empty() {
            println!("    {}", meta.join("  ").dimmed());
        }
    }
}

/// Render search results as JSON - used when writing to `--output`.
pub fn render_json(
    matches: &[&ServiceEntry],
    query_raw: &str,
    total: usize,
) -> anyhow::Result<String> {
    let json = serde_json::json!({
        "query": query_raw,
        "total_searched": total,
        "match_count": matches.len(),
        "matches": matches.iter().map(|svc| serde_json::json!({
            "name": svc.name,
            "team": svc.team,
            "language": svc.language,
            "platform": svc.platform,
            "role": svc.role,
            "oncall": svc.oncall,
            "url": svc.url,
            "tags": svc.tags,
            "depends_on": svc.depends_on,
            "path": svc.path,
        })).collect::<Vec<_>>(),
    });
    Ok(serde_json::to_string_pretty(&json)?)
}
