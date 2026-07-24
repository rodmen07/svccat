# svccat roadmap

Last updated: 2026-07-24. This file is the single source of planning truth for svccat.
Older planning docs under `docs/` carry status banners pointing here and are kept as
historical records only.

The public 1.x API (library and CLI) is frozen under semver. The freeze is defined in
[docs/API_STABILITY.md](docs/API_STABILITY.md); this roadmap does not restate it. MSRV
is Rust 1.85.

## Current state (2026-07-24)

- Latest published release: **v1.5.0** (SPDX SBOM release; tag `v1.5.0` =
  merge commit `60c56b2`; published to crates.io 2026-07-18). Verified 2026-07-24
  via the crates.io API (`newest_version` = `1.5.0`) and `git tag -l v1.5.0`.
  v1.5.0 delivered SPDX 2.3 JSON SBOM export (`export --format spdx-json`),
  a `snapshot save --sbom` sidecar with delete cleanup, and a shared `timefmt`
  module. `Cargo.toml` on main reads `version = "1.5.0"`.
- **Unreleased on main, accumulating into the next minor** (each PR deliberately
  shipped without a version bump; the next release will publish them together):
  - CycloneDX 1.7 JSON SBOM export as a sibling of `spdx-json` (PR #11, `4202db6`).
  - `workspace check --format html` self-contained interactive multi-repo report
    (PR #6, `8f625fc`), hardened by PR #7 (DOM-XSS fix, `e97a67b`), PR #8
    (binary-level assert_cmd tests, `23cccff`), and PR #10 (CI builds this
    checkout, `8c6dc20`).
  - Policy rule schema validation in `svccat lint` (PR #12, `da3d537`).
  - SSRF redirect-hardening for `--ping`/webhooks (PR #14, `c925000`).
  - Base-chain-cycle crash fix in `svccat check` (PR #16, `32f2bca`; a HIGH
    stack-overflow on untrusted policy input, surfaced by the fuzzing rework).
  - Real fuzzing harness / `Continuous Fuzzing` workflow (PR #15, `f840161`).
  - `cargo audit --deny warnings` CI security gate plus live branch protection
    (PR #17, `1a7d9a3`).
- Shipped in earlier releases during 2026-06 and 2026-07 (the crate is NOT in
  frozen maintenance mode):
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
- Workflows: CI (now including a `Security audit` `cargo audit` gate as of PR #17),
  Code Coverage, Performance Benchmarks (meaningful signals since v0.21.0; a
  benchmark failure is a real regression, not noise), `Continuous Fuzzing`, and
  publish (runs on `v*` tag push). `main` is branch-protected with the required
  contexts set live 2026-07-20. **The fuzzing workflow is no longer a stub**
  (that 2026-07-18 caveat is resolved): PR #15 created `fuzz/Cargo.toml`, made
  the three targets (`fuzz_glob`, `fuzz_manifest`, `fuzz_url`) build, and
  replaced the placeholder run step with real time-boxed runs. What remains for
  fuzzing is a `fuzz_policy` target and committed seed corpora (see v1.6.0).

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

## Milestones

### v1.5.0: SPDX SBOM release — SHIPPED 2026-07-18

Done: `feat/spdx-sbom` merged as PR #3 (merge commit `60c56b2`), tag `v1.5.0`
pushed, the publish workflow pushed 1.5.0 to crates.io, and the GitHub release
is published. All three done-when conditions are met: the `v1.5.0` tag exists,
crates.io lists 1.5.0 (`newest_version` = `1.5.0`, verified 2026-07-24), and the
GitHub release is published. See History and supersession for detail.

### v1.5.1: docs and changelog hygiene

Cheap cleanup of drift found in the 2026-07-18 audit; one small PR. Now
UNBLOCKED (was gated on the v1.5.0 release, which shipped 2026-07-18).

- Reorder CHANGELOG.md entries into descending version order (1.2.0 currently sits
  between 1.4.0 and 1.3.0, and the 1.3.x entries are scrambled). No longer blocked:
  editing CHANGELOG.md is permitted now that v1.5.0 is published.
- Mark the SECURITY.md "v0.19.0 (Planned)" checklist as delivered with the actual
  ship date (v0.19.0 shipped 2026-05-28 per CHANGELOG.md); all five checklist items
  appear in the 0.19.0 changelog entry, but the boxes were never checked. (Confirmed
  still stale 2026-07-24: the five boxes under SECURITY.md "### v0.19.0 (Planned)"
  are all unchecked.)
- Already done 2026-07-18 (no further action): status banners added to
  docs/RELEASE_PLAN_V1.4.0.md, docs/FEATURE_DESIGN_MULTI_REPO.md, and
  docs/PERFORMANCE_OPTIMIZATIONS_PHASE1.md.

**Product note (decision for the user, filed 2026-07-24 during this audit):** the
CHANGELOG reorder above is a patch-level fix, but several *features* now sit
unreleased on main (see Current state), so the next actual release is a MINOR
(v1.6.0-class), not a patch. Recommended default: fold this v1.5.1 hygiene into
the next minor's release-prep rather than cutting a separate patch release. Kept
as its own milestone only so the two hygiene tasks are not lost; sequencing is
the user's call.

Done when: CHANGELOG.md is strictly descending, SECURITY.md has no stale unchecked
"Planned" boxes, and the fix is published (standalone or folded into the next minor).

### v1.6.0: make fuzzing real — PR 1 SHIPPED, PR 2 remaining

Audit finding (2026-07-18): the fuzzing setup was a stub. **PR 1 SHIPPED
2026-07-21 as PR #15 (`f840161`):** created `fuzz/Cargo.toml` (standard cargo-fuzz
layout, a separate crate so the root Cargo.toml is untouched), made the three
targets (`fuzz_glob`, `fuzz_manifest`, `fuzz_url`) build, renamed the workflow to
`Continuous Fuzzing`, and replaced the placeholder run step with real time-boxed
runs. docs/FUZZING.md was rewritten at the same time and now describes the real
setup. Widening `fuzz_manifest` from parse-only to parse-then-compile surfaced a
HIGH stack-overflow crash on cyclic policy `base` chains, fixed in PR #16
(`32f2bca`) — so the circular-base-rules security gate from
docs/RELEASE_PLAN_V1.4.0.md is closed at the code level, though a dedicated
fuzz_policy target is still wanted.

Remaining (PR 2):

- Add a `fuzz_policy` target covering `.svccat/policy.yaml` rule parsing,
  including circular base-rules (the crash above is fixed; a fuzz target guards
  against regressions and finds new cases). Confirmed 2026-07-24:
  `fuzz/fuzz_targets/` holds only `fuzz_glob.rs`, `fuzz_manifest.rs`, and
  `fuzz_url.rs` — no `fuzz_policy.rs`.
- Seed and commit small corpus directories for all targets from existing test
  fixtures (no committed corpus exists yet).
- Release per the flow in Working agreements.

Done when: the fuzz_policy target builds and runs green in the `Continuous
Fuzzing` workflow, and committed seed corpora exist for all targets.

### v1.6.1: coverage improvements

Pure test PRs; no API risk under the 1.x freeze.

- Pull the latest coverage report, identify the 2-3 lowest-covered `src/` modules,
  and add unit or regression tests for the worst one.
- Add edge-case tests for the new SBOM surface: empty catalog, services without
  `depends_on`, SPDXID sanitization collisions, sidecar delete when the snapshot is
  missing.
- Release per the flow in Working agreements (or fold into the next minor).

Done when: coverage on the targeted modules measurably improves and all tests pass.

### v1.7.0: dependency currency, part 1 (notify and criterion)

Direct dependencies have aging majors (notify 6, colored 2, ureq 2, criterion 0.5).
Bumps keep RUSTSEC exposure down. Split across two milestones so each stays at one
or two small PRs. UNBLOCKED (was gated on the v1.5.0 release, which shipped
2026-07-18; both parts touch Cargo.toml and Cargo.lock, which is now permitted).

- Run `cargo outdated` and `cargo audit`; record the bump list in the PR description.
- PR 1: bump notify to the current major with call-site migration and tests.
- PR 2: bump criterion (dev-only; cannot affect the frozen API).
- Confirm MSRV 1.85 still holds after each bump; document any required MSRV change
  per the docs/API_STABILITY.md policy.
- Release per the flow in Working agreements.

Done when: notify and criterion are on current majors, tests pass, and MSRV 1.85
is verified.

### v1.8.0: dependency currency, part 2 (ureq and colored)

Same rules as v1.7.0, and likewise UNBLOCKED (the v1.5.0 gate cleared 2026-07-18);
one runtime major per PR.

- PR 1: bump ureq to the current major with call-site migration and tests.
- PR 2: bump colored to the current major.
- Confirm MSRV 1.85 still holds after each bump; run `cargo audit`.
- Release per the flow in Working agreements.

Done when: no stale direct-dependency majors remain (or a skip decision is recorded),
`cargo audit` is clean, and MSRV 1.85 is verified.

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
Cargo.toml, Cargo.lock, the v1.5.1 CHANGELOG reorder, and the v1.7.0/v1.8.0
dependency bumps) are all lifted. What remains:

| Item | Status | Reason |
|------|--------|--------|
| Writing the `CRATES_IO_TOKEN` repo secret | USER-ONLY | Secret values are never handled by an agent; set via `gh secret set` (interactive) |
| Tag push, GitHub release, `cargo publish` | Delegated | Follow the release flow in Working agreements; merge and tag only on green CI |
| Editing README.md | Avoid | CRLF line endings; no roadmap work needs it |

## History and supersession

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
  the published crate, commit `8c6dc20`). The fuzz_manifest circular-base-rules
  expansion shipped 2026-07-21 (PR #15, `f840161`), and the HIGH stack-overflow
  crash it surfaced on cyclic policy `base` chains was fixed in PR #16
  (`32f2bca`); a dedicated fuzz_policy target is still carried in v1.6.0 PR 2.
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
  design record only.
- docs/PERFORMANCE_OPTIMIZATIONS_PHASE1.md: Phase 1 work was completed 2026-05-30
  and released in v0.20.0 on 2026-06-01; the Phase 2 follow-up (parallel discovery)
  shipped in v1.4.0 on 2026-07-09; historical record only.
- The old note that "Performance Benchmarks always fail, red herring" is obsolete:
  benchmark failures are real signals since v0.21.0.
- The 2026-06-04 GCP and Fly.io infrastructure decommission does not affect svccat:
  it is a pure CLI and library crate with GitHub-Actions-only CI and no deployed
  runtime.
