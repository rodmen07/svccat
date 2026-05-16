# svccat

[![CI](https://github.com/rodmen07/svccat/actions/workflows/ci.yml/badge.svg)](https://github.com/rodmen07/svccat/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/svccat.svg)](https://crates.io/crates/svccat)
[![Downloads](https://img.shields.io/crates/d/svccat.svg)](https://crates.io/crates/svccat)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Service catalog drift detection for multi-service repositories.**

svccat reads your declared service manifest and compares it against what
actually exists in the repo — flagging missing services, undeclared additions,
and stale metadata before they become operational toil.

---

## Why svccat?

In any multi-service repo the architecture docs, service inventory, and the
codebase evolve at different speeds. A new service appears in `services/` but
never makes it into the manifest. A deprecated service stays in the YAML long
after the directory is gone. `docs:` and `ci:` references silently rot.

svccat makes drift visible in your terminal and in CI, so your declared
architecture stays honest.

---

## Installation

```bash
cargo install svccat
```

Or build from source:

```bash
git clone https://github.com/rodmen07/svccat
cd svccat
cargo build --release
# binary at target/release/svccat
```

---

## Quick start

```
svccat init                              # scaffold services.yaml from your repo
svccat import --from backstage           # seed services.yaml from catalog-info.yaml files
svccat check                             # inspect drift in the current repo
svccat check --fail-on-drift             # gate CI on zero drift (exit 1 on drift)
svccat check --format compact            # one-line-per-service summary (great for large repos)
svccat check --format csv                # CSV output: service, severity, kind, message, detail
svccat check --depth 2                   # also discover services nested one level deeper
svccat check --team platform             # only check services owned by "platform"
svccat check --ignore "examples/*"       # skip directories matching the pattern
svccat check --format json               # machine-readable output
svccat check --format sarif              # SARIF 2.1.0 for GitHub Code Scanning
svccat check --format markdown           # Markdown table for PR comments
svccat check --format junit              # JUnit XML for CI test reporting
svccat check --format github-annotation  # GitHub Actions annotations for CI
svccat check --since HEAD~1              # show only drift new since the previous commit
svccat check --since HEAD~1 --fail-on-new-drift  # exit 1 only on new drift (ignores pre-existing)
svccat check --baseline baseline.json    # ignore pre-existing drift in the saved baseline
svccat fix                               # add undeclared services to the manifest
svccat fix --prune                       # also remove declared services with missing directories
svccat fix --dry-run                     # preview what would change without writing
svccat import --from backstage           # seed services.yaml from catalog-info.yaml files
svccat import --from docker-compose      # seed services.yaml from docker-compose.yml
svccat install-hooks                     # install a pre-commit hook (runs svccat check)
svccat install-hooks --hook pre-push     # install a pre-push hook instead
svccat graph                             # Mermaid diagram grouped by platform
svccat graph --team platform             # diagram scoped to one team
svccat graph --format markdown           # Markdown table
svccat report                            # full ownership + drift report (Markdown)
svccat report --format html --output report.html  # self-contained HTML report
svccat report --history 5                # drift evolution across last 5 commits
svccat report --badge                    # Shields.io badge (for README)
svccat lint                              # validate manifest for structural issues
svccat export --format json > snap.json  # save a catalog snapshot
svccat export --format csv               # CSV catalog: name, language, platform, role, url, team
svccat diff before.json after.json       # compare two snapshots
svccat watch                             # continuous drift detection (re-runs on changes)
svccat completions bash                  # print bash completion script
```

Manifest is auto-detected: svccat tries `svccat.yaml`, `svccat.yml`,
`services.yaml`, `services.yml` in order.

> **GitHub Actions tip:** When running inside a GitHub Actions workflow, `svccat check`
> automatically switches to `github-annotation` output (no `--format` flag needed) so drift
> items appear as inline annotations on pull requests. Set `upload-sarif: true` in the Action
> step to also send results to GitHub Code Scanning.

---

## `svccat init`

Bootstrap a `services.yaml` in seconds by letting svccat discover what's
already in your repo:

```bash
svccat init            # writes services.yaml in the current directory
svccat init --force    # overwrite an existing file
svccat init --output path/to/svccat.yaml   # custom output path
```

The generated file includes every detected service with language inferred from
marker files (`Cargo.toml` → Rust, `go.mod` → Go, `package.json` → TypeScript,
`pyproject.toml` / `requirements.txt` → Python), plus `~` placeholders for
`platform`, `role`, and `url` that you fill in before committing.

Example output:

```yaml
# Generated by `svccat init`
# Fill in the ~placeholder~ fields and commit this file.
# Run `svccat check` to verify there is no drift.

version: "1"

discovery:
  paths:
    - services/*
    - microservices/*
    - apps/*
    - packages/*

services:
  - name: api-gateway
    path: services/api-gateway
    language: Go
    platform: ~  # e.g. gcp-cloud-run, fly.io, vercel, aws-lambda
    role: ~      # e.g. api, worker, frontend, database
    url: ~       # e.g. https://my-service.example.com
  - name: auth-service
    path: services/auth-service
    language: Rust
    platform: ~
    role: ~
    url: ~
```

---

## Manifest format

```yaml
# svccat.yaml  (or services.yaml for backwards compat)
version: "1"

# Optional: configure how svccat discovers services in the repo.
discovery:
  paths:                  # glob patterns for candidate service directories
    - "services/*"
    - "microservices/*"
  markers:                # files that identify a directory as a service
    - Cargo.toml
    - Dockerfile
    - go.mod
    - package.json
    - pyproject.toml
    - requirements.txt
  ignore:                 # paths to exclude from discovery
    - "services/examples"
    - "services/vendor/*"

policy:
  require_fields:         # every service must declare these fields (error if missing)
    - url
    - language
    - platform

services:
  - name: api-gateway               # required
    language: Go                    # recommended
    platform: Cloud Run             # recommended
    role: Rate-limiting reverse proxy  # required (error if missing)
    url: https://gateway.example.com   # optional: enables --ping health checks
    team: platform                  # optional: owning team name
    oncall: platform@example.com    # optional: on-call contact (email, handle, PD service)
    path: infra/gateway             # optional: explicit path (overrides name matching)
    submodule: go-gateway           # optional: git submodule path (Portfolio-compatible)
    docs: docs/api-gateway.md       # optional: warn if file missing
    ci: .github/workflows/api-gateway.yml  # optional: warn if file missing
    depends_on:                     # optional: rendered as edges in svccat graph
      - auth-service
      - postgres
```

### Default discovery paths

When `discovery.paths` is empty svccat tries `services/*`, `microservices/*`,
`apps/*`, and `packages/*`.

### Matching declared ↔ discovered

1. If the entry has `path:` → check that path exists.
2. Else if the entry has `submodule:` → check that path exists.
3. Else → match by name against discovered service directory names.

---

## Drift types

| Kind | Severity | Description |
|------|----------|-------------|
| `declared_missing_from_repo` | **error** | Service is in the manifest but its directory is not found in the repo. |
| `undeclared_in_repo` | warning | A service directory was discovered but is not listed in the manifest. |
| `missing_field` | error / warning | A recommended metadata field is absent (`role` = error; `language`, `platform` = warning). |
| `missing_referenced_file` | warning | A `docs:` or `ci:` path is declared but the file does not exist. |
| `policy_violation` | **error** | A field required by `policy.require_fields` is absent. |
| `dangling_dependency` | **error** | A `depends_on` entry references a service not declared in the manifest. |
| `circular_dependency` | **error** | A cycle was detected in the `depends_on` graph. |

---

## `depends_on` validation and cycle detection

svccat validates all `depends_on` references and checks for cycles in the
dependency graph:

```yaml
services:
  - name: api-gateway
    depends_on: [auth-service, ghost-service]   # ghost-service → DanglingDependency error
  - name: auth-service
    depends_on: [api-gateway]                   # api-gateway → api-gateway = CircularDependency error
```

```
x  [DEPENDS]    'api-gateway' depends_on 'ghost-service' which is not declared in the manifest
x  [CYCLE]      circular dependency detected: api-gateway → auth-service → api-gateway
```

Both kinds are error-severity and count toward `--fail-on-drift`.

---

## SARIF output — GitHub Code Scanning integration

Use `--format sarif` to emit [SARIF 2.1.0](https://sarifweb.azurewebsites.net/)
output.  Upload it to GitHub Code Scanning so drift items appear as inline
annotations on pull requests — no extra tooling required.

```bash
svccat check --format sarif > results.sarif
```

### GitHub Actions integration with Code Scanning

```yaml
# .github/workflows/catalog.yml
- name: Run svccat
  run: svccat check --format sarif > results.sarif

- name: Upload SARIF to GitHub Code Scanning
  uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: results.sarif
```

Each drift item becomes a code-scanning alert with:
- **Rule ID** matching the drift kind (e.g. `declared_missing_from_repo`)
- **Severity** (`error` or `warning`)
- **Location** pointing to the manifest file

---

## `svccat graph --team` — team-scoped diagrams

Filter the Mermaid diagram to a single team's services. Cross-team
`depends_on` targets are shown as external placeholder nodes so
dependencies are still visible.

```bash
svccat graph --team platform
```

```mermaid
graph TD
  subgraph Cloud_Run["Cloud Run"]
    api_gateway["api-gateway\nGo\nAPI gateway"]
    auth_service["auth-service\nRust\nJWT issuance"]
  end
  subgraph External["External (other teams)"]
    postgres["postgres\n[ext]"]
  end
  api_gateway --> auth_service
  api_gateway --> postgres
```

---

## `svccat diff` — compare snapshots

Track how your catalog evolves over time by diffing two JSON snapshots:

```bash
# Save a snapshot before making changes
svccat export --format json > before.json

# ... update services.yaml or add/remove services ...

# Save a snapshot after
svccat export --format json > after.json

# Show what changed
svccat diff before.json after.json
```

Example output:

```
svccat diff: before.json → after.json

  Services added (1):
    +  new-worker

  Services removed (1):
    -  legacy-api

  Services changed (1):
    ~  auth-service
       language: Python → Rust

  Resolved drift (1):
    ✓  [ERROR] legacy-api — 'legacy-api' is declared in the manifest but not found in the repo
```

---

## Policy rules

Enforce field requirements across all services via the `policy:` section in
your manifest:

```yaml
policy:
  require_fields:
    - url        # every service must have a health-check URL
    - language   # documentation requirement
    - platform   # deployment target must be explicit
```

Any service missing a required field is flagged as an error-level drift item:

```
x  [POLICY]  'worker-service' violates policy: required field 'url' is missing
```

Policy violations count toward `--fail-on-drift` and `fail_on_drift` in
`svccat.toml`.

---

## Ownership metadata — `team` and `oncall`

Declare service ownership directly in your manifest so every entry has a clear
owner:

```yaml
services:
  - name: api-gateway
    team: platform        # owning team name
    oncall: platform@example.com   # on-call contact (email, handle, or PD service)
  - name: billing-service
    team: growth
    oncall: growth-pagerduty
```

### Team-scoped checks

Pass `--team <name>` to limit drift detection to a single team's services.
Services belonging to other teams are excluded from analysis (no false
`UndeclaredInRepo` noise).

```bash
# CI step for the platform team — only checks platform-owned services.
svccat check --team platform --fail-on-drift
```

### Enforce ownership via policy

Require `team` and `oncall` on every service using `policy.require_fields`:

```yaml
policy:
  require_fields:
    - team
    - oncall
```

Any service missing either field becomes an error-level policy violation.

---

## `svccat watch` — continuous drift detection

`svccat watch` monitors the manifest file and service directories for changes
and re-runs drift analysis automatically. Useful while actively editing a
manifest or onboarding services.

```bash
svccat watch                          # watch and re-check on every file change
svccat watch --team platform          # only watch platform-owned services
svccat watch --fail-on-drift          # exit 1 if initial check finds drift
svccat watch --ignore "examples/*"    # exclude patterns (same as check)
```

Example output when a service directory is added or the manifest changes:

```
svccat: 3 declared, 3 discovered  [services.yaml]

  OK  No drift detected

● Watching services.yaml and service directories. Press Ctrl-C to stop.

[14:32:07 UTC] change detected — re-running drift check
svccat: 3 declared, 4 discovered  [services.yaml]

  DRIFT DETECTED  (0 errors, 1 warning)

  !  [UNDECLARED]  'services/new-worker' exists in the repo but is not listed in the manifest

  !  1 warning(s)
```

Press **Ctrl-C** to stop watching.

---

## `svccat report` — ownership report

Generate a full Markdown (or HTML) ownership report: per-team service tables,
drift status per service, dependency summary, and full drift details.

```bash
svccat report                          # Markdown to stdout
svccat report --format html            # self-contained HTML page to stdout
svccat report --output report.md       # write to file
svccat report --format html --output catalog.html
```

Example Markdown output:

```markdown
# Service Catalog Report

## Summary

| Metric | Value |
|--------|-------|
| Services | 5 |
| Teams | 2 |
| Drift errors | 0 |
| Drift warnings | 1 |

## Services by Team

### platform (3 services)

| Service | Language | Platform | Role | Oncall | Drift |
|---------|----------|----------|------|--------|-------|
| api-gateway | Go | Cloud Run | API gateway | platform@example.com | ✅ |
| auth-service | Rust | Cloud Run | JWT issuance | platform@example.com | ✅ |
| event-stream | Rust | Fly.io | Event bus | platform@example.com | ⚠️ 1 warning(s) |
```

The HTML format produces a self-contained styled page suitable for sharing with
non-technical stakeholders.

### `svccat report --history <N>` — drift evolution over time

Run the report across the last N git commits and emit a Markdown table showing how
drift has evolved. Uses the current discovered services against each historical manifest.

```bash
svccat report --history 5                        # last 5 commits
svccat report --history 10 --output history.md   # write to file
```

Example output:

```markdown
## Drift History (last 5 commits)

| Commit | Summary | Errors | Warnings | Total |
|--------|---------|--------|----------|-------|
| `a1b2c3` | feat: add auth-service | 0 | 1 | ⚠️ 1 |
| `d4e5f6` | fix: update API gateway | 1 | 1 | ❌ 2 |
| `e7f8a9` | chore: update deps | 1 | 2 | ❌ 3 |
```

Useful for sprint reviews, architecture health dashboards, and understanding how
long-running drift was introduced.

---

## `svccat lint` — manifest validation

Check the manifest for structural problems that drift analysis won't catch:

```bash
svccat lint                  # lint services.yaml in the current directory
svccat lint -m svccat.yaml   # explicit manifest path
```

Checks performed:

| Check | Severity |
|-------|----------|
| Duplicate service names | error |
| Blank or whitespace-only service name | error |
| Service depends_on itself | error |
| Blank entry in depends_on list | error |
| Duplicate entries in depends_on | warning |
| Unrecognised manifest version | warning |

Example output:

```
✗ [error] duplicate service name 'api' appears 2 times
✗ [error] 'worker' lists itself in depends_on
⚠ [warn]  'api' lists 'auth' more than once in depends_on

found 2 error(s), 1 warning(s)
```

Exit codes: `0` if no errors (warnings are allowed), `1` if any errors are found.

---

## `svccat check --since` — PR-friendly drift diffs

Compare current drift against the manifest as it was at a given git ref.
Outputs only what changed — new drift items or issues that were resolved.

```bash
svccat check --since HEAD~1       # compare against the previous commit
svccat check --since main         # compare against the main branch
svccat check --since v0.7.0       # compare against a tag
```

Example output:

```
svccat --since HEAD~1  [services.yaml]

  NEW drift since HEAD~1 (1):

  x  [MISSING]    'legacy-worker' is declared in the manifest but not found in the repo

  RESOLVED since HEAD~1 (1):

  ✓  [UNDECLARED]  'services/old-api' exists in the repo but is not listed in the manifest

  3 existing drift items unchanged
```

Ideal for a CI step on pull requests: only fails when the PR introduces _new_ drift,
not when existing drift is already tracked.

### `--fail-on-new-drift` — gate CI on new drift only

Add `--fail-on-new-drift` to exit 1 only when `--since` reveals _new_ drift items.
Pre-existing drift that was already present at the base ref is ignored.

```bash
# In CI: fail the PR only if it introduces new drift
svccat check --since origin/main --fail-on-new-drift --fail-on-drift

# Can combine with --format markdown to post as a PR comment
svccat check --since origin/main --format markdown --fail-on-new-drift
```

### `--format markdown` — Markdown output for PR comments

Emit the drift report as a Markdown table suitable for posting as a GitHub PR comment.

```bash
svccat check --format markdown
svccat check --since HEAD~1 --format markdown --fail-on-new-drift
```

Example output:

```markdown
## 🔍 svccat drift check

**3 declared · 3 discovered** — `services.yaml`

❌ **DRIFT DETECTED** (1 error, 1 warning)

| Severity | Kind | Service | Message |
|----------|------|---------|---------|
| ❌ Error | MISSING | `legacy-worker` | declared in manifest but directory not found |
| ⚠️ Warning | FIELD | `api-gateway` | missing recommended field: oncall |
```

### `--format github-annotation` — GitHub Actions annotations (v0.10.0)

Emit drift as [GitHub Actions annotations](https://docs.github.com/en/actions/using-workflows/workflow-commands-for-github-actions#setting-an-error-message) for inline PR feedback:

```bash
svccat check --format github-annotation
svccat check --since HEAD~1 --format github-annotation
```

This outputs:
```
::error file=services.yaml,title=svccat [MISSING]::legacy-worker: declared in manifest but directory not found
::warning file=services.yaml,title=svccat [FIELD]::api-gateway: missing recommended field: oncall
```

Annotations appear in PR checks and in the Annotations tab of the workflow run. See the included [`.github/workflows/svccat-pr.yml`](.github/workflows/svccat-pr.yml) for a production-ready workflow template.

### `--format junit` — JUnit XML output (v0.11.0)

Emit drift as JUnit XML for CI systems that ingest test reports:

```bash
svccat check --format junit > svccat-junit.xml
svccat check --since origin/main --format junit --fail-on-new-drift > svccat-junit.xml
```

This is useful for GitHub Actions, GitLab CI, Jenkins, CircleCI, and other systems that can display JUnit test results.

### `svccat report --badge` — Shields.io badge (v0.10.0)

Emit a Markdown badge snippet for embedding in your README:

```bash
svccat report --badge
```

Output (example — green for clean, red/yellow for drift):

```markdown
[![svccat drift: clean](https://img.shields.io/badge/svccat-drift%20clean-brightgreen)](https://crates.io/crates/svccat)
```

Embed in your README to show catalog health at a glance.

---

## `svccat.toml` — workspace defaults

Place a `svccat.toml` in your repo root to set persistent defaults so you
don't need to pass flags on every invocation. CLI flags always take precedence.

```toml
# svccat.toml
format = "terminal"         # default output format: "terminal" or "json"
fail_on_drift = true        # always exit 1 on drift (no need for --fail-on-drift)
ignore = [
  "services/examples",
  "services/vendor/*",
  "test-fixtures/*",
]
```

### `--ignore` patterns

Exclude directories from drift detection on the fly:

```bash
svccat check --ignore "services/examples" --ignore "vendor/*"
```

Ignore patterns in `svccat.toml` and in `discovery.ignore` (manifest) are
merged with patterns from the CLI flag.

---

## Shell completions

Generate tab-completion scripts for your shell:

```bash
# Bash (add to ~/.bashrc)
source <(svccat completions bash)

# Zsh (add to your $fpath)
svccat completions zsh > ~/.zfunc/_svccat

# Fish
svccat completions fish > ~/.config/fish/completions/svccat.fish
```

---

## CI integration

### `svccat check --ping`

Add `url:` to any service entry and pass `--ping` to verify each endpoint is
reachable at run time:

```bash
svccat check --ping              # terminal output with HTTP status per service
svccat check --ping --format json  # machine-readable ping results
```

Example output:

```
svccat: 3 declared, 3 discovered  [services.yaml]

  OK  No drift detected

  Ping results:
    ✔  api-gateway     https://gateway.example.com  200 OK
    ✔  auth-service    https://auth.example.com     200 OK
    ✗  legacy-worker   https://worker.example.com   unreachable (connection refused)
```

### `depends_on` graph edges

Declare explicit service dependencies and they are rendered as directed edges
in `svccat graph`:

```yaml
services:
  - name: api-gateway
    depends_on:
      - auth-service
      - postgres
```

```
svccat graph
```

```mermaid
graph TD
  subgraph Cloud_Run["Cloud Run"]
    api_gateway["api-gateway\nGo\nreverse proxy"]
    auth_service["auth-service\nRust\nJWT issuance"]
  end
  subgraph Cloud_SQL["Cloud SQL"]
    postgres["postgres\nSQL\ndatabase"]
  end
  api_gateway --> auth_service
  api_gateway --> postgres
```

### GitHub Action

Use svccat in GitHub Actions without installing Rust first:

```yaml
# .github/workflows/catalog.yml
name: Catalog check
on: [push, pull_request]

jobs:
  catalog:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rodmen07/svccat@v1
        with:
          fail-on-drift: 'true'   # default — exits 1 on drift
```

**Inputs:**

| Input | Default | Description |
|-------|---------|-------------|
| `root` | `.` | Path to repo root (where `services.yaml` lives) |
| `fail-on-drift` | `true` | Exit 1 when drift is detected |
| `version` | `latest` | svccat crates.io version to install |

The action caches the installed binary so subsequent runs skip the `cargo install` step.

### Drift annotations + PR comments (v0.10.0)

For advanced workflows that emit GitHub Actions annotations and post drift summaries as PR comments, use the included workflow template:

```bash
cp .github/workflows/svccat-pr.yml .github/workflows/svccat.yml
```

The template runs `svccat check` with `--format github-annotation`, creates an annotated drift report, posts it as a PR comment (with automatic update on subsequent pushes), and gates the PR check on new drift.

### Manual CI integration

Add a step to your pipeline to gate merges on zero drift:

```yaml
# .github/workflows/catalog.yml
name: Catalog check
on: [push, pull_request]

jobs:
  catalog:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with: { toolchain: stable }
      - run: cargo install svccat
      - run: svccat check --fail-on-drift
```

Exit codes:
- `0` — no drift (or drift present but `--fail-on-drift` not set)
- `1` — drift detected and `--fail-on-drift` is set
- `2` — fatal error (unreadable manifest, parse failure, etc.)

---

## Example output

### Terminal

```
svccat: 3 declared, 3 discovered  [services.yaml]

  OK  No drift detected
```

```
svccat: 4 declared, 3 discovered  [services.yaml]

  DRIFT DETECTED  (1 error, 2 warnings)

  x  [MISSING]     'legacy-worker' is declared in the manifest but not found in the repo
  !  [UNDECLARED]  'services/experimental-api' exists in the repo but is not listed in the manifest
  !  [FIELD]       'event-stream' is missing recommended field: platform

  x  1 error(s)
  !  2 warning(s)
```

### Mermaid graph (`svccat graph`)

````markdown
```mermaid
graph TD
  subgraph Cloud_Run["Cloud Run"]
    api_gateway["api-gateway\nGo\nRate-limiting reverse proxy"]
    auth_service["auth-service\nPython / FastAPI\nJWT issuance and OAuth"]
  end
  subgraph GitHub_Pages["GitHub Pages"]
    frontend["frontend\nTypeScript / React\nSingle-page application"]
  end
```
````

---

## Try the sample monorepo

```bash
cd examples/sample-monorepo
svccat check
svccat graph
svccat export --format json
```

---

## Project status

`v0.11.0` — `svccat check --format junit` (JUnit XML output for CI test report ingestion and `--since` support for new-drift-only test failures).

Previous releases:
- `v0.10.0` — `svccat report --badge` (Shields.io drift-status badge for your README), `svccat check --format github-annotation` (native GitHub Actions workflow annotations), included workflow template (`.github/workflows/svccat-pr.yml`)
- `v0.9` — `svccat check --format markdown` (PR-comment-ready Markdown output), `svccat check --since --fail-on-new-drift` (CI gate on new drift only), `svccat report --history <N>` (drift evolution over last N commits)
- `v0.8` — `svccat report` ownership report (Markdown + HTML), `svccat lint` manifest validation, `svccat check --since` PR-friendly drift diffs
- `v0.6` — `svccat watch` continuous drift detection, `team`/`oncall` ownership metadata, `--team` team-scoped checks
- `v0.5` — `svccat diff` snapshot comparison, `policy.require_fields` enforcement
- `v0.4` — `svccat.toml` workspace config, `--ignore` discovery patterns, shell tab completions
- `v0.3` — GitHub Action (`rodmen07/svccat@v1`), `depends_on` dependency graph edges, `svccat check --ping` health checks
- `v0.2` — `svccat init` command (scaffold `services.yaml` from your repo)
- `v0.1` — core drift detection, terminal/JSON/Mermaid/Markdown output, CI integration

---

## Contributing

Bug reports and pull requests welcome.  
Please run `cargo clippy -- -D warnings` and `cargo fmt` before opening a PR.

## License

MIT — see [LICENSE](LICENSE).
