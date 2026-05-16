# Changelog

All notable changes to svccat are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

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
