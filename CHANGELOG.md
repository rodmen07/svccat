# Changelog

All notable changes to svccat are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added

- **`svccat check --format sarif` results now carry `region.startLine`, so SARIF consumers can render them as inline annotations** (PR #36). The module doc had promised "inline PR annotations" since the format shipped, but every result named only the manifest file with no region — there was no line to anchor an annotation to, because the YAML parser exposes position information only for documents that fail to parse. A new internal line scan recovers the manifest line of each service's `name:` entry positionally (the Nth `name:` key in the `services:` block belongs to the Nth service) and every drift finding about a declared service now anchors there. The scan fails closed: when it and the parser disagree about how many services the file holds (e.g. a block scalar whose body resembles a `name:` key), nothing is attached and findings stay file-level exactly as before, never anchored to a wrong line. Ping findings stay deliberately file-level (a ping failure is about a URL answering, not a line in the file), as do findings about undeclared services, which have no manifest line to point at. Library: `drift::DriftItem` gains a `line: Option<usize>` field (additive on a `#[non_exhaustive]` struct per `docs/API_STABILITY.md`); it serializes only when present, so `--format json` documents and saved snapshots gain the field for anchored items and old snapshots and baselines keep loading unchanged.

- **`svccat check --format github-annotation` annotations now carry `line=`, so they appear inline at the drifting service's `name:` entry** (PR #37). The follow-through on the SARIF `region.startLine` change above: the annotation renderer — the DEFAULT format when `check` runs inside GitHub Actions — kept anchoring every annotation to the manifest file as a whole even after drift items began carrying the manifest line of their service's `name:` entry. A drift annotation whose item carries a line now emits it as the `line=` workflow-command property, placing the annotation on that line in the pull-request view; items the fail-closed line scan could not anchor render exactly as before, with no `line=` property rather than a wrong one. Ping findings stay deliberately file-level (a ping failure is about a URL answering, not a line in the file), matching the SARIF renderer. The line value is formatted from a `usize` inside the one escaping-aware line builder, so it cannot smuggle workflow-command syntax and no call site can bypass the builder.

### Fixed

- **`svccat export --format backstage-yaml` now emits byte-identical output for an unchanged manifest** (PR #50). The `metadata.annotations` block was built in a `HashMap`, and `serde_yaml` writes a map in iteration order — randomised per process — so a single service declaring `oncall`, `path`, `docs` and `ci` exported in nine different annotation orders across ten consecutive runs. A Backstage `catalog-info.yaml` is a file you commit and regenerate, so this was not a display preference: every regeneration produced a diff on services nothing had changed about, and a byte comparison of exported catalogs (a `git diff --exit-code` after regeneration, a CI drift check) reported a change that was not one. Annotations are now held in a `BTreeMap` and emitted in alphabetical key order, which is stable, tool-agnostic, and what any other YAML writer that rewrites the file will produce. Alphabetical was chosen over insertion order deliberately: the insertion sequence was just the order of the `if let` arms in the renderer and encoded nothing a reader could rely on. No public API changed — `CatalogMetadata` is private to the module — and no annotation key, value, or any other part of the document moved.

- **`svccat audit --cost-estimate` prints its `By platform:` block in the same order on every run** (PR #49). The block was ordered by a stable sort on `(cost as i32).wrapping_neg()` over a `HashMap`, so any two platforms whose costs compared equal simply kept whatever order the map iterated in — which is randomised per process. Ten consecutive runs over one unchanged three-service catalog, every platform estimating $10.00, printed six different orderings. The truncation to `i32` widened the tie class past exact equality as well, so $10.90 and $10.20 tied too. Platforms are now ordered by their real cost, dearest first, with the platform name breaking ties, so the block depends only on the data and a report saved from one run diffs cleanly against the next. `svccat audit --format json` was never affected — it builds a `BTreeMap` — and is unchanged. Library: `cost::CostBreakdown::platforms_by_cost` is new and additive; no existing signature changed.

- **`svccat search depends_on:<name>` no longer returns nothing, and a mistyped field name says so instead of looking like an empty result** (PR #46). `svccat search --help` has listed `depends_on` as a searchable field since the command shipped, but the `field:value` form had no match arm for it — every `depends_on:auth` query returned zero results, indistinguishable from a catalog where nothing depends on `auth`. The same silence covered any query whose value contains a colon: the parser split on the first one, so `svccat search https://api.example.com` looked for `//api.example.com` inside a field called `https` and matched nothing, even though the help text promises plain substring matching over all fields and names `url` among them. Now `depends_on` (and the `deps` alias the result lines already print) is matched like `tags`; a `field:value` query whose field names nothing searchable falls back to the documented plain substring search over the whole query and writes a note to stderr naming the unrecognised field and listing the real ones, so a typo is visible rather than silently empty. Exit codes are unchanged — this is a note, not a failure. Library: `search::SEARCHABLE_FIELDS`, `search::canonical_field` and `Query::parse_reporting` are new and additive; `Query::parse` keeps its signature, and the query parser and the matcher now resolve field names and aliases through that one function, so a hand-built `Query` and a parsed one agree.

- **`svccat check --format sarif --output <file>` and `--format junit --output <file>` now write the file instead of silently printing to stdout** (PR #44). Both formats exited 0, wrote nothing, and printed the report to stdout: neither was listed in the routing that renders a check report to a string, so both fell through to the print-directly arm and `--output` did nothing at all. These are exactly the two formats a CI step uploads *from disk* -- SARIF to code scanning, JUnit XML to a test reporter -- so the silent drop landed on the artifacts most likely to be wanted as files. `--format github-annotation` still prints and ignores `--output`, deliberately: a `::error::` workflow command means something only on the live stdout GitHub Actions is watching, so writing one to a file would produce an inert artifact rather than fix anything. Library: `output::sarif::render_check_to_string` is new and additive (`junit::build_check_document` already existed); no existing signature changed. `src/output/junit.rs` also gained its first tests, which assert well-formedness with a real XML parser rather than by re-implementing the escaper in reverse.

- **A metadata field declared as an empty string no longer satisfies a policy that requires it** (PR #42). `svccat` had six places that asked "does this service declare `team`?" and they did not agree: `stats` and `lint` treated `team: ""` as undeclared, while `scorecard`, `policy` and both of `drift`'s field checks tested only `Option::is_some` and credited it. The consequence was a gate that a blank value walked straight through — `.svccat/policy.yaml` with `required: [team]` reported "All services comply" for a catalog naming no owner at all, the `policy` step of `svccat ci` passed with it, and a manifest's own `policy.require_fields` raised no violation. `svccat scorecard` separately inflated `completeness` (and therefore `total`, `avg_completeness` and `avg_total`, in the terminal, JSON and markdown renderers) by counting blank fields as populated: a service with six empty strings scored 55% where `svccat stats` on the same file said 0%. All six surfaces now route through one predicate, `manifest::ServiceEntry::has_field`, which reads declared-and-non-empty; an empty field is reported exactly like an absent one everywhere. This makes the gates stricter, so a catalog that was passing on blank values will now report those services — which is the finding it was always meant to raise. Library: `ServiceEntry` gains `field_value` and `has_field` (additive on a `#[non_exhaustive]` struct per `docs/API_STABILITY.md`); no existing signature changed.

- **An orphaned SBOM sidecar no longer wedges its snapshot name, and `svccat snapshot delete` is now the recovery** (PR #40). When `.svccat/snapshots/<name>.json` was deleted by hand (or lost to a partial copy) while `<name>.spdx.json` survived, the name was stuck: `snapshot delete <name>` bailed with "not found" before its sidecar cleanup ran, and `snapshot save <name> --sbom` failed on the leftover sidecar — after already writing the new snapshot json, leaving a half-finished state (snapshot saved, command exited nonzero). The only working recovery (`save` without `--sbom`, then `delete`) was documented nowhere. Now: `delete` on a missing snapshot removes an orphaned sidecar and succeeds (still an error when neither file exists), `save --sbom` checks the sidecar before writing anything so it can no longer half-finish, and the sidecar-exists error names the recovery command (`svccat snapshot delete <name>`) instead of suggesting manual file removal.

- **`svccat check --ping --format github-annotation` no longer silently drops the ping results** (PR #35). The annotation renderer — the DEFAULT format when `check` runs inside GitHub Actions — took only the drift report, so an unreachable or SSRF-blocked service URL produced no annotation at all, and invisibly twice over since ping never affects `check`'s exit code. The same defect was fixed for the SARIF renderer one release cycle earlier; this closes its sibling. A failed probe now emits an `::error` annotation (`svccat [UNREACHABLE]` / `svccat [INVALID-URL]`) naming the service, the URL and the reason; a reachable service deliberately emits nothing, matching the SARIF and JUnit renderers, where only findings are reported.

- **`svccat check --format sarif` no longer writes bare absolute paths into `artifactLocation.uri`** (PR #34). SARIF 2.1.0 types that property as a URI reference, so `svccat --root C:\repo check --format sarif` used to emit `C:/repo/services.yaml`, which a conforming consumer parses as a URI with scheme `c` (a drive letter is a legal scheme name), and a POSIX absolute path was read as a root-relative reference rather than a file URI. An absolute manifest path is now relativised against the run root when it sits under it (the SARIF-preferred repo-relative form, and what GitHub Code Scanning expects), and otherwise becomes a proper percent-encoded `file://` URI. Relative paths are emitted exactly as before, and the artifact URI is computed once and threaded into every result, so the `artifacts` entry and the `results` locations can never disagree.

- **`svccat snapshot diff` no longer prints its drift lists in a random order, or in a different format from `svccat diff`** (PR #31). The two commands build the same public `DiffReport::new_drift` / `resolved_drift` fields through two different code paths, and only one of them was correct. The snapshot path differenced two `HashSet`s, whose iteration order is unspecified and re-randomised per set by the default hasher, so two runs over byte-identical snapshots listed the same drift in different orders (observed failing on the first repeat inside a single process, not just across runs); it also emitted the bare `service:message` dedup key while `svccat diff` emitted the severity-prefixed line, so one public field carried two formats depending on which command filled it. Both lists are now built by one shared helper that walks the source snapshot's drift vector in order and reports each `service:message` once, so `svccat snapshot diff --format markdown` is stable enough to commit or to diff in CI, and both commands render drift identically. A `service:message` whose severity was re-classified between snapshots is still not reported as one resolved plus one new, as before.

### Security

- **GitHub Actions annotations now escape manifest content per the workflow-command rules** (PR #35). Every value interpolated into an `::error` / `::warning` line (the message, the file property, the title) was written raw, and manifest content is untrusted input in the annotation context: a service name or policy message containing a newline terminated the workflow command early, and any following text beginning with `::` would be executed by the runner as a NEW workflow command (annotation spoofing, `::add-mask`, `::stop-commands`, ...). All interpolated values now go through GitHub's documented escaping (`%` → `%25` first, then `\r` → `%0D`, `\n` → `%0A`; property values additionally `:` → `%3A`, `,` → `%2C`), applied inside the one line-building function so no call site can opt out — the same shape as the D3 tooltip XSS fix, where the escape lives in the builder rather than at each caller.

- **Policy files are read under resource limits, the way the manifest already was** (PR #33). `Manifest::load` caps the file size before parsing and bounds the parsed document afterwards, naming YAML-bomb resource exhaustion as the reason, but `.svccat/policy.yaml` went through the same deserializer, across the same trust boundary, with no limits at all. A policy file is now refused before parsing when it is larger than 1 MiB (the cap is enforced on the bytes actually read, so an oversized document is never held in memory whole or handed to the parser at all), and refused after parsing when it declares more than 10,000 fields across `required` and `recommended` combined or names a field longer than 256 bytes. The refusal is a new `PolicyLoadError::Limit` naming the limit breached, distinct from `Parse` on purpose: an over-limit document may be perfectly well-formed YAML, and calling it a parse failure would send the user hunting for a syntax error that is not there. Every command that reads the policy file (`svccat policy`, `svccat ci`, `svccat scorecard`) inherits the bound through the one shared loader.

## [1.6.0] - 2026-07-26

Everything in this release was merged to `main` between 2026-07-19 and 2026-07-26 and
had no published version until now: `cargo install svccat` delivered 1.5.0 the whole
time. Each entry cites the pull request that landed it.

### Added

- **Multi-repo `workspace` config completion, and `workspace check --filter` actually filters** (PR #4). The `[workspace]` `name` and `description` fields of `svccat.toml` are parsed and propagated to all three `workspace check` renderers. `--filter` (comma-separated repo names) had been accepted and then discarded since v0.21.0, a documented flag that silently did nothing, and now restricts the run to the named repos.
- **`[reporting]` section in `svccat.toml`** (PR #5): `format` sets the default output format (an explicit `--format` still wins), `include_cross_repo_deps = false` skips cross-repo dependency analysis entirely rather than merely hiding it from the report, and `exclude_patterns` merges additively into the discovery ignore globs coming from `--ignore` and the root config, so no source of ignores can silently drop another's. A recognised key carrying an unusable value is rejected rather than ignored.
- **`svccat workspace check --format html`** (PR #6, hardened by PR #8): a self-contained multi-repo HTML report (summary table, per-repo drift tables, cross-repo dependency analysis, and an interactive D3 dependency graph) with no external assets and no network access when it is opened.
- **CycloneDX 1.7 JSON SBOM export** (PR #11): `svccat export --format cyclonedx-json [--output <file>]`, a sibling of the SPDX 2.3 exporter added in v1.5.0.
- **`svccat lint` now validates inline policy rules** (PR #12). `manifest.policy.rules` was never looked at by `lint`, so a duplicate rule id, a dangling `base` reference, a bad severity or an unparsable expression passed lint and then disabled *every* policy rule at check time behind a warning that did not change the exit code. A new `src/rule_schema.rs` runs the structural checks first, including base-chain cycle detection, and only then delegates to the rule compiler for semantic ones. Rule-compiler errors now name the offending rule id.
- **`PolicyConfig::load_checked` and `PolicyLoadError`** (library, PR #25): loads `.svccat/policy.yaml` and reports *why* an existing file did not load, via `PolicyLoadError` (`Read` / `Parse`, each carrying the path). `PolicyConfig::load` is unchanged for callers that want "policy or nothing" and now delegates to `load_checked`, so the two can never disagree about which candidate file wins.

### Changed

- **`notify` upgraded from 6.1.1 to 8.2.0** (PR #21), the filesystem-watching backend behind `svccat watch` and `svccat ci --watch`. No behaviour change is intended or expected: the `Config` / `RecommendedWatcher` / `RecursiveMode` / `EventKind` surface svccat uses is identical across both majors, and `src/watch.rs` gained its first tests (an `is_relevant` contract test and an end-to-end test that the platform watcher really delivers events) so the swap is proven rather than assumed. Transitive dependency count dropped from 213 to 212 (`crossbeam-channel`, `filetime` and `bitflags` 1.x dropped, `notify-types` added).

### Fixed

- **`svccat watch` now reports a service whose `path` or `submodule` was edited** (PR #24). The comparison behind watch mode's "Manifest changes detected" summary hand-listed 11 of `ServiceEntry`'s 13 fields and omitted exactly the two that decide where a service lives on disk (`ServiceEntry::declared_path`), so re-pointing a service in `services.yaml` was never listed as modified, and when the re-point did not also change the drift count, watch mode printed nothing at all. The comparison now delegates to the derived `PartialEq`, so it reads the struct definition instead of a copy of it, and a field added later cannot fall out of change detection the same way.
- **A broken `.svccat/policy.yaml` is no longer reported as an absent one** (PR #25). `PolicyConfig::load` swallowed both the read error and the parse error and returned `None`, so a policy file with a typo in it was indistinguishable from having no policy file at all: `svccat policy` printed *"No policy file found. Create .svccat/policy.yaml ..."*, about a file that exists, and exited 0; `svccat ci` dropped the `policy` step from its report and said "all checks passed", silently disabling the policy gate in a pipeline; `svccat scorecard` scored the repo with no policy contribution and said nothing. All three now name the file and the reason. `svccat policy` exits 2 (the CLI's existing code for a bad input), `svccat ci` reports the `policy` step as failed rather than skipped, and `svccat scorecard` warns and scores on. A genuinely absent policy file behaves exactly as before, and a policy file that exists but declares no fields now says so instead of claiming none was found.
- **`svccat watch`'s "Manifest changes detected" summary is no longer printed in a random order** (PR #27). The added and removed lists were collected straight out of `HashSet::difference`, whose iteration order is unspecified and is re-randomised per set by the default hasher, so adding `cache` and `worker` in one edit printed `+ 2 service(s): cache, worker` on one reload and `worker, cache` on the next with nothing about the manifest having changed. All three lists now follow the order of the manifest they were read from (added and modified follow the new `services:` list, removed follows the previous one), which is the rule the modified list already used, and each name is reported at most once even if `services.yaml` declares it twice.
- **`check --ping --format sarif` no longer computes ping results and throws them away** (PR #28). The SARIF renderer took the ping results and never used them, so a service failing its health check produced no SARIF result at all: the one output format wired into GitHub Code Scanning reported drift and stayed silent about reachability. Ping failures are now results under their own rules, emitted in the manifest order `ping::ping_services` walked so the document is deterministic for a given input.

### Security

- **DOM-based XSS in the HTML dependency graph** (PR #7). `graph --format html` embedded its node and link JSON by Rust `{:?}` Debug interpolation instead of the escaping writer used everywhere else, and the D3 tooltip wrote service metadata straight into `innerHTML`. A service `name`, `platform`, `team` or `language` in `services.yaml` could therefore execute script in whoever opened the generated report. Both the data island and the tooltip escape now, with regression tests asserting the breakout sequence is absent and the escaped form present for every affected field.
- **SSRF via HTTP redirect in `--ping` and webhooks** (PR #14). `ureq` follows redirects internally, so a URL that passed `validate_url` could be redirected to a private or loopback address with no re-validation of the new target, and the check that blocks private IPs applied only to the address the user typed. The new `src/safe_http.rs` disables automatic redirect-following and re-validates every hop before following it. Two pre-existing trust-boundary limits are unchanged and remain documented: the `localhost` development exception also applies to redirect targets, and non-IP-literal hostnames are still not resolved before the fetch (DNS rebinding).
- **HIGH severity: stack overflow on a cyclic policy rule `base` chain** (PR #16). A rule naming itself, or two rules naming each other, sent `RuleEngine::compile`'s inheritance resolver into unbounded recursion and killed the process with a stack overflow (`STATUS_STACK_OVERFLOW`, `0xc00000fd` on Windows) instead of returning an error, and it is reachable from the `services.yaml` of any repository being scanned. Resolution is now an iterative walk with cycle detection, and the case is pinned by a fuzz target plus committed crash-reproducer corpora.

## [1.5.0] - 2026-07-18

### Added

- **SPDX 2.3 JSON SBOM export (`--format spdx-json`)**: `svccat export --format spdx-json [--output <file>]` emits a schema-conformant SPDX 2.3 document with ISO 8601 timestamps, a unique `documentNamespace`, sanitized SPDXIDs, NTIA-friendly `supplier` fields, and `DESCRIBES` plus `DEPENDS_ON` relationships derived from the catalog's `depends_on` edges.
- **Snapshot SBOM sidecar (`snapshot save --sbom`)**: `svccat snapshot save <name> --sbom` also writes an SPDX 2.3 JSON SBOM beside the canonical snapshot at `.svccat/snapshots/<name>.spdx.json`; `svccat snapshot delete <name>` removes the sidecar too.

## [1.4.1] - 2026-07-09

### Fixed

- **Dependencies:** Resolved two vulnerability advisories (`RUSTSEC-2026-0204` in `crossbeam-epoch` / `rayon` and `RUSTSEC-2026-0190` in `anyhow`) by upgrading them inside `Cargo.lock` (Patch release).

## [1.4.0] - 2026-07-09

### Added

- **Multi-threaded Service Discovery (Phase 2 Performance):** Parallelized pattern discovery via `rayon` parallel iterators in `src/discovery.rs` to accelerate monorepo scans.
- **Ecosystem Export to Backstage (`--format backstage-yaml`)**: Support exporting services cleanly to Backstage multi-document `catalog-info.yaml` with `--output <file>` option.

### Changed

- `svccat export` now supports the `--output <file>` option across JSON, CSV, Markdown, and Backstage YAML formats.
 
## [1.3.2] - 2026-07-09

### Fixed

- Formatting: fix long warning line in `src/main.rs` to satisfy rustfmt check. (Patch release)

## [1.3.1] - 2026-07-09

### Fixed

- CI: ensure `svccat check` (installed from crates.io) won't fail PR runs when no `services.yaml` is present by exiting gracefully. (Patch release)

## [1.3.0] - 2026-07-09

### Added

- `svccat check --summary` - concise machine-friendly summary of the check results: declared, discovered, total drifts, and error/warning breakdown.

### Notes

- Minor, backwards-compatible feature release.

### Changed

- `svccat check` now exits gracefully (prints a warning) when no manifest is found in the repo root. This makes CI runs that invoke `svccat check` safe for repositories without a `services.yaml` file.

## [1.2.0] - 2026-07-09

### Changed

- Bumped crate version to 1.2.0.

### Notes

- Release notes autogenerated from commits since `1.1.7`. No additional user-facing changes were found since the last published version.

## [1.1.7] - 2026-06-27

### Added

- Regression coverage for CSV check rendering when drift `detail` contains
  newlines, ensuring multiline values remain correctly quoted.

## [1.1.6] - 2026-06-27

### Added

- Regression coverage for CSV check rendering when no drift exists, ensuring
  output still emits exactly the header row.

## [1.1.5] - 2026-06-27

### Changed

- CSV field escaping now also quotes values containing carriage returns (`\r`),
  aligning with RFC 4180 newline handling expectations.

### Added

- Regression coverage for carriage-return CSV field quoting.

## [1.1.4] - 2026-06-27

### Changed

- `svccat check --output <file>` now supports CSV output by using a shared
  string renderer for drift rows.

### Added

- Added CSV output string-rendering regression coverage in output and main-path
  tests to lock down escaping and header/row structure.

## [1.1.3] - 2026-06-27

### Added

- Added a shared-fixture output matrix integration test covering JSON, Slack,
  Teams, Datadog, and Mermaid output structures.

### Changed

- Added `render_check_to_string` helpers for Slack, Teams, and Datadog outputs
  to make integration-level payload validation straightforward.
- `svccat check --output <file>` now supports Slack, Teams, and Datadog
  formats in addition to JSON/Markdown.

---

## [1.1.2] - 2026-06-27

### Fixed

- Moved Mermaid module tests to the end of the file so strict clippy CI with
  `-D warnings` passes (`clippy::items_after_test_module`).

---

## [1.1.1] - 2026-06-27

### Added

- Focused output-layer regression tests for:
  - Slack payload rendering
  - Teams adaptive-card payload rendering
  - Datadog event payload rendering
  - CSV escaping helpers
  - Mermaid graph helper normalization and escaping

### Changed

- Output renderers now expose internal payload builder helpers used by module
  tests to reduce regression risk across supported output formats.

### Internal

- Formatting-only cleanup to maintain strict `cargo fmt --check` compliance.

## [1.1.0] - 2026-06-07

### Added

- **Metadata auto-inference in `init` and `fix`.** When scaffolding or
  remediating a manifest, svccat now infers each service's `platform` from
  deploy descriptors found in its directory (`fly.toml` → `fly.io`,
  `vercel.json` → `vercel`, `Chart.yaml` → `kubernetes-helm`, a `k8s/` directory
  → `kubernetes`, and others), in addition to the existing `language` inference.
  Inference is conservative: a field is only populated on an unambiguous signal,
  otherwise it stays a placeholder. A bare `Dockerfile` is not treated as a
  platform signal. This reduces `missing_field` drift in freshly generated
  manifests. Language inference now also recognises `setup.py` (Python) and
  `composer.json` (PHP).

---

## [1.0.1] - 2026-06-05

### Added

- **`homepage` and `documentation` metadata** in `Cargo.toml` so crates.io shows
  the Homepage and Documentation links alongside Repository. Metadata-only patch;
  no code or API changes.

---

## [1.0.0] - 2026-06-05

First stable release. The public API is now frozen under [semantic
versioning](https://semver.org/); see `docs/API_STABILITY.md` for exactly what is
covered.

There are **no functional or API changes since 0.23.0** - this release only
promotes the API surface curated in 0.23.0 to a stable `1.x` guarantee.

---

## [0.23.0] - 2026-06-05

This release prepares the public API for a `1.0.0` freeze. It is the last planned
window for breaking library changes before `1.0`.

### Changed

- **Curated public library API.** Only `manifest`, `discovery`, `drift`,
  `report`, and `config` are now documented as the stable, semver-covered API.
  The remaining modules are CLI implementation details and are marked
  `#[doc(hidden)]`; they remain reachable but are no longer part of the public
  contract. See the new `docs/API_STABILITY.md`.
- **Core types are now `#[non_exhaustive]`** (`Manifest`, `ServiceEntry`,
  `DriftItem`, `DriftReport`, `DriftKind`, `Severity`, `DiscoveredService`) so
  fields and variants can be added in future minor releases without a breaking
  change. `Manifest` and `ServiceEntry` now derive `Default` for ergonomic
  programmatic construction.
  - **Migration:** construct these types via `Default::default()` plus field
    assignment instead of struct literals, and add a wildcard arm when matching
    on `DriftKind` or `Severity`.
- **Replaced the unmaintained `serde_yaml` dependency** with the maintained
  `serde_yaml_ng` fork. No behavioural change; YAML parsing and serialization
  are unaffected.

### Added

- **`docs/API_STABILITY.md`** documenting exactly what semver covers for the
  library, the CLI, and the MSRV.
- **Crate-level documentation** with a runnable library usage example.
- **Declared MSRV** of Rust `1.85` via `rust-version` in `Cargo.toml`.

---

## [0.22.0] - 2026-06-03

### Added

- **`svccat demo`** - Zero-setup walkthrough: generates a throwaway sample monorepo
  (with deliberate drift) in a temp dir and runs `check`, `graph`, and `stats` against
  it, then cleans up (`--keep` retains the sample). Useful for first-run onboarding.
- **`examples/demo.rs`** - Library usage example (`cargo run --example demo`) showing
  how to load a manifest, discover services, and analyze drift through the crate API.

### Fixed

- **Windows stack overflow** - The CLI now runs on a worker thread with a 16 MB stack.
  clap's construction of the large `Commands` enum could exceed Windows' default 1 MB
  main-thread stack (Linux's 8 MB default masked this in CI and tests).

### Changed

- **README** - Condensed by ~75%: per-command deep dives removed in favor of
  `svccat <command> --help`, with a getting-started flow and a `svccat demo` pointer.

---

## [0.21.0] - 2026-06-03

### Added

- **`svccat workspace check`** - Analyze drift across multiple repositories in one pass.
  Define the repos in a `[workspace]` section of `svccat.toml`. Supports `--filter <repos>`,
  `--format`, `--fail-on-drift`, `--ignore`, `--depth`, and `--output`, and emits an aggregated
  report (terminal/JSON/Markdown) with declared services, errors, and warnings per repository.

- **Cross-repo dependency analysis** - New dependency-graph module surfaces `depends_on`
  relationships that span repositories in a workspace, flagging dangling and circular
  references across repo boundaries.

- **Composable policy rules** - A custom rule can now extend a `base` rule, so a shared
  condition is defined once and reused; derived rules inherit the base condition and add their own.

### Changed

- **`svccat watch`** - Now detects and reports services added or removed between runs,
  not just edits to the manifest file.
- **`svccat install-hooks`** - The installed pre-commit hook now runs drift analysis natively
  (structured results with error and warning counts), plus internal uninstall and
  install-status helpers.

---

## [0.20.0] - 2026-06-01

### Added

- **`svccat audit --cost-estimate`** - Analyze declared platforms and estimate monthly deployment costs.
  Outputs total cost and breakdown by platform. Supports `--format json` for integration with dashboards.
  Includes sensible defaults for common platforms (Cloud Run, Fly.io, GitHub Pages, AWS, etc.).

### Infrastructure

- Performance benchmarking with criterion, plus a GitHub Actions workflow that tracks results over time on the `gh-pages` branch.
- Code coverage workflow with Codecov integration.
- Comprehensive security integration tests and a security best-practices guide.

---

## [0.19.0] - 2026-05-28

### 🔒 Security (10 vulnerabilities addressed)

**⚠️ Important: Multiple critical and high-severity security fixes in this release.**

- **Git command injection (CRITICAL)** - Validate `--since` git references against strict allowlist pattern.
  Prevents injection via malicious git refs in compromised repositories.

- **SSRF in ping/webhooks (HIGH)** - Add URL validation module to block requests to private IP addresses
  (127.x, 10.x, 172.16-31.x, 192.168.x, ::1, fe80::/10, fc00::/7). Enforce `https://` for webhooks
  (except localhost for development). Prevents probing of internal infrastructure.

- **Deserialization bombs (HIGH)** - Add resource limits to prevent YAML/TOML expansion attacks:
  - Manifest files limited to 10 MB
  - Maximum 10,000 services per manifest
  - Service names limited to 256 bytes
  - `depends_on` lists limited to 1,000 entries
  - Config files limited to 1 MB

- **Path traversal (MEDIUM)** - Validate manifest paths to prevent `..` and absolute path attacks.
  Applies to service `path`, `submodule`, `docs`, and `ci` fields.

- **Symlink attacks (MEDIUM)** - Reject symlinks during service discovery to prevent
  directory traversal and time-of-check-time-of-use (TOCTOU) attacks.

- **Glob pattern DoS (MEDIUM)** - Limit discovery patterns to 20 total with max 2 consecutive
  wildcards. Prevents expensive glob expansion on untrusted manifests.

- **Information disclosure (MEDIUM)** - Path redaction module converts absolute paths to
  repo-relative in error messages, preventing system information leaks.

- **IPv6 loopback detection (MEDIUM)** - Properly detect and reject IPv6 loopback (::1)
  and link-local addresses (fe80::/10) in URL validation.

- **Cross-platform compatibility (MEDIUM)** - Convert Windows backslashes to forward slashes
  in git ref:path specifications for correct behavior on all platforms.

- **Dependency scanning (MEDIUM)** - Added `cargo audit` to GitHub Actions CI to catch
  vulnerable dependencies on every push and pull request.

### Code Quality Improvements

- Fixed 11 clippy warnings for improved code quality and maintainability
- Updated all GitHub Actions to Node.js 24 (actions/checkout@v4.1.7, Swatinem/rust-cache@v2.7.3)
- Optimized iterator patterns and removed redundant code branches
- Comprehensive test coverage: 69 passing tests (17 unit + 52 integration)

### Documentation

- **Added SECURITY.md** - Comprehensive security policy documenting threat model, attack vectors,
  all mitigations, known limitations, and responsible disclosure process.
- Updated CHANGELOG with detailed security fix descriptions and impact assessment.

**See SECURITY.md for complete threat model documentation and security recommendations.**

---

## [0.18.0] - 2025-07-21

### Added

- **`svccat scorecard`** - Per-service health scoring (completeness 40%, drift 40%, policy 20%).
  Outputs a ranked table to the terminal, or use `--format json/markdown` and `--output <file>`
  to write reports for dashboards and CI pipelines.

- **`svccat snapshot compare <BEFORE> <AFTER>`** - Side-by-side diff of two named snapshots.
  Reuses the existing diff renderer; supports `--format terminal/markdown`.

- **`svccat graph --format html`** - Self-contained HTML page with a D3.js v7 force-directed
  graph. Colour-coded by platform, hover tooltips, draggable nodes. Pipe to a file:
  `svccat graph --format html > services.html`.

- **`svccat ci --watch`** - File-watcher mode for CI. Re-runs the full CI check on every
  manifest change. Use `--interval <secs>` (default 2s debounce) to tune sensitivity.
  Exits non-zero if the last run had errors.

- **`svccat webhook`** - Fire a one-shot webhook with the current drift report as JSON payload.
  Use `--url <URL>` to override the URL in `svccat.toml`. Also fires automatically from
  `svccat check` and `svccat ci` when `[webhook]` is configured in `svccat.toml`.

- **`--output <file>`** on `svccat search` and `svccat deps`** - Write JSON output to a file
  instead of stdout, consistent with the existing `--output` flag on other commands.

### Changed

- `svccat check` and `svccat ci` now fire the configured webhook automatically on each run.

---

## [0.17.0] - 2025-07-20

### Added

- **`svccat ci`** - CI pipeline command that runs lint, drift, and policy checks in one pass.
  Returns exit code 1 if any errors are found. Use `--format json` for machine-readable output.

- **`svccat search <QUERY>`** - Search services by substring or `field:value` syntax.
  Searchable fields: `name`, `language`/`lang`, `platform`, `url`, `role`, `team`, `oncall`,
  `docs`, `ci`, `path`, `tags`, `depends_on`. Returns colored matches to the terminal.

- **`svccat snapshot diff <NAME>`** - Compare a named snapshot against the current state of
  the repo. Reports services added/removed/changed and drift items that appeared or resolved
  since the snapshot was taken. Supports `--format terminal` and `--format markdown`.

- **`--output <FILE>`** on `svccat check` and `svccat graph`** - Write output to a file
  instead of stdout. `check` supports `--format json` and `--format markdown` with `--output`.
  `graph` writes whichever graph format was requested directly to the specified file.

- **`svccat deps`** - Analyze inter-service dependencies declared via `depends_on`.
  Detects missing targets and circular dependency chains. Outputs a dependency summary to
  the terminal, as a Mermaid diagram, or as JSON.

- **`svccat tag add/remove`** - Mutate tags on services in the manifest YAML in-place.
  Tags are stored on `ServiceEntry` under the `tags` field (skipped when empty).

---

## [0.16.0] - 2025-07-18

### Added

- **`svccat policy`** - Enforce required and recommended fields across services using a
  `.svccat/policy.yaml` config file. Reports violations per service with severity levels
  (error/warning). Use `--format json` for machine-readable output and `--fail-on-violations`
  to gate CI pipelines on policy compliance.

- **`svccat snapshot save/list/delete`** - Named snapshot management in `.svccat/snapshots/`.
  Save the current drift report as a named snapshot, list all saved snapshots, or delete one
  by name. Snapshots capture manifest metadata and the full drift analysis for later comparison.

- **`--format datadog`** for `svccat check` - Emit a Datadog Events API JSON payload.
  Each drifting service becomes one event with `alert_type`, `tags`, and `priority` fields.
  A clean run emits a single success event. Pipe directly to `curl` to post events.

- **`--format json`** for `svccat report` - Machine-readable JSON output with manifest path,
  summary counts (declared/discovered/errors/warnings), and a team-grouped service listing.

- **`--filter <PATTERN>`** for `svccat graph` - Case-insensitive substring filter applied
  before rendering. Only services whose name contains the pattern are included in the graph.

- **`--interval <N>`** for `svccat watch` - Emit a synthetic re-check every N seconds in
  addition to filesystem events. Useful for catching remote changes (config maps, external
  registries) that do not touch local files.

---

## [0.15.0] - 2026-05-18

### Added

- **`svccat audit`** - Unified health check that runs lint + drift + optional URL ping in
  one pass and emits a scored report (0-100). Scoring: -10 per drift error, -3 per drift
  warning, -5 per lint error, -2 per lint warning, -5 per ping failure. Exits with code 1
  when any errors are present. Use `--format json` for machine-readable output.

- **`--format teams`** for `svccat check` - Emit a Microsoft Teams Adaptive Card JSON
  payload suitable for posting to a channel via an incoming webhook. The card includes a
  per-service drift summary table with colour-coded status indicators.

- **`--format markdown`** for `svccat diff` - Render the snapshot diff as GitHub-flavoured
  Markdown tables, suitable for pasting into PR comments or documentation.

- **`--notify`** for `svccat watch` - Send a native OS desktop notification whenever the
  drift count changes. Uses PowerShell on Windows, `osascript` on macOS, and `notify-send`
  on Linux. No additional dependencies required.

- **`--since <GIT_REF>`** for `svccat export` - Filter the export to only services that are
  new or have changed fields since the given git ref. Loads the historical manifest from git
  history and performs a field-level comparison, retaining only changed/added entries.

- **`--format plantuml`** for `svccat graph` - Emit a PlantUML component diagram. Services
  are grouped into `package` blocks by platform, with `..>` arrows for `depends_on`
  relationships. Paste the output at plantuml.com or pipe to `plantuml -pipe`.

---

## [0.14.0] - 2026-05-17

### Added

- **`svccat serve`** - Start a local HTTP server (`--port`, default 7777) that renders the
  live HTML drift report on every request. Use `--refresh N` to inject a
  `<meta http-equiv="refresh">` tag so the browser auto-reloads every N seconds.
  No extra dependencies - uses `std::net::TcpListener` from the standard library.

- **`svccat import --from openapi`** - Walk the repository for `openapi.yaml`, `openapi.yml`,
  `swagger.yaml`, and `swagger.yml` spec files. Extracts the service name (from `info.title`,
  slugified), URL (from `servers[0].url` for OpenAPI 3 or `host`+`basePath` for Swagger 2),
  and optional `x-team`, `x-oncall`, and `x-language` extension fields.

- **`svccat stats`** - Print a field-coverage summary table with ASCII bar charts showing
  what percentage of services have each metadata field set (language, platform, team, docs,
  url, role, oncall) plus an overall health score.

- **`--format slack`** for `svccat check` - Emit a Slack Block Kit JSON payload suitable
  for posting to a channel via the Slack API or an incoming webhook.

- **`svccat graph --format dot`** - Emit a Graphviz DOT digraph. Services are grouped into
  `subgraph cluster_N` blocks by platform. Pipe to `dot -Tsvg` or `dot -Tpng` to render.

- **`svccat watch --since <git-ref>`** - Each watch iteration now optionally compares the
  current drift against the manifest at the given git ref, displaying only newly introduced
  drift items (same behaviour as `svccat check --since`).

## [0.13.0] - 2026-05-16

### Added

- **`svccat fix`** - Auto-remediate simple drift. Adds `UndeclaredInRepo` services to the
  manifest with inferred language; use `--prune` to also remove `DeclaredMissingFromRepo`
  entries. Use `--dry-run` to preview changes without writing.

- **`svccat import --from docker-compose`** - Parse `docker-compose.yml` / `compose.yaml` at
  the repo root and generate service entries from each declared service. Handles both string
  and extended (`context:`) build paths, and maps `depends_on` (list or map form) to the
  manifest's `depends_on` field.

- **`svccat check --baseline <file>`** - Filter drift to only items absent from a saved
  baseline snapshot (JSON from `svccat export --format json`). Combine with `--fail-on-drift`
  to gate CI on regressions only - pre-existing drift is silently ignored.

- **`svccat install-hooks`** - Write a `.git/hooks/pre-commit` (or `--hook pre-push`) shell
  script that runs `svccat check --fail-on-drift` on every commit or push. On Unix the hook
  file is made executable automatically.

- **`--format csv`** for `svccat check` - Outputs drift items as RFC 4180 CSV
  (`service, severity, kind, message, detail`). Pipe into spreadsheets or ticket scripts.

- **`--format csv`** for `svccat export` - Outputs the service catalog as CSV
  (`name, language, platform, role, url, team, oncall`).

- **Two new `svccat lint` validators:**
  - Services with no `team` owner - warns when a service has no `team:` field.
  - Services with no `docs` reference - warns when a service has no `docs:` field.

- **Cleaner manifest serialization** - `svccat fix` (and any other command that rewrites the
  manifest) now omits `null` optional fields from the YAML output, producing much cleaner
  entries for newly added services.

---

## [0.12.0] - 2026-05-16

### Added

- **`svccat import --from backstage`** - Walk the repo for `catalog-info.yaml` files and generate
  service entries from every `kind: Component` entity found. Merges into an existing manifest
  without overwriting existing entries (use `--force` to replace the whole file).

- **`--format compact`** for `svccat check** - One line per service: status icon, name, and
  first drift kind. Ideal for large repos where the full terminal table is too noisy.

- **`--depth N`** for `svccat check`, `svccat export`, and `svccat watch` - Control how many
  directory levels deep discovery scans. Default is 1 (current behaviour). Set `--depth 2` to
  also detect services nested one level deeper (e.g. `services/team/auth-service`).

- **CI auto-format** - When `GITHUB_ACTIONS=true` is set and no explicit `--format` is given,
  `svccat check` automatically switches to `github-annotation` output so drift items appear as
  inline annotations on pull requests with zero extra configuration.

- **`upload-sarif` input for the GitHub Action** - Set `upload-sarif: true` in your workflow step
  to have the action generate a SARIF file and upload it to GitHub Code Scanning automatically.
  Requires Code Scanning to be enabled on the repository.

- **Extended language/build markers** - Discovery now recognises `build.gradle`,
  `build.gradle.kts`, and `pom.xml` (Java/Kotlin), `CMakeLists.txt` (C++),
  `Directory.Build.props` (.NET/C#), `Gemfile` (Ruby), `mix.exs` (Elixir), and
  `pubspec.yaml` (Dart/Flutter). `svccat init` infers the correct language for all
  of these.

- **Two new `svccat lint` validators:**
  - Duplicate `url` values - warns when multiple services share the same URL.
  - Cross-platform `depends_on` edges - warns when a service on one platform (e.g.
    `gcp-cloud-run`) declares a dependency on a service on a different platform (e.g. `fly.io`).

---

## [0.11.0] - 2026-04-20

### Added

- `svccat check --since <ref> --fail-on-new-drift` - exit 1 only on drift that is *new* since a
  given git ref, ignoring pre-existing items. Useful for incremental CI gates.
- `svccat check --format github-annotation` - GitHub Actions annotation output (warnings and
  errors appear inline on PRs).
- `svccat watch --team` - team-scoped continuous monitoring.

---

## [0.10.0] - 2026-04-06

### Added

- `svccat report --history N` - drift evolution table across the last N git commits.
- `svccat report --badge` - Shields.io Markdown badge snippet.
- `svccat report --format html --output report.html` - self-contained HTML report.

---

## [0.9.0] - 2026-03-23

### Added

- `svccat watch` - continuous drift detection; re-runs on file-system changes with 500 ms debounce.
- `svccat diff before.json after.json` - compare two `svccat export` snapshots.

---

## [0.8.0] - 2026-03-09

### Added

- `svccat check --format sarif` - SARIF 2.1.0 output for GitHub Code Scanning integration.
- `svccat check --format junit` - JUnit XML output for CI test reporters.
- `svccat check --format markdown` - Markdown table for PR comments.
- `svccat lint` - manifest structural validation (duplicate names, blank names, circular
  depends_on, self-referential deps, unknown version).

---

## [0.7.0] - 2026-02-23

### Added

- `policy.require_fields` in the manifest - make specific fields mandatory at the error level.
- `svccat check --ping` - HTTP reachability check for each service URL.
- `svccat graph --team` - scope the Mermaid diagram to a single team; cross-team nodes shown
  as external.

---

## [0.6.0] - 2026-02-09

### Added

- `svccat check --since <git-ref>` - compare current drift against the manifest at a past ref.
- `svccat check --team` - team-scoped drift check; suppresses `[UNDECLARED]` noise from other
  teams' services.
- `DanglingDependency` and `CircularDependency` drift kinds - validated against the full
  depends_on graph.

---

## [0.5.0] - 2026-01-26

### Added

- `svccat check --format json` - machine-readable drift output.
- `svccat export --format json` - full catalog snapshot for use with `svccat diff`.
- `svccat check --ignore` - glob-based exclusion patterns (repeatable).
- `svccat.toml` workspace config - `format`, `fail_on_drift`, and `ignore` keys.

---

## [0.4.0] - 2026-01-12

### Added

- `svccat graph` - Mermaid dependency diagram grouped by platform.
- `svccat graph --format markdown` - Markdown table alternative.
- `svccat export` - save a catalog snapshot.
- `svccat report` - full per-team ownership and drift report.

---

## [0.3.0] - 2025-12-29

### Added

- `svccat init` - scaffold a `services.yaml` from the current repo with language inference.
- `svccat completions <shell>` - shell completion scripts (bash, zsh, fish, PowerShell).
- GitHub Action (`action.yml`) - composite action that installs svccat and runs `svccat check`.

---

## [0.2.0] - 2025-12-15

### Added

- `svccat check --fail-on-drift` - exit code 1 on any drift; suitable for CI gating.
- `MissingField` drift kind for `role`, `language`, and `platform`.
- `MissingReferencedFile` drift kind for `docs:` and `ci:` paths that do not exist.
- `PolicyViolation` drift kind for `policy.require_fields` enforcement.

---

## [0.1.0] - 2025-12-01

### Added

- Initial release.
- `svccat check` - compare a `services.yaml` manifest against directories discovered in the repo.
- `DeclaredMissingFromRepo` and `UndeclaredInRepo` drift detection.
- Terminal, colored output.
