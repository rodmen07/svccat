# svccat roadmap

Last updated: 2026-07-26. This file is the single source of planning truth for svccat.
Older planning docs under `docs/` carry status banners pointing here and are kept as
historical records only.

Every claim in this file that a machine can check is now checked: `tests/roadmap_truth.rs`
reads this document against `Cargo.toml` and `CHANGELOG.md` and fails the build when they
disagree. A one-time reconciliation rots (this one did, twice, within two days of being
written); a guarded one cannot rot silently.

The public 1.x API (library and CLI) is frozen under semver. The freeze is defined in
[docs/API_STABILITY.md](docs/API_STABILITY.md); this roadmap does not restate it. MSRV is
*declared* as Rust 1.85, is enforced by nothing, and does not currently hold — see v1.7.0.

## Current state (2026-07-26)

- Crate version on main: **v1.6.0** (`Cargo.toml` `[package] version`). Pinned to this
  document by `roadmap_current_state_matches_the_crate_version`, so a release-prep PR
  that bumps one and not the other fails the build.
- Published to crates.io: **1.6.0**, 2026-07-26 (annotated tag `v1.6.0` = `fe59cd5`,
  the release-prep commit, deliberately NOT `main`). Verified after the publish via
  the crates.io API: `"newest_version":"1.6.0"`, `"max_stable_version":"1.6.0"`.
  Earlier: v1.5.0, 2026-07-18 (tag `v1.5.0` = merge commit `60c56b2`), which delivered
  SPDX 2.3 JSON SBOM export (`export --format spdx-json`), a `snapshot save --sbom`
  sidecar with delete cleanup, and a shared `timefmt` module.
- **v1.6.0 is CUT and PUBLISHED (2026-07-26).** The 24 commits that had piled up on
  `main` since the `v1.5.0` tag were written down by release-prep PR #30 and are now
  shipped: `CHANGELOG.md` carries `## [1.6.0] - 2026-07-26` naming all nine
  previously-unrecorded user-facing changes. `cargo install svccat` therefore finally
  delivers the three security fixes (DOM-based XSS in the HTML graph, PR #7; SSRF via
  HTTP redirect in `--ping` and webhooks, PR #14; the HIGH cyclic-`base` stack
  overflow, PR #16) that reached nobody for the eight days 1.5.0 was newest. The
  version question was answered MINOR by the user on 2026-07-26. `publish.yml` run
  `30196265195` is green end to end, including the three `Post-Publish Registry
  Validation` legs that `cargo install svccat --force` straight from the registry and
  run `svccat --version` on ubuntu, windows and macos.
- The tag is at `fe59cd5`, not at `main`, on purpose: `main` had already taken PR #31
  (snapshot-diff ordering), which is listed under `## [Unreleased]` and ships in the
  next release, so published 1.6.0 contains exactly what its own changelog section
  describes. See `## Unreleased on main` for what that leaves pending.
- Shipped in earlier releases during 2026-06 and 2026-07 (the crate is NOT in frozen
  maintenance mode):
  - v1.1.0 (2026-06-07): language and platform inference in `init` and `fix`.
  - v1.1.1 to v1.1.7 (all dated 2026-06-27): Slack, Teams, Datadog, and CSV support for
    `check --output`, CSV escaping fixes, and output-layer regression coverage. Note:
    no git tags exist for the 1.1.1 to 1.1.7 series; they are changelog releases only.
  - v1.2.0 (2026-07-09): version bump only; the changelog records no user-facing
    changes. v1.3.0 (2026-07-09): `check --summary`; CI-safe graceful exit when no
    manifest is present. v1.3.1 and v1.3.2 (2026-07-09): graceful-exit follow-up fix
    and rustfmt patch.
  - v1.4.0 (2026-07-09): rayon multi-threaded service discovery; `export --format
    backstage-yaml`; `export --output <file>` for all formats.
  - v1.4.1 (2026-07-09): security patch resolving the RUSTSEC-2026-0204
    (crossbeam-epoch via rayon) and RUSTSEC-2026-0190 (anyhow) lockfile advisories.
- Workflows: `Continuous Integration & Verification` (the build/test matrix, the
  `Lint (fmt + clippy)` gate added by PR #26, the `Security audit` `cargo audit --deny
  warnings` gate added by PR #17, and a service-catalog drift check), `Code Coverage`,
  `Performance Benchmarks` (meaningful signals since v0.21.0; a benchmark failure is a
  real regression, not noise), `Continuous Fuzzing` (real since PR #15 and seeded from
  the committed corpora since PR #22), and `publish` (runs on `v*` tag push). `main` is
  branch-protected with the required contexts set live 2026-07-20.

## Working agreements

- Track: stable 1.x crate on a light maintenance-plus-small-minors track, at roughly
  one minor version per week.
- Every milestone below is sized for one or two small PRs.
- Releases follow the documented flow under the standing Merges-and-releases
  delegation (2026-07-18): merge to main on green CI, push the `v*` tag (which
  fires `publish.yml`), create the GitHub release. What stays USER-ONLY is
  writing the repo secret `CRATES_IO_TOKEN` (`gh secret set`, interactive, never
  pasted into chat or a file an agent reads); no secret value is ever handled by
  an agent. (The prior blanket "all releases are USER-ONLY, never by an agent"
  line was stale: v1.5.0 and every other crate release since 2026-07-18 shipped
  under this delegation.)
- File-edit gate LIFTED (was: "BLOCKED until v1.5.0 is merged, tagged, and
  published: any change touching CHANGELOG.md, Cargo.toml, Cargo.lock, or
  README.md"). That block was premised on the v1.5.0 release-prep commit sitting
  unpushed on `feat/spdx-sbom`; that branch merged as PR #3 (`60c56b2`) and was
  deleted, v1.5.0 is published, so there is nothing left to collide with. Edits
  to CHANGELOG.md, Cargo.toml, and Cargo.lock now proceed under the normal
  branch-plus-PR flow.
- README.md uses CRLF line endings; prefer not to edit README.md at all.
- This document is machine-checked. `tests/roadmap_truth.rs` guards four claims: the
  crate version, that no released version is still listed as an upcoming milestone,
  that no released version sits in a `BLOCKED` row, and that `## Unreleased on main`
  exists exactly when `CHANGELOG.md` has `[Unreleased]` entries. Keep the headings it
  anchors on intact when editing: `## Current state`'s `- Crate version on main:`
  line, `## Milestones`, and `## Blocked and user-only summary`. The fourth,
  `## Unreleased on main`, is the one heading that is *supposed* to come and go: it
  exists exactly while `CHANGELOG.md` has `[Unreleased]` entries, and the guard fails
  if it lingers after a release is prepared or is missing while work sits unpublished.

## Milestones

### v1.6.1: coverage improvements

Pure test PRs; no API risk under the 1.x freeze.

- Pull the latest coverage report, identify the 2-3 lowest-covered `src/` modules,
  and add unit or regression tests for the worst one. (Note 2026-07-26: the QA stream
  has been working the *untested* surfaces ahead of the lowest-covered ones — `src/watch.rs`
  got its first tests in PR #21/#24 and `src/output/sarif.rs` in PR #28, each finding a
  live defect. Re-derive the target list from a fresh coverage run rather than from
  this bullet.)
- Add edge-case tests for the SBOM surface: empty catalog, services without
  `depends_on`, SPDXID sanitization collisions, sidecar delete when the snapshot is
  missing.
- Release per the flow in Working agreements (or fold into the next minor).

Done when: coverage on the targeted modules measurably improves and all tests pass.

### v1.7.0: dependency currency, part 1 (notify and criterion) — PR 1 shipped

Direct dependencies have aging majors. Bumps keep RUSTSEC exposure down. Split across
two milestones so each stays at one or two small PRs.

- Run `cargo outdated` and `cargo audit`; record the bump list in the PR description.
- **PR 1 SHIPPED 2026-07-25 (PR #21, `d622555`): notify 6.1.1 → 8.2.0**, still the
  current stable major (re-verified 2026-07-26: crates.io `max_stable_version` = 8.2.0;
  9.0.0 remains release-candidate only). No call-site migration was needed — the
  `Config` / `RecommendedWatcher` / `RecursiveMode` / `EventKind` surface `src/watch.rs`
  and `src/ci.rs` use is unchanged across both majors — so the PR's substance is the
  coverage proving the swapped backend still works, run by the CI matrix on inotify,
  `ReadDirectoryChangesW` and FSEvents. Supply chain shrank 213 → 212 crates.
- **PR 2 remaining: bump criterion `0.5` → `0.8.2`** (crates.io `max_stable_version`,
  verified 2026-07-26). Dev-only, so it cannot affect the frozen API; expect benchmark
  harness churn in `benches/` rather than in `src/`.
- **MSRV: `Cargo.toml` declares `rust-version = "1.85"` and that is fiction.**
  `idna_adapter` 1.2.2 and the `icu_*` 2.2.0 crates it pulls in via `url` declare
  `rust-version = "1.86"`, and they were on `origin/main` before the notify bump, so
  `cargo +1.85 build` cannot succeed against the committed lockfile. Nothing enforces
  the floor: all six files in `.github/workflows/` use only
  `dtolnay/rust-toolchain@stable`, `@nightly`, or the `[stable, beta, nightly]` matrix,
  and there is no `rust-toolchain.toml`. slokit's `MSRV 1.82` job, with its documented
  rust-version-aware resolve-and-pin recipe, is the pattern to copy. Close condition:
  an `MSRV <version>` job exists in `ci.yml`, is green on its own PR with before/after
  numbers, is added to `main`'s required contexts *after* it first posts a status, and
  `rust-version` matches what that job actually builds.
- Release per the flow in Working agreements.

Done when: notify and criterion are on current majors, tests pass, and the declared
MSRV is one a CI job proves.

### v1.8.0: dependency currency, part 2 (ureq and colored)

Same rules as v1.7.0; one runtime major per PR.

- PR 1: bump `ureq` `2` → `3.3.0` (crates.io `max_stable_version`, verified 2026-07-26)
  with call-site migration and tests. This one touches `src/safe_http.rs`, whose whole
  purpose is disabling redirect-following, so the migration must re-prove the SSRF
  guard rather than assume it survives — `tests/redirect_ssrf_tests.rs` is the gate.
- PR 2: bump `colored` `2` → `3.1.1` (crates.io `max_stable_version`, verified
  2026-07-26).
- Confirm the declared MSRV still holds after each bump; run `cargo audit`.
- Release per the flow in Working agreements.

Done when: no stale direct-dependency majors remain (or a skip decision is recorded),
`cargo audit` is clean, and the declared MSRV is verified.

## Unreleased on main

Merged to `main`, written down in `CHANGELOG.md` under `## [Unreleased]`, and not in
any published version. This section exists exactly while that is true, enforced by
`roadmap_declares_unreleased_work_exactly_when_the_changelog_has_some`; a release-prep
PR moves these entries into the new version's section and deletes this heading.

- **SARIF absolute-path URIs** (2026-07-26). `check --format sarif` under an absolute
  `--root` wrote the bare filesystem path into every `artifactLocation.uri`, which on
  Windows parses as a URI with a one-letter scheme. Absolute manifest paths are now
  relativised against the run root when possible and emitted as percent-encoded
  `file://` URIs otherwise; relative paths are unchanged. Targets the next release
  after v1.6.0.
- **`svccat snapshot diff` drift-list ordering and format** (2026-07-26). The
  `snapshot diff` path built `DiffReport::new_drift` / `resolved_drift` by
  differencing two `HashSet`s, so the lists shuffled between runs over identical
  input, and it rendered them as the bare `service:message` key while `svccat diff`
  rendered the severity-prefixed line. Both paths now share one deterministic,
  deduplicating builder, guarded by `tests/diff_drift_order_tests.rs`. Targets the
  next release after v1.6.0.
- **Policy loader resource limits** (2026-07-26). `.svccat/policy.yaml` is now read
  under the posture `Manifest::load` already had: a 1 MiB size cap enforced on the
  bytes read, before the parser runs, plus post-parse bounds on the declared field
  count and field-name length, reported as the new `PolicyLoadError::Limit` variant.
  Closes the hardening-asymmetry bug found while shipping the `fuzz_policy` target
  (filed alongside PR #20). Targets the next release after v1.6.0.

## Later / candidates (no version assigned)

Unshipped ideas on record. Pull forward only if the user chooses feature work over
pure maintenance.

None currently open. Four items previously listed here (policy rule schema
validation, the `workspace check --format html` visualization, CycloneDX JSON
export, and SSRF redirect-hardening for `--ping`/webhooks) have shipped; see
History and supersession below for their PRs and merge commits.

## Blocked and user-only summary

The v1.5.0 in-flight release gate that previously filled this table CLEARED on
2026-07-18 when v1.5.0 shipped, so the edit blocks it imposed (CHANGELOG.md,
Cargo.toml, Cargo.lock, the CHANGELOG reorder, and the v1.7.0/v1.8.0 dependency
bumps) are all lifted. What remains:

| Item | Status | Reason |
|------|--------|--------|
| Writing the `CRATES_IO_TOKEN` repo secret | USER-ONLY | Secret values are never handled by an agent; set via `gh secret set` (interactive) |
| Tag push, GitHub release, `cargo publish` in general | Delegated | Follow the release flow in Working agreements; merge and tag only on green CI |
| Editing README.md | Avoid | CRLF line endings; no roadmap work needs it |

## History and supersession

- **v1.6.0 "publish what is already on main" — SHIPPED 2026-07-26.** Release prep landed
  as PR #30 and the cut followed the same day: annotated tag `v1.6.0` at `fe59cd5`,
  `publish.yml` run `30196265195` green, crates.io `"newest_version":"1.6.0"`, GitHub
  release published. Its whole task
  list is done: `CHANGELOG.md` gained `## [1.6.0] - 2026-07-26` naming all nine
  user-facing changes that were on `main` and in no released version (six features,
  three security fixes, four behaviour fixes, one dependency major); the changelog was
  reordered into strictly descending version order (the 1.2.0 / 1.3.x cluster had been
  out of order since 2026-07-09 and survived two hygiene milestones as a hand task), and
  is now pinned there by `changelog_versions_are_in_strictly_descending_order`, so the
  third reconciliation is the last one; `SECURITY.md`'s "v0.19.0 (Planned)" boxes,
  unchecked since 2026-05-28 although v0.19.0 shipped all five that day, were resolved
  (with the honest note that the `--follow-symlinks` opt-out was never built, because
  discovery rejects symlinks unconditionally) and this release's own three security
  fixes were added to that file's fix changelog; and `Cargo.toml` went to 1.6.0.
  The MINOR-versus-MAJOR call was an overridable default this document routed to the
  user, who answered **MINOR** on 2026-07-26. The
  argument is restated here in full so it does not disappear with the milestone that
  held it, and so the classification is not relitigated: **MINOR.** Every library change since 1.5.0 is additive
  (`PolicyConfig::load_checked` and `PolicyLoadError` are new; no existing signature,
  type or field changed), and the behaviour changes are fixes to output ordering, to an
  exit code on input that was already broken, and to a format that was emitting nothing.
  Nothing in the frozen surface described by `docs/API_STABILITY.md` was removed or
  narrowed. If the `svccat policy` exit-code change (0 to 2 on a policy file that exists
  but does not parse, PR #25) had been judged breaking for CLI consumers rather than a
  fix, this would have been 2.0.0; the user classified it a fix, on the grounds that the
  old exit 0 was a false report of success about a file it could not read, so no
  consumer could have depended on it deliberately.
- **v1.6.0 "make fuzzing real" — COMPLETE and PUBLISHED (2026-07-26).** The 2026-07-18 audit found the fuzzing setup was a stub that could
  never have built. Delivered across six PRs: PR #15 (`f840161`) created
  `fuzz/Cargo.toml`, made `fuzz_glob` / `fuzz_manifest` / `fuzz_url` build, renamed the
  workflow to `Continuous Fuzzing` and replaced the placeholder run step with real
  time-boxed runs; widening `fuzz_manifest` from parse-only to parse-then-compile
  surfaced the HIGH cyclic-`base` stack overflow fixed in PR #16 (`32f2bca`); PR #19
  (`f1a3e08`) committed seed/regression corpora plus a PR-time replay suite; PR #20
  (`37cdcb3`) added the `fuzz_policy` target; PR #22 (`ff7e9ae`) wired the campaign to
  actually read the committed corpora; PR #23 (`e08852b`) rewrote `docs/FUZZING.md` and
  pinned it with two doc guards. Both done-when conditions verified 2026-07-26:
  `fuzz/fuzz_targets/` holds `fuzz_policy.rs` alongside the other three, and
  `fuzz/corpus_seeds/` holds all four target directories (7 / 9 / 11 / 10 seeds), with
  `Continuous Fuzzing` green on `main` at `eb9b9c1`. **Note for anyone reading the
  history: "committed seed corpora exist" was true from PR #19 while the fuzzer was
  still ignoring them — existence and use are separate claims, and only PR #22 made the
  second one true.** The milestone shares its version number with the release cut above:
  both landed in the 1.6.0 that shipped on 2026-07-26.
- **v1.5.0: SPDX SBOM release — SHIPPED 2026-07-18** (moved out of Milestones
  2026-07-26). `feat/spdx-sbom` merged as PR #3 (merge commit `60c56b2`), tag `v1.5.0`
  pushed, the publish workflow pushed 1.5.0 to crates.io, and the GitHub release is
  published. All three done-when conditions met: the tag exists, crates.io lists 1.5.0
  (re-verified 2026-07-26), and the GitHub release is published.
- **v1.5.1 "docs and changelog hygiene" — RETIRED 2026-07-26, folded into v1.6.0.** It
  held two live tasks (the CHANGELOG descending-order reorder and the stale SECURITY.md
  "v0.19.0 (Planned)" boxes) and one already-done note (status banners added 2026-07-18
  to `docs/RELEASE_PLAN_V1.4.0.md`, `docs/FEATURE_DESIGN_MULTI_REPO.md` and
  `docs/PERFORMANCE_OPTIMIZATIONS_PHASE1.md`). Its own product note already recommended
  folding it in, because features — not patches — are what sit unreleased. Cutting a
  1.5.1 patch now would publish a version number strictly older than nine unreleased
  user-facing changes. Both live tasks are carried verbatim into the v1.6.0 task list;
  nothing was dropped.
- Prior notes described svccat as "v1.0.1, maintenance mode". That direction is
  superseded: the repo shipped v1.1.0 through v1.5.0 across 2026-06 and 2026-07,
  and further features now sit unreleased on main toward the next minor (see
  Current state). Current direction is stable 1.x with small weekly minors, not a
  frozen crate.
- docs/RELEASE_PLAN_V1.4.0.md: features 1 and 2 shipped in v1.4.0 on 2026-07-09.
  Feature 3 (policy rule schema validation folded into `svccat lint`) shipped
  2026-07-20 via PR #12 (squash commit `da3d537`). Feature 4 (`workspace check
  --format html` interactive visualization reusing the existing D3 graph HTML
  renderer) shipped 2026-07-20 via PR #6 (squash commit `8f625fc`), hardened by
  PR #7 (DOM-based XSS fix in the shared HTML/mermaid renderer, commit
  `e97a67b`), PR #8 (binary-level CLI integration tests via assert_cmd, commit
  `23cccff`), and PR #10 (CI now builds and tests this checkout instead of only
  the published crate, commit `8c6dc20`). The circular-base-rules security gate is
  closed at the code level by PR #16 (`32f2bca`) and guarded by the `fuzz_policy`
  target from PR #20 (`37cdcb3`).
- CycloneDX JSON export as a sibling to `spdx-json` (previously listed under
  Later / candidates, not carried from any prior planning doc) shipped
  2026-07-20 via PR #11 (squash commit `4202db6825a6c18c66be7ecdcd70f45036e70dcc`).
- SSRF redirect-hardening for `--ping`/webhooks (carried from the
  RELEASE_PLAN_V1.4.0 security gates, previously listed under Later /
  candidates): fix opened 2026-07-20 as PR #14 (new `src/safe_http.rs`
  disables `ureq`'s automatic redirect-following and re-validates every
  redirect target with `urlvalidation::validate_url` before following it),
  MERGED 2026-07-21 (merge commit `c925000`). Two pre-existing trust-boundary
  caveats it deliberately left unchanged are recorded in the autodev backlog:
  the `localhost` dev-exception in `urlvalidation.rs` still allows a redirect
  to `localhost:<port>`, and the DNS-rebinding gap (non-IP-literal hostnames
  are not resolved before the fetch) is unchanged.
- docs/FEATURE_DESIGN_MULTI_REPO.md: shipped in v0.21.0 on 2026-06-03; historical
  design record only. The three multi-repo slices that landed after it (PRs #4, #5
  and #6) are recorded in the `## [1.6.0]` CHANGELOG entry.
- docs/PERFORMANCE_OPTIMIZATIONS_PHASE1.md: Phase 1 work was completed 2026-05-30
  and released in v0.20.0 on 2026-06-01; the Phase 2 follow-up (parallel discovery)
  shipped in v1.4.0 on 2026-07-09; historical record only.
- The old note that "Performance Benchmarks always fail, red herring" is obsolete:
  benchmark failures are real signals since v0.21.0.
- The 2026-06-04 GCP and Fly.io infrastructure decommission does not affect svccat:
  it is a pure CLI and library crate with GitHub-Actions-only CI and no deployed
  runtime.
