# Changelog

All notable changes to svccat are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
