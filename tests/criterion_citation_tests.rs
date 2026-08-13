//! Drift guard: every criterion version this repo CITES in prose must be the
//! criterion the build actually resolves.
//!
//! WHY THIS EXISTS. `.github/workflows/benchmark.yml` and
//! `tests/benchmark_output_contract_tests.rs` do not merely configure the
//! benchmark job; they carry an EMPIRICAL claim about how one specific version
//! of criterion behaves — that a partially restored `target/criterion` makes it
//! print `Criterion.rs ERROR: ...` through `println!`, onto stdout, in the
//! middle of the single line `benchmark-action/github-action-benchmark`'s cargo
//! extractor matches. That claim is the entire justification for the
//! `rm -rf target/criterion` line in the `Run benchmarks` step, which otherwise
//! reads as removable hygiene.
//!
//! A dependency bump silently invalidates a claim like that. The evidence was
//! gathered on one version, the CI job then runs a different one, and the
//! paragraph explaining why the step exists goes on looking authoritative while
//! describing a program nobody builds any more. Nothing fails, because prose
//! cannot fail. The bump that added this guard is the worked example: the
//! citations still said one version while `Cargo.toml` had moved to the next
//! major line, and only a hand re-derivation caught it.
//!
//! WHAT IT ASSERTS. Every citation found anywhere in the glob-discovered corpus
//! names the version in `Cargo.lock`. A citation naming only `MAJOR.MINOR`
//! claims the behaviour line and is checked against the lock's major and minor;
//! a citation naming a full `MAJOR.MINOR.PATCH` (a registry path such as
//! `criterion-<version>/src/...`) claims that exact build and is checked in
//! full. Each citation is therefore held to exactly what it says, so a patch
//! bump reddens only the citations that named a patch.
//!
//! WHAT IT DOES NOT ASSERT, stated because it is the limit: this proves the
//! prose and the lockfile AGREE, not that the described behaviour is still
//! real. Only running the benchmark in each of the three cache states does
//! that, and the bump that lands with this guard did so. What the guard buys is
//! that the next bump cannot skip it and stay green.
//!
//! The obliged party can see the trigger, which is why this is a test and not a
//! convention written in a comment: the author who edits the dependency holds
//! the cause in their hand, and the check runs on their own pull request.

use std::fs;
use std::path::{Path, PathBuf};

/// The dependency whose citations are guarded. Written once so no assertion
/// message has to spell it, and so this file never contains the literal shape
/// its own scanner looks for.
const CRATE: &str = "criterion";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()))
        .replace("\r\n", "\n")
}

/// Every file that may carry a citation, GLOB-DISCOVERED rather than listed.
///
/// Hand-enumerating the corpus is how this class of guard goes partial without
/// anyone noticing: a new workflow or a new test file citing a version would
/// ship OUTSIDE the corpus, which is indistinguishable from passing. The
/// patterns cover the three places prose about a dev-dependency plausibly
/// lives, plus the manifest that declares it.
fn citation_corpus() -> Vec<PathBuf> {
    let root = repo_root().display().to_string().replace('\\', "/");
    let patterns = [
        format!("{root}/.github/workflows/*.y*ml"),
        format!("{root}/tests/*.rs"),
        format!("{root}/benches/*.rs"),
        format!("{root}/Cargo.toml"),
    ];

    let mut files: Vec<PathBuf> = Vec::new();
    for pattern in &patterns {
        let matched: Vec<PathBuf> = glob::glob(pattern)
            .unwrap_or_else(|e| panic!("glob pattern `{pattern}` does not compile: {e}"))
            .map(|entry| entry.expect("readable glob entry"))
            .collect();
        assert!(
            !matched.is_empty(),
            "glob `{pattern}` matched no files, so any citation living there would be \
             invisible to this guard and it would pass vacuously"
        );
        files.extend(matched);
    }
    files.sort();
    files
}

/// One version citation: where it was found and what it claims.
#[derive(Debug, PartialEq, Eq)]
struct Citation {
    line: usize,
    /// The version exactly as written, e.g. `0.1` or `0.1.2`.
    version: String,
}

/// Every citation in one text.
///
/// A citation is the crate name followed by `-` or a space and then a dotted
/// numeric version of at least two components. That shape deliberately does NOT
/// match the neighbouring spellings this repo is full of: the sibling crate
/// (name plus `-p...`), the macro imports (name plus `_group` / `_main`), the
/// cache directory (`target/` plus the name, followed by a path separator or
/// end of line), or the manifest requirement (name plus ` = "..."`, whose next
/// character is `=`). Those are covered as explicit negatives by
/// `the_scanner_separates_a_version_citation_from_the_names_around_it`.
fn citations(text: &str) -> Vec<Citation> {
    let mut out = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let mut from = 0usize;
        while let Some(offset) = line[from..].find(CRATE) {
            let after = from + offset + CRATE.len();
            from = after;

            let Some(separator) = line.as_bytes().get(after) else {
                continue;
            };
            if *separator != b'-' && *separator != b' ' {
                continue;
            }

            let raw: String = line[after + 1..]
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            let parts: Vec<&str> = raw.split('.').filter(|p| !p.is_empty()).collect();
            if parts.len() < 2 || !parts.iter().all(|p| p.bytes().all(|b| b.is_ascii_digit())) {
                continue;
            }

            out.push(Citation {
                line: index + 1,
                version: parts.join("."),
            });
        }
    }
    out
}

/// The version `Cargo.lock` resolves for the guarded crate.
///
/// Matched on the package NAME being exactly the guarded crate, never on a
/// prefix: the lockfile also holds its plotting sibling, whose name starts with
/// the same nine characters and whose version moves independently.
fn locked_version() -> String {
    let lock = read(&repo_root().join("Cargo.lock"));
    let wanted = format!("name = \"{CRATE}\"");

    let lines: Vec<&str> = lock.lines().collect();
    for (index, line) in lines.iter().enumerate() {
        if line.trim() != wanted {
            continue;
        }
        for following in &lines[index + 1..] {
            if let Some(rest) = following.trim().strip_prefix("version = ") {
                return rest.trim_matches('"').to_string();
            }
            if following.trim() == "[[package]]" {
                break;
            }
        }
        panic!("Cargo.lock has a `{wanted}` package with no version line under it");
    }
    panic!("Cargo.lock holds no package named `{CRATE}`; the dependency was removed and this guard now guards nothing");
}

#[test]
fn every_cited_version_matches_the_locked_dependency() {
    let locked = locked_version();
    let locked_parts: Vec<&str> = locked.split('.').collect();
    assert!(
        locked_parts.len() >= 2,
        "`{locked}` is not a dotted version; the lockfile parse is wrong, not the citations"
    );
    let locked_line = format!("{}.{}", locked_parts[0], locked_parts[1]);

    let mut found = 0usize;
    let mut wrong: Vec<String> = Vec::new();
    for path in citation_corpus() {
        let text = read(&path);
        for citation in citations(&text) {
            found += 1;
            let claimed = citation.version.split('.').count();
            let expected = if claimed >= 3 { &locked } else { &locked_line };
            if citation.version != *expected {
                wrong.push(format!(
                    "{}:{} cites `{}` but Cargo.lock resolves `{}`",
                    path.display(),
                    citation.line,
                    citation.version,
                    locked,
                ));
            }
        }
    }

    assert!(
        found > 0,
        "no version citation was found anywhere in the corpus. Either the prose that \
         justifies the `rm -rf target/criterion` step was deleted, or the citation \
         spelling changed and this guard now passes without reading anything"
    );
    assert!(
        wrong.is_empty(),
        "the dependency moved and the prose describing its behaviour did not. Every \
         claim below was measured against a different build than the one CI runs, so \
         re-run the three cache states on the locked version and restate them, rather \
         than editing the numbers to match: {}",
        wrong.join(" || "),
    );
}

#[test]
fn the_scanner_separates_a_version_citation_from_the_names_around_it() {
    let probe = format!(
        "{CRATE}-1.2.3/src/macros_private.rs:36\n\
         reproduced locally on {CRATE} 4.5 in all three states\n\
         use {CRATE}::{{{CRATE}_group, {CRATE}_main}};\n\
         rm -rf target/{CRATE}\n\
         {CRATE}-plot 9.9.9 is a different package\n\
         {CRATE} = \"7.7\"\n"
    );

    assert_eq!(
        citations(&probe),
        vec![
            Citation {
                line: 1,
                version: "1.2.3".to_string()
            },
            Citation {
                line: 2,
                version: "4.5".to_string()
            },
        ],
        "the scanner must read a registry path and a prose citation, and must read \
         nothing from the macro imports, the cache directory, the sibling package or \
         the manifest requirement. A scanner that also matched the sibling would fail \
         the agreement test above for a version that is not even this dependency's"
    );
}
