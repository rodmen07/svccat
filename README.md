# svccat

[![CI](https://github.com/rodmen07/svccat/actions/workflows/ci.yml/badge.svg)](https://github.com/rodmen07/svccat/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/svccat.svg)](https://crates.io/crates/svccat)
[![Downloads](https://img.shields.io/crates/d/svccat.svg)](https://crates.io/crates/svccat)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Service catalog drift detection for multi-service repositories.**

svccat reads your declared service manifest and compares it against what actually
exists in the repo, flagging missing services, undeclared additions, and stale
metadata before they become operational toil. It runs in your terminal and in CI,
so your declared architecture stays honest.

---

## Install

```bash
cargo install svccat
```

This puts a `svccat` binary on your PATH. (Or clone the repo and run
`cargo build --release`; the binary lands at `target/release/svccat`.)

---

## Getting started

After installing, see svccat work immediately with the built-in demo, which runs a
narrated walkthrough against a throwaway sample repo (no setup, nothing touched in
your own files):

```bash
svccat demo
```

Then point it at your own repo:

```bash
cd my-monorepo
svccat init            # scaffold services.yaml from the services already in the repo
svccat check           # report drift between the manifest and the repo
```

And gate CI on a clean catalog:

```bash
svccat check --fail-on-drift   # exits 1 when drift is found
```

Inside GitHub Actions, `svccat check` automatically emits inline PR annotations.

---

## Quick start

Commands are grouped by task. Run `svccat <command> --help` for the full flags of any
command.

**Set up a manifest**

- `svccat init` scaffolds a `services.yaml` from your repo (`--force` to overwrite),
  inferring each service's `language` and `platform` from files it finds on disk
- `svccat import --from backstage|docker-compose|openapi` seeds it from another source
- `svccat lint` validates the manifest for structural issues
- `svccat install-hooks` installs a pre-commit hook (`--hook pre-push` for pre-push)
- `svccat completions bash` prints a shell completion script (also `zsh`, `fish`)

**Check for drift**

- `svccat check` inspects drift in the current repo
- `svccat check --fail-on-drift` exits 1 on any drift (CI gate)
- `svccat check --depth 2` also discovers services nested one level deeper
- `svccat check --team platform` limits the check to one team's services
- `svccat check --since HEAD~1 --fail-on-new-drift` exits 1 only on new drift
- `svccat check --ping` verifies that each service `url` is reachable

Add `--format <fmt>` to change the output, where `<fmt>` is one of `compact`, `csv`,
`json`, `sarif`, `markdown`, `junit`, `github-annotation`, `slack`, `teams`, or `datadog`.

**Fix drift**

- `svccat fix` adds undeclared services to the manifest, inferring `language` and
  `platform` for each (`--prune` removes missing ones, `--dry-run` previews)

**Visualize and report**

- `svccat graph` renders a Mermaid diagram (`--format dot|plantuml|markdown`, `--team`, `--filter`)
- `svccat report` writes a full ownership and drift report (`--format html`, `--history N`, `--badge`)
- `svccat scorecard` scores each service on completeness, drift, and policy
- `svccat stats` prints a field-coverage summary with ASCII bar charts

**Snapshots and diffs**

- `svccat export --format json > snap.json` saves a catalog snapshot (`csv` also works)
- `svccat diff before.json after.json` compares two snapshots
- `svccat snapshot save|list|delete|diff <name>` manages named drift snapshots

**Watch and serve**

- `svccat watch` re-runs drift detection on every file change (`--notify`, `--interval 30`)
- `svccat serve` serves a live HTML report at http://localhost:7777 (`--port`, `--refresh`)

**Audit, policy, and CI**

- `svccat audit` runs lint + drift + score in one pass (add `--ping`)
- `svccat policy` checks required/recommended fields (add `--fail-on-violations`)
- `svccat ci` runs lint + drift + policy in one CI-friendly pass

**Query the catalog**

- `svccat search auth` searches by substring; `svccat search team:platform` by field
- `svccat deps` analyzes inter-service dependencies (`--format mermaid|json`)
- `svccat tag add|remove <service> <tag>` edits tags

**Multiple repositories**

- `svccat workspace check` checks drift across every repo in a `[workspace]` section of
  `svccat.toml`, with aggregated reporting and cross-repo dependency analysis

Manifest is auto-detected: svccat tries `svccat.yaml`, `svccat.yml`, `services.yaml`,
`services.yml` in order. Most commands accept `--output <file>` and `--format json`.

---

## Manifest format

```yaml
# svccat.yaml  (or services.yaml)
version: "1"

discovery:                # optional: where to look for services
  paths: ["services/*", "microservices/*"]
  markers: [Cargo.toml, go.mod, package.json, pyproject.toml, Dockerfile]
  ignore: ["services/vendor/*"]

policy:                   # optional: fields every service must declare (error if missing)
  require_fields: [url, language, platform]

services:
  - name: api-gateway              # required
    language: Go
    platform: Cloud Run
    role: Rate-limiting reverse proxy   # required (error if missing)
    url: https://gateway.example.com    # enables --ping health checks
    team: platform                 # optional ownership
    oncall: platform@example.com
    docs: docs/api-gateway.md      # optional: warn if the file is missing
    depends_on: [auth-service]     # rendered as graph edges; validated for cycles
```

Declared services are matched to the repo by `path:`, then `submodule:`, then by name
against discovered directories. When `discovery.paths` is empty, svccat tries
`services/*`, `microservices/*`, `apps/*`, and `packages/*`.

---

## Drift types

| Kind | Severity | Description |
|------|----------|-------------|
| `declared_missing_from_repo` | **error** | Declared in the manifest but its directory is missing. |
| `undeclared_in_repo` | warning | A service directory exists but is not in the manifest. |
| `missing_field` | error / warning | A recommended field is absent (`role` = error; `language`, `platform` = warning). |
| `missing_referenced_file` | warning | A `docs:` or `ci:` path is declared but the file is missing. |
| `policy_violation` | **error** | A field required by `policy.require_fields` is absent. |
| `dangling_dependency` | **error** | A `depends_on` entry references an undeclared service. |
| `circular_dependency` | **error** | A cycle was detected in the `depends_on` graph. |

Errors and warnings both count toward `--fail-on-drift`.

---

## Use it as a library

svccat ships as both a binary and a library, so you can call into the same modules the
CLI uses:

```rust
use std::path::Path;
use svccat::{discovery, drift, manifest::Manifest};

let manifest = Manifest::load(Path::new("services.yaml"))?;
let discovered = discovery::discover_services(Path::new("."), &manifest);
let report = drift::analyze(&manifest, &discovered, Path::new("."));
println!("{} errors, {} warnings", report.error_count(), report.warning_count());
```

See [`examples/demo.rs`](examples/demo.rs) (`cargo run --example demo`) and the API docs
on [docs.rs/svccat](https://docs.rs/svccat).

---

## Learn more

- `svccat <command> --help` documents every command and flag.
- [CHANGELOG.md](CHANGELOG.md) for release history.
- [SECURITY.md](SECURITY.md) for the threat model and supported versions.

---

## Support

svccat is free and open source. If it saves you some toil, you can buy me a coffee:

[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-support-yellow?logo=buymeacoffee&logoColor=white)](https://www.buymeacoffee.com/rodmen07)

---

## Contributing

Bug reports and pull requests welcome. Please run `cargo clippy -- -D warnings` and
`cargo fmt` before opening a PR.

## License

MIT. See [LICENSE](LICENSE).