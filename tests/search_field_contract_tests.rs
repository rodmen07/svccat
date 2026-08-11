//! First coverage for `src/search.rs` (`svccat search`).
//!
//! Chosen under the oldest-untested-surface rule (L-014): `src/search.rs` was
//! last touched 2026-05-27 and had, until this file, zero inline `mod tests`
//! and zero references from `tests/` — one of the four remaining zero-coverage
//! modules in the crate, alongside `audit.rs`, `import.rs` and `demo.rs`.
//!
//! The contract under test is a DOC-VERSUS-CODE agreement, so the guard reads
//! both sources rather than restating either: the searchable-field list is
//! parsed out of the `Search` doc comment in `src/cli.rs` (which is what
//! `svccat search --help` prints) and every field it names is then exercised
//! by RUNNING a real query, never by grepping for a match arm.

use assert_cmd::Command;
use svccat::manifest::{Manifest, ServiceEntry};
use svccat::search::{self, Query};

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

// ── The documented lists, read from src/cli.rs ──────────────────────────────

/// The comma-separated names following `<anchor>` in `src/cli.rs`'s `Search`
/// doc comment — i.e. what `svccat search --help` shows a user.
///
/// Parsed from the file rather than hand-copied so that editing the help text
/// alone can never leave this suite asserting a list nobody ships. Hard-fails
/// when the anchor is missing or yields nothing, so renaming it makes the
/// suite loud instead of vacuously green.
fn documented_list(anchor: &str) -> Vec<String> {
    let cli_rs = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/cli.rs");
    let text = fs::read_to_string(&cli_rs)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", cli_rs.display()));

    // The sentence may wrap across several `///` lines, so join from the
    // anchor and stop at the first blank doc line or the first non-doc line.
    let mut collecting = false;
    let mut sentence = String::new();
    for line in text.lines() {
        let doc = line.trim().strip_prefix("///").map(str::trim);
        match doc {
            Some(d) if d.starts_with(anchor) => {
                collecting = true;
                sentence.push_str(d.trim_start_matches(anchor).trim());
                sentence.push(' ');
            }
            Some(d) if collecting => {
                if d.is_empty() {
                    break;
                }
                sentence.push_str(d);
                sentence.push(' ');
            }
            _ => {
                if collecting {
                    break;
                }
            }
        }
    }

    assert!(
        collecting,
        "src/cli.rs no longer contains a `{anchor}` doc line — this suite \
         derives its expectations from it, so it is now guarding nothing. \
         Restore the anchor or rewrite this parser in the same edit."
    );

    let names: Vec<String> = sentence
        .split(',')
        .map(|f| f.trim().trim_end_matches('.').trim().to_string())
        .filter(|f| !f.is_empty())
        .collect();

    assert!(
        !names.is_empty(),
        "parsed the `{anchor}` sentence out of src/cli.rs but it yielded no \
         names; sentence was {sentence:?}"
    );
    names
}

fn documented_search_fields() -> Vec<String> {
    documented_list("Searchable fields:")
}

fn documented_aliases() -> Vec<String> {
    documented_list("Field aliases:")
}

// ── Fixtures ────────────────────────────────────────────────────────────────

/// A service carrying `sentinel` in exactly one documented field and nothing
/// else, so a match can only have come through that field.
///
/// Panics on a documented field this suite does not know how to populate:
/// adding a field to the help text without teaching this map about it must
/// fail the build, not silently shrink the set under test.
fn service_with_only(field: &str, sentinel: &str) -> ServiceEntry {
    let mut svc = ServiceEntry::default();
    svc.name = "placeholder-service".to_string();
    let s = || Some(sentinel.to_string());
    match field {
        "name" => svc.name = sentinel.to_string(),
        "language" => svc.language = s(),
        "platform" => svc.platform = s(),
        "url" => svc.url = s(),
        "role" => svc.role = s(),
        "team" => svc.team = s(),
        "oncall" => svc.oncall = s(),
        "docs" => svc.docs = s(),
        "ci" => svc.ci = s(),
        "path" => svc.path = s(),
        "tags" => svc.tags = vec![sentinel.to_string()],
        "depends_on" => svc.depends_on = vec![sentinel.to_string()],
        other => panic!(
            "src/cli.rs documents {other:?} as a searchable field but this \
             suite does not know how to populate it. Add it to \
             `service_with_only` in the same edit that documents it."
        ),
    }
    svc
}

/// A service that carries the sentinel nowhere, so every assertion below has
/// something it must NOT match.
fn decoy() -> ServiceEntry {
    let mut svc = ServiceEntry::default();
    svc.name = "decoy-service".to_string();
    svc.team = Some("unrelated".to_string());
    svc.tags = vec!["unrelated".to_string()];
    svc.depends_on = vec!["unrelated".to_string()];
    svc
}

fn manifest_of(services: Vec<ServiceEntry>) -> Manifest {
    let mut m = Manifest::default();
    m.services = services;
    m
}

// ── The contract ────────────────────────────────────────────────────────────

/// Every field `svccat search --help` calls searchable must actually be
/// reachable through `field:value` syntax.
///
/// This is the behaviour, not the declaration: each field is populated with a
/// unique sentinel and queried for real. A field whose match arm is missing
/// returns zero results here, which is exactly what a user sees.
#[test]
fn every_documented_search_field_is_reachable_by_field_value_syntax() {
    let sentinel = "zsentinelz";
    let mut dead: Vec<String> = Vec::new();

    for field in documented_search_fields() {
        let m = manifest_of(vec![service_with_only(&field, sentinel), decoy()]);
        let q = Query::parse(&format!("{field}:{sentinel}"));
        let hits = search::run(&m, &q);
        if hits.len() != 1 {
            dead.push(format!(
                "{field}: expected exactly 1 match, got {}",
                hits.len()
            ));
        }
    }

    assert!(
        dead.is_empty(),
        "src/cli.rs documents these searchable fields but `field:value` \
         queries against them match nothing:\n  {}",
        dead.join("\n  ")
    );
}

/// A `field:value` query must be scoped to that field, or the test above
/// would pass on a search that ignores the field entirely.
#[test]
fn a_field_value_query_does_not_match_a_sentinel_in_a_different_field() {
    let sentinel = "zsentinelz";
    let m = manifest_of(vec![service_with_only("team", sentinel), decoy()]);

    let hits = search::run(&m, &Query::parse(&format!("language:{sentinel}")));
    assert!(
        hits.is_empty(),
        "language:{sentinel} matched a service whose sentinel is in `team` \
         only, so field scoping is not being applied"
    );
}

/// `svccat search --help` promises "plain substring matching against all
/// fields", and `url` is one of them — so searching for a URL must work.
///
/// A URL contains a colon, and the query parser splits on the FIRST colon, so
/// this is the case where the `field:value` shortcut collides with the plain
/// form. Whichever way that collision is resolved, it must not resolve into
/// matching nothing.
#[test]
fn a_query_containing_a_colon_still_reaches_the_plain_substring_search() {
    let mut api = ServiceEntry::default();
    api.name = "api".to_string();
    api.url = Some("https://api.example.com".to_string());
    let m = manifest_of(vec![api, decoy()]);

    let hits = search::run(&m, &Query::parse("https://api.example.com"));
    assert_eq!(
        hits.len(),
        1,
        "searching for a literal URL matched nothing; the colon in the query \
         was read as a `field:value` separator and `https` is not a field"
    );
    assert_eq!(hits[0].name, "api");
}

/// The bare-term form is documented as substring matching over all fields, so
/// it must reach the multi-valued fields too.
#[test]
fn a_bare_term_matches_tags_and_depends_on() {
    let m = manifest_of(vec![
        service_with_only("tags", "ztagz"),
        service_with_only("depends_on", "zdepz"),
        decoy(),
    ]);

    assert_eq!(search::run(&m, &Query::parse("ztagz")).len(), 1);
    assert_eq!(search::run(&m, &Query::parse("zdepz")).len(), 1);
}

// ── The two field lists must agree ──────────────────────────────────────────

/// The help text and the code's own vocabulary are two sources that must say
/// the same thing, so this reads both rather than restating either.
#[test]
fn the_help_text_and_searchable_fields_name_the_same_fields() {
    let documented = documented_search_fields();
    let declared: Vec<String> = search::SEARCHABLE_FIELDS
        .iter()
        .map(|f| f.to_string())
        .collect();
    assert_eq!(
        documented, declared,
        "the `Searchable fields:` list in src/cli.rs and \
         `search::SEARCHABLE_FIELDS` disagree; a user reading --help would be \
         told about a vocabulary the code does not implement, or the reverse"
    );
}

/// Same for the aliases: every one the help text names must resolve, and it
/// must resolve to a field the help text also names.
#[test]
fn every_documented_alias_resolves_to_a_documented_field() {
    let fields = documented_search_fields();
    let aliases = documented_aliases();

    for alias in &aliases {
        let resolved = search::canonical_field(alias).unwrap_or_else(|| {
            panic!("src/cli.rs documents `{alias}` as a field alias but it resolves to nothing")
        });
        assert!(
            fields.iter().any(|f| f == resolved),
            "alias `{alias}` resolves to `{resolved}`, which is not in the \
             documented searchable-field list"
        );
    }

    // And each one works end to end, not merely resolves.
    for alias in &aliases {
        let resolved = search::canonical_field(alias).unwrap();
        let sentinel = "zaliasz";
        let m = manifest_of(vec![service_with_only(resolved, sentinel), decoy()]);
        assert_eq!(
            search::run(&m, &Query::parse(&format!("{alias}:{sentinel}"))).len(),
            1,
            "`{alias}:{sentinel}` matched nothing even though `{alias}` \
             resolves to `{resolved}`"
        );
    }
}

/// An alias the help text does NOT name must not be silently accepted, or the
/// test above would be satisfied by a vocabulary wider than the documented one.
#[test]
fn a_field_name_outside_the_documented_vocabulary_resolves_to_nothing() {
    for bogus in ["tema", "https", "submodule", "owner", ""] {
        assert!(
            search::canonical_field(bogus).is_none(),
            "`{bogus}` resolved to a searchable field but is documented nowhere"
        );
    }
}

// ── The unrecognised-field report ───────────────────────────────────────────

/// A mistyped field must be reported, not silently degraded into a substring
/// search that happens to find nothing.
#[test]
fn an_unrecognized_field_prefix_is_reported() {
    let (query, unrecognized) = Query::parse_reporting("tema:payments");
    assert_eq!(unrecognized.as_deref(), Some("tema"));
    match query {
        Query::AnyField(term) => assert_eq!(term, "tema:payments"),
        other => panic!("expected a plain substring search, got {other:?}"),
    }
}

/// ...and a field that IS recognised must not be reported, or the note would
/// fire on every ordinary query and mean nothing.
#[test]
fn a_recognized_field_prefix_is_not_reported() {
    for good in ["team:payments", "deps:auth", "lang:rust", "depends_on:auth"] {
        let (_, unrecognized) = Query::parse_reporting(good);
        assert_eq!(
            unrecognized, None,
            "`{good}` was reported as naming an unknown field"
        );
    }
    // A bare term has no field part at all, so there is nothing to report.
    assert_eq!(Query::parse_reporting("auth").1, None);
    // A colon with an empty value is not a field query either.
    assert_eq!(Query::parse_reporting("team:").1, None);
}

// ── Binary level ────────────────────────────────────────────────────────────

const FIXTURE: &str = "\
services:
  - name: auth
    team: platform
    url: https://auth.example.com
  - name: web
    team: growth
    depends_on:
      - auth
";

/// Run `svccat search <query>` against FIXTURE, return (stdout, stderr).
fn search_cmd(query: &str) -> (String, String) {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("services.yaml"), FIXTURE).unwrap();

    let out = Command::cargo_bin("svccat")
        .unwrap()
        .current_dir(tmp.path())
        .env("NO_COLOR", "1")
        .args(["search", query])
        .output()
        .unwrap();

    (
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

#[test]
fn the_binary_finds_a_service_by_depends_on() {
    let (stdout, _) = search_cmd("depends_on:auth");
    assert!(
        stdout.contains("1 result") && stdout.contains("web"),
        "`svccat search depends_on:auth` did not report the depending \
         service. stdout:\n{stdout}"
    );
}

#[test]
fn the_binary_finds_a_service_by_url_typed_in_full() {
    let (stdout, stderr) = search_cmd("https://auth.example.com");
    assert!(
        stdout.contains("1 result") && stdout.contains("auth"),
        "searching for a full URL found nothing. stdout:\n{stdout}"
    );
    assert!(
        stderr.contains("`https` is not a searchable field"),
        "the fallback to a substring search was not reported. stderr:\n{stderr}"
    );
}

#[test]
fn the_binary_reports_a_mistyped_field_on_stderr_and_still_exits_zero() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("services.yaml"), FIXTURE).unwrap();

    let out = Command::cargo_bin("svccat")
        .unwrap()
        .current_dir(tmp.path())
        .env("NO_COLOR", "1")
        .args(["search", "tema:platform"])
        .output()
        .unwrap();

    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("`tema` is not a searchable field"),
        "no note naming the unrecognised field. stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("depends_on"),
        "the note does not list the searchable fields. stderr:\n{stderr}"
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "a mistyped field is a note, not a failure"
    );
}

#[test]
fn an_ordinary_field_query_prints_no_note() {
    let (stdout, stderr) = search_cmd("team:platform");
    assert!(stdout.contains("auth"), "stdout:\n{stdout}");
    assert!(
        !stderr.contains("is not a searchable field"),
        "an ordinary query emitted the unrecognised-field note. stderr:\n{stderr}"
    );
}
