use crate::manifest::{Manifest, ServiceEntry};
use colored::Colorize;

// ── The searchable-field vocabulary ───────────────────────────────────────────

/// Every field name a `field:value` query accepts, in the order the `Search`
/// doc comment in `src/cli.rs` lists them — that doc comment is what
/// `svccat search --help` prints, so the two must agree.
///
/// `tests/search_field_contract_tests.rs` parses the help text and exercises
/// each name it finds with a real query, so a field documented here but
/// unreachable in [`service_field`] or [`service_list_field`] fails the build.
pub const SEARCHABLE_FIELDS: &[&str] = &[
    "name",
    "language",
    "platform",
    "url",
    "role",
    "team",
    "oncall",
    "docs",
    "ci",
    "path",
    "tags",
    "depends_on",
];

/// Extra spellings accepted for a documented field, mapped to that field.
///
/// `lang` and `deps` exist because [`render`] prints `lang:rust` and
/// `deps:auth,db` on every result line, so a user copying a label straight
/// back into a query would otherwise be naming a field that does not exist.
const FIELD_ALIASES: &[(&str, &str)] = &[
    ("lang", "language"),
    ("tag", "tags"),
    ("deps", "depends_on"),
];

/// Resolve a user-typed field name to its documented spelling, or `None` when
/// it names no searchable field.
///
/// This is the single definition of the vocabulary: both the query parser and
/// the matcher route through it, so a hand-built [`Query`] and a parsed one
/// resolve aliases identically.
pub fn canonical_field(field: &str) -> Option<&'static str> {
    let lower = field.trim().to_lowercase();
    if let Some(f) = SEARCHABLE_FIELDS.iter().find(|f| **f == lower) {
        return Some(f);
    }
    FIELD_ALIASES
        .iter()
        .find(|(alias, _)| *alias == lower)
        .map(|(_, canonical)| *canonical)
}

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
        Self::parse_reporting(raw).0
    }

    /// Same as [`Query::parse`], plus the unrecognised field name when the
    /// query LOOKED like `field:value` but `field` names nothing searchable.
    ///
    /// Such a query becomes a plain substring search over the whole raw
    /// string, because the help text promises substring matching over all
    /// fields and a value may legitimately contain a colon —
    /// `svccat search https://api.example.com` must search for that URL, not
    /// for `//api.example.com` inside a field called `https`. The caller is
    /// expected to report the returned name: a mistyped field would otherwise
    /// be indistinguishable from a genuine zero-result search.
    pub fn parse_reporting(raw: &str) -> (Self, Option<String>) {
        if let Some((field, value)) = raw.split_once(':') {
            let field = field.trim();
            let value = value.trim().to_lowercase();
            if !field.is_empty() && !value.is_empty() {
                if let Some(canonical) = canonical_field(field) {
                    return (
                        Query::FieldValue {
                            field: canonical.to_string(),
                            value,
                        },
                        None,
                    );
                }
                return (
                    Query::AnyField(raw.trim().to_lowercase()),
                    Some(field.to_string()),
                );
            }
        }
        (Query::AnyField(raw.trim().to_lowercase()), None)
    }
}

// ── Search ────────────────────────────────────────────────────────────────────

/// The single-valued searchable fields.
///
/// Returns `None` both for an unknown field and for a multi-valued one; the
/// latter is served by [`service_list_field`], and [`matches_query`] asks it
/// first.
fn service_field<'a>(svc: &'a ServiceEntry, field: &str) -> Option<&'a str> {
    match canonical_field(field)? {
        "name" => Some(svc.name.as_str()),
        "language" => svc.language.as_deref(),
        "platform" => svc.platform.as_deref(),
        "url" => svc.url.as_deref(),
        "role" => svc.role.as_deref(),
        "team" => svc.team.as_deref(),
        "oncall" => svc.oncall.as_deref(),
        "docs" => svc.docs.as_deref(),
        "ci" => svc.ci.as_deref(),
        "path" => svc.path.as_deref(),
        // `tags` and `depends_on` are Vec<String>; see `service_list_field`.
        _ => None,
    }
}

/// The multi-valued searchable fields, which have no single `&str` to return.
fn service_list_field<'a>(svc: &'a ServiceEntry, field: &str) -> Option<&'a [String]> {
    match canonical_field(field)? {
        "tags" => Some(&svc.tags),
        "depends_on" => Some(&svc.depends_on),
        _ => None,
    }
}

fn service_tags(svc: &ServiceEntry) -> &[String] {
    &svc.tags
}

fn matches_query(svc: &ServiceEntry, query: &Query) -> bool {
    match query {
        Query::FieldValue { field, value } => {
            if let Some(values) = service_list_field(svc, field) {
                return values
                    .iter()
                    .any(|v| v.to_lowercase().contains(value.as_str()));
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
