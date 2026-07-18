# svccat roadmap

Last updated: 2026-07-18. This file is the single source of planning truth for svccat.
Older planning docs under `docs/` carry status banners pointing here and are kept as
historical records only.

The public 1.x API (library and CLI) is frozen under semver. The freeze is defined in
[docs/API_STABILITY.md](docs/API_STABILITY.md); this roadmap does not restate it. MSRV
is Rust 1.85.

## Current state (2026-07-18)

- Latest published release: **v1.4.1** (tagged 2026-07-09), a security patch resolving
  the RUSTSEC-2026-0204 (crossbeam-epoch via rayon) and RUSTSEC-2026-0190 (anyhow)
  lockfile advisories.
- Shipped during 2026-06 and 2026-07 (the crate is NOT in frozen maintenance mode):
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
- **In flight, unreleased:** branch `feat/spdx-sbom` (4 local commits ahead of origin
  as of 2026-07-18) implements SPDX 2.3 JSON SBOM export (`export --format spdx-json`),
  a `snapshot save --sbom` sidecar with delete cleanup, a shared `timefmt` module, and
  tests. The v1.5.0 release-prep commit (version bump, changelog entry, README updates)
  was committed locally on 2026-07-18 (commit ec727fe). The branch is NOT yet pushed,
  merged, tagged, or published.
- Workflows: CI, Code Coverage, Performance Benchmarks (meaningful signals since
  v0.21.0; a benchmark failure is a real regression, not noise), a scheduled fuzzing
  workflow, and publish (runs on `v*` tag push). Caveat found 2026-07-18: the fuzzing
  workflow is a stub. Three target sources exist (`fuzz_glob`, `fuzz_manifest`,
  `fuzz_url`) but there is no `fuzz/Cargo.toml`, no committed corpus, and the
  workflow's run step is a placeholder that never executes the targets. See v1.6.0.

## Working agreements

- Track: stable 1.x crate on a light maintenance-plus-small-minors track, at roughly
  one minor version per week.
- Every milestone below is sized for one or two small PRs.
- All releases are USER-ONLY: committing release prep, merging to main, tagging,
  creating GitHub releases, and `cargo publish` are always done by the user, never
  by an agent.
- BLOCKED until v1.5.0 is merged, tagged, and published: any change touching
  CHANGELOG.md, Cargo.toml, Cargo.lock, or README.md. The v1.5.0 release-prep
  commit for these files exists locally on `feat/spdx-sbom` as of 2026-07-18, and
  agent edits to them before the release lands would collide with it.
- README.md uses CRLF line endings; prefer not to edit README.md at all.

## Milestones

### v1.5.0: SPDX SBOM release (finalize and ship)

The feature is code-complete with tests on `feat/spdx-sbom`, and the release-prep
commit (ec727fe, 2026-07-18) is in local history; only release mechanics remain, and
they gate everything touching the release files.

- DONE 2026-07-18: the release-prep commit for CHANGELOG.md, Cargo.lock, Cargo.toml,
  and README.md exists locally on `feat/spdx-sbom`.
- USER-ONLY: push `feat/spdx-sbom` and merge to main.
- USER-ONLY: tag v1.5.0, let the publish workflow push to crates.io, create the
  GitHub release.
- BLOCKED for agents until the above: any edit touching CHANGELOG.md, Cargo.toml,
  Cargo.lock, or README.md.
- Agent-doable now: run the local gates (cargo test, cargo fmt --check, cargo clippy)
  on the branch head and report any failures. CI results for the 4 unpushed commits
  do not exist until the USER-ONLY push happens.

Done when: the v1.5.0 tag exists, crates.io lists 1.5.0, and the GitHub release is
published.

### v1.5.1: docs and changelog hygiene

Cheap patch-level cleanup of drift found in the 2026-07-18 audit; one small PR.

- Reorder CHANGELOG.md entries into descending version order (1.2.0 currently sits
  between 1.4.0 and 1.3.0, and the 1.3.x entries are scrambled). BLOCKED until
  v1.5.0 is merged, tagged, and published.
- Mark the SECURITY.md "v0.19.0 (Planned)" checklist as delivered with the actual
  ship date (v0.19.0 shipped 2026-05-28 per CHANGELOG.md); all five checklist items
  appear in the 0.19.0 changelog entry, but the boxes were never checked.
- Already done 2026-07-18 (no further action): status banners added to
  docs/RELEASE_PLAN_V1.4.0.md, docs/FEATURE_DESIGN_MULTI_REPO.md, and
  docs/PERFORMANCE_OPTIMIZATIONS_PHASE1.md.
- USER-ONLY: tag and publish the patch release.

Done when: CHANGELOG.md is strictly descending, SECURITY.md has no stale unchecked
"Planned" boxes, and the patch is published.

### v1.6.0: make fuzzing real

Audit finding (2026-07-18): the fuzzing setup is a stub. The 3 target sources exist
under `fuzz/fuzz_targets/`, but there is no `fuzz/Cargo.toml`, so the targets cannot
even build, and the workflow's run step is a placeholder that installs svccat from
crates.io and executes nothing. This milestone also carries the unshipped security
gate from docs/RELEASE_PLAN_V1.4.0.md. Two small PRs:

- PR 1: create `fuzz/Cargo.toml` (standard cargo-fuzz layout; it is a separate
  crate, so the root Cargo.toml stays untouched and the v1.5.0 release gate does
  not apply), make the 3 existing targets compile, and replace the placeholder
  run step in `.github/workflows/fuzzing.yml` with real time-boxed
  `cargo fuzz run` invocations.
- PR 2: add a `fuzz_policy` target covering `.svccat/policy.yaml` rule parsing,
  including circular base-rules (carried from the RELEASE_PLAN_V1.4.0 security
  gates); seed and commit small corpus directories for all 4 targets from existing
  test fixtures; rewrite docs/FUZZING.md to describe the actual setup instead of
  the aspirational one.
- USER-ONLY: tag and publish.

Done when: 4 fuzz targets build and run green in the fuzzing workflow with
committed corpora, and docs/FUZZING.md matches the real setup.

### v1.6.1: coverage improvements

Pure test PRs; no API risk under the 1.x freeze.

- Pull the latest coverage report, identify the 2-3 lowest-covered `src/` modules,
  and add unit or regression tests for the worst one.
- Add edge-case tests for the new SBOM surface: empty catalog, services without
  `depends_on`, SPDXID sanitization collisions, sidecar delete when the snapshot is
  missing.
- USER-ONLY: tag and publish (or fold into the next minor).

Done when: coverage on the targeted modules measurably improves and all tests pass.

### v1.7.0: dependency currency, part 1 (notify and criterion)

Direct dependencies have aging majors (notify 6, colored 2, ureq 2, criterion 0.5).
Bumps keep RUSTSEC exposure down. Split across two milestones so each stays at one
or two small PRs. BLOCKED until v1.5.0 is merged, tagged, and published (both parts
touch Cargo.toml and Cargo.lock).

- Run `cargo outdated` and `cargo audit`; record the bump list in the PR description.
- PR 1: bump notify to the current major with call-site migration and tests.
- PR 2: bump criterion (dev-only; cannot affect the frozen API).
- Confirm MSRV 1.85 still holds after each bump; document any required MSRV change
  per the docs/API_STABILITY.md policy.
- USER-ONLY: tag and publish.

Done when: notify and criterion are on current majors, tests pass, and MSRV 1.85
is verified.

### v1.8.0: dependency currency, part 2 (ureq and colored)

Same gate and rules as v1.7.0: BLOCKED until v1.5.0 is merged, tagged, and
published; one runtime major per PR.

- PR 1: bump ureq to the current major with call-site migration and tests.
- PR 2: bump colored to the current major.
- Confirm MSRV 1.85 still holds after each bump; run `cargo audit`.
- USER-ONLY: tag and publish.

Done when: no stale direct-dependency majors remain (or a skip decision is recorded),
`cargo audit` is clean, and MSRV 1.85 is verified.

## Later / candidates (no version assigned)

Unshipped ideas on record. Pull forward only if the user chooses feature work over
pure maintenance.

- Policy rule schema validation folded into `svccat lint` (carried from
  docs/RELEASE_PLAN_V1.4.0.md item 3).
- `workspace check --format html` interactive visualization reusing the existing D3
  graph HTML renderer (carried from docs/RELEASE_PLAN_V1.4.0.md item 4).
- CycloneDX JSON export as a sibling to `spdx-json` (a new ExportFormat value is
  additive and allowed under the 1.x freeze).
- SSRF redirect-hardening verification pass for `--ping` (carried from the
  RELEASE_PLAN_V1.4.0 security gates).

## Blocked and user-only summary

| Item | Status | Reason |
|------|--------|--------|
| Push, merge, tag, and publish of v1.5.0 | USER-ONLY | Releases are manual by policy |
| Any tag, GitHub release, or `cargo publish` (all milestones) | USER-ONLY | Releases are manual by policy |
| Edits to CHANGELOG.md, Cargo.toml, Cargo.lock, README.md | BLOCKED | The v1.5.0 release-prep commit sits unpushed on `feat/spdx-sbom` as of 2026-07-18; edits before the release lands would collide with it |
| CHANGELOG.md reorder (v1.5.1) | BLOCKED | Same in-flight-release gate |
| Dependency bumps (v1.7.0 and v1.8.0) | BLOCKED | Touch Cargo.toml and Cargo.lock; same gate |
| Editing README.md at all | Avoid | CRLF line endings and in-flight release changes; no roadmap work needs it |

## History and supersession

- Prior notes described svccat as "v1.0.1, maintenance mode". That direction is
  superseded: the repo shipped v1.1.0 through v1.4.1 during 2026-06 and 2026-07 and
  has v1.5.0 in flight. Current direction is stable 1.x with small weekly minors,
  not a frozen crate.
- docs/RELEASE_PLAN_V1.4.0.md: features 1 and 2 shipped in v1.4.0 on 2026-07-09;
  features 3 and 4 plus the fuzz_manifest circular-base-rules expansion never
  shipped and are carried above (v1.6.0 and Later / candidates).
- docs/FEATURE_DESIGN_MULTI_REPO.md: shipped in v0.21.0 on 2026-06-03; historical
  design record only.
- docs/PERFORMANCE_OPTIMIZATIONS_PHASE1.md: Phase 1 work was completed 2026-05-30
  and released in v0.20.0 on 2026-06-01; the Phase 2 follow-up (parallel discovery)
  shipped in v1.4.0 on 2026-07-09; historical record only.
- The old note that "Performance Benchmarks always fail, red herring" is obsolete:
  benchmark failures are real signals since v0.21.0.
- The 2026-06-04 GCP and Fly.io infrastructure decommission does not affect svccat:
  it is a pure CLI and library crate with GitHub-Actions-only CI and no deployed
  runtime.
