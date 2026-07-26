//! Drift guard: `ROADMAP.md` against `Cargo.toml` and `CHANGELOG.md`.
//!
//! The roadmap is a planning document that nothing else in this repo checks, and it
//! rots faster than anything else here. Two reconciliations were needed inside eight
//! days: the 2026-07-24 pass (PR #18) fixed a roadmap that still called v1.5.0
//! upcoming, and by 2026-07-26 that same pass was itself stale — its "unreleased on
//! main" list stopped at PR #17 while eleven more PRs had merged, it carried a
//! milestone whose remaining work had shipped in four separate PRs, and it never
//! mentioned three multi-repo features that were already on `main`. A prose
//! reconciliation cannot see any of that; the third one is a test instead.
//!
//! What is guarded, each claim derived from a second source so the two can disagree
//! out loud:
//!
//! 1. the `- Crate version on main:` line equals `Cargo.toml`'s `[package] version`,
//! 2. no version `CHANGELOG.md` says is released is still an upcoming `### vX.Y.Z`
//!    milestone under `## Milestones`,
//! 3. no released version sits in a `BLOCKED` row of the blocked/user-only table, and
//! 4. `## Unreleased on main` exists exactly when the CHANGELOG has `[Unreleased]`
//!    entries — shipped-but-unpublished work must be visible in the roadmap, and must
//!    disappear from it when the release is cut.
//!
//! The extractors are the only thing standing between these assertions and vacuous
//! truth, so `extractors_find_what_they_are_looking_for` exercises every one of them
//! on synthetic input whose answer is known. A parser that silently stops matching
//! fails there instead of quietly passing everything above.

use std::fs;
use std::path::Path;

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn read(name: &str) -> String {
    let path = repo_root().join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("{} is unreadable: {e}", path.display()))
}

type Version = (u64, u64, u64);

fn parse_version(raw: &str) -> Option<Version> {
    let raw = raw.trim().trim_matches('*').trim().trim_start_matches('v');
    let mut parts = raw.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

fn show(v: Version) -> String {
    format!("{}.{}.{}", v.0, v.1, v.2)
}

/// The lines of `doc` belonging to the `## `-level section with this heading,
/// excluding the heading itself. `### ` subheadings stay inside the section.
fn section<'a>(doc: &'a str, heading: &str) -> Vec<&'a str> {
    let mut inside = false;
    let mut out = Vec::new();
    for line in doc.lines() {
        if line.trim_end() == heading {
            inside = true;
            continue;
        }
        if inside && line.starts_with("## ") {
            break;
        }
        if inside {
            out.push(line);
        }
    }
    out
}

/// The `version = "x.y.z"` of the `[package]` table, ignoring every other table.
fn crate_version(cargo_toml: &str) -> Option<Version> {
    let mut in_package = false;
    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if in_package {
            if let Some(rest) = trimmed.strip_prefix("version") {
                let rest = rest.trim_start().strip_prefix('=')?;
                return parse_version(rest.trim().trim_matches('"'));
            }
        }
    }
    None
}

/// Every version with a `## [x.y.z]` CHANGELOG heading. `[Unreleased]` is not one.
fn released_versions(changelog: &str) -> Vec<Version> {
    changelog
        .lines()
        .filter_map(|line| line.trim_end().strip_prefix("## ["))
        .filter_map(|rest| rest.split(']').next())
        .filter_map(parse_version)
        .collect()
}

/// True when the CHANGELOG's `[Unreleased]` section holds anything but blank lines.
fn changelog_has_unreleased_entries(changelog: &str) -> bool {
    section(changelog, "## [Unreleased]")
        .into_iter()
        .any(|line| !line.trim().is_empty())
}

/// The version claimed by the `- Crate version on main:` bullet in `## Current state`.
fn crate_version_claimed_by_roadmap(roadmap: &str) -> Option<Version> {
    let line = roadmap
        .lines()
        .find(|line| line.trim_start().starts_with("- Crate version on main:"))?;
    line.split_whitespace()
        .find_map(|word| parse_version(word).filter(|_| word.contains('v')))
}

/// Versions presented as still-upcoming milestones (`### vX.Y.Z...`) under
/// `## Milestones`. Restricted to that section so a history entry naming a shipped
/// version is not mistaken for a plan.
fn upcoming_milestone_versions(roadmap: &str) -> Vec<Version> {
    section(roadmap, "## Milestones")
        .into_iter()
        .filter_map(|line| line.trim_end().strip_prefix("### v"))
        .filter_map(|rest| parse_version(rest.split([':', ' ', '\t']).next().unwrap_or_default()))
        .collect()
}

/// Versions named by rows of the blocked/user-only table whose status cell is
/// exactly `BLOCKED`.
fn blocked_row_versions(roadmap: &str) -> Vec<Version> {
    let mut out = Vec::new();
    for line in section(roadmap, "## Blocked and user-only summary") {
        if !line.trim_start().starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = line
            .trim()
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        let [item, status, ..] = cells.as_slice() else {
            continue;
        };
        if *status != "BLOCKED" {
            continue;
        }
        out.extend(
            item.split_whitespace()
                .filter_map(|word| parse_version(word).filter(|_| word.contains('v'))),
        );
    }
    out
}

#[test]
fn roadmap_current_state_matches_the_crate_version() {
    let declared = crate_version(&read("Cargo.toml")).expect("Cargo.toml [package] version parses");
    let claimed = crate_version_claimed_by_roadmap(&read("ROADMAP.md"))
        .expect("ROADMAP.md has a `- Crate version on main: **vX.Y.Z**` bullet");
    assert_eq!(
        claimed,
        declared,
        "ROADMAP.md claims the crate on main is v{} but Cargo.toml says {}. \
         A release-prep PR must move both.",
        show(claimed),
        show(declared)
    );
}

#[test]
fn roadmap_does_not_list_a_released_version_as_an_upcoming_milestone() {
    let changelog = read("CHANGELOG.md");
    let released = released_versions(&changelog);
    assert!(
        released.len() > 20,
        "parsed only {} released versions from CHANGELOG.md; the `## [x.y.z]` heading \
         format changed and this guard stopped guarding anything",
        released.len()
    );

    let milestones = upcoming_milestone_versions(&read("ROADMAP.md"));
    assert!(
        !milestones.is_empty(),
        "parsed no `### vX.Y.Z` milestones from ROADMAP.md's `## Milestones` section; \
         the heading format changed and this guard stopped guarding anything"
    );

    let shipped: Vec<String> = milestones
        .into_iter()
        .filter(|v| released.contains(v))
        .map(show)
        .collect();
    assert!(
        shipped.is_empty(),
        "ROADMAP.md still lists {} under `## Milestones`, but CHANGELOG.md says it has \
         already been released; move the section to `## History and supersession`",
        shipped.join(", ")
    );
}

#[test]
fn roadmap_does_not_list_a_released_version_as_blocked() {
    let released = released_versions(&read("CHANGELOG.md"));
    let shipped: Vec<String> = blocked_row_versions(&read("ROADMAP.md"))
        .into_iter()
        .filter(|v| released.contains(v))
        .map(show)
        .collect();
    assert!(
        shipped.is_empty(),
        "ROADMAP.md's blocked/user-only table still gates {}, which CHANGELOG.md says \
         has shipped",
        shipped.join(", ")
    );
}

#[test]
fn roadmap_declares_unreleased_work_exactly_when_the_changelog_has_some() {
    let has_entries = changelog_has_unreleased_entries(&read("CHANGELOG.md"));
    let has_section = read("ROADMAP.md")
        .lines()
        .any(|line| line.trim_end() == "## Unreleased on main");
    assert_eq!(
        has_section,
        has_entries,
        "CHANGELOG.md {} unreleased entries but ROADMAP.md {} an `## Unreleased on main` \
         section. Shipped-but-unpublished work must be visible in the roadmap, and must \
         disappear from it when the release is cut.",
        if has_entries { "has" } else { "has no" },
        if has_section { "has" } else { "lacks" },
    );
}

// Every extractor above is exercised on input whose answer is known, so a parser that
// silently stops matching fails here rather than turning the four guards into
// assertions that cannot fire.
#[test]
fn extractors_find_what_they_are_looking_for() {
    assert_eq!(
        crate_version("[dependencies]\nversion = \"9.9.9\"\n\n[package]\nversion = \"1.5.0\"\n"),
        Some((1, 5, 0)),
        "crate_version must read [package], not the first `version =` in the file"
    );

    assert_eq!(
        released_versions("## [Unreleased]\n## [1.5.0] - 2026-07-18\n## [1.4.1] - 2026-07-09\n"),
        vec![(1, 5, 0), (1, 4, 1)],
        "released_versions must skip [Unreleased] and keep dated releases"
    );

    assert!(changelog_has_unreleased_entries(
        "## [Unreleased]\n\n### Added\n- thing\n\n## [1.5.0] - 2026-07-18\n"
    ));
    assert!(
        !changelog_has_unreleased_entries(
            "## [Unreleased]\n\n## [1.5.0] - 2026-07-18\n\n### Added\n- thing\n"
        ),
        "an empty [Unreleased] must not be fooled by the next release's entries"
    );

    assert_eq!(
        crate_version_claimed_by_roadmap(
            "## Current state (2026-07-26)\n\n- Crate version on main: **v1.5.0** (`Cargo.toml`)\n"
        ),
        Some((1, 5, 0)),
        "the crate-version bullet must survive bold markers and a trailing clause"
    );
    assert_eq!(
        crate_version_claimed_by_roadmap("- Published to crates.io: **1.5.0**, 2026-07-18\n"),
        None,
        "only the `- Crate version on main:` bullet is the anchor; the crates.io line is not"
    );

    assert_eq!(
        upcoming_milestone_versions(
            "## Milestones\n\
             ### v1.6.0: publish what is already on main\n\
             prose\n\
             ### v1.7.0: dependency currency, part 1 — PR 1 shipped\n\
             ## History and supersession\n\
             ### v1.5.0: SPDX SBOM release\n"
        ),
        vec![(1, 6, 0), (1, 7, 0)],
        "milestones must come from `## Milestones` only, and must parse a heading whose \
         version is followed by a colon or a space"
    );

    assert_eq!(
        blocked_row_versions(
            "## Blocked and user-only summary\n\
             | v1.7.0 dependency bumps | BLOCKED | gated on the v1.5.0 release |\n\
             | Writing the `CRATES_IO_TOKEN` repo secret | USER-ONLY | never handled by an agent |\n\
             |------|--------|--------|\n\
             ## History and supersession\n"
        ),
        vec![(1, 7, 0)],
        "blocked_row_versions must read the status cell, ignore USER-ONLY rows, and not \
         trip over the separator row"
    );
}
