# API stability policy

This document defines what is, and is not, covered by [semantic
versioning](https://semver.org/) for `svccat` once it reaches `1.0.0`.

svccat ships two surfaces: a **command-line tool** (the `svccat` binary) and a
**Rust library** (the `svccat` crate). Both are versioned together.

## Library API

### Covered by semver

Only the modules documented on [docs.rs](https://docs.rs/svccat) are part of the
stable public API. As of 1.0 that is:

| Module | Stable surface |
|--------|----------------|
| [`manifest`] | `Manifest`, `ServiceEntry`, `DiscoveryConfig`, `PolicyConfig`, and their public methods |
| [`discovery`] | `DiscoveredService`, `discover_services`, and related discovery functions |
| [`drift`] | `DriftReport`, `DriftItem`, `DriftKind`, `Severity`, `analyze` |
| [`report`] | the `render_*` functions |
| [`config`] | `SvccatConfig` and workspace config loading |

Within a `1.x` release line:

- Existing public items keep their signatures and behaviour.
- New items may be added.

### Not covered by semver

- **Every other module.** All modules outside the table above are marked
  `#[doc(hidden)]`. They are `pub` only so the `svccat` binary can use them, and
  they may change or disappear in any release, including patch releases. Do not
  depend on them.
- **`#[non_exhaustive]` types.** The core public types are marked
  `#[non_exhaustive]`, so new fields and new enum variants may be added in a
  minor release without a major bump. Construct them with `Default` plus field
  assignment (not struct literals), and always include a wildcard arm when
  matching `DriftKind` / `Severity`.
- **Exact rendered text.** The precise wording, ordering, and whitespace of
  human-readable output (terminal, Markdown, HTML) is not part of the contract.
  Machine-readable formats (JSON, SARIF) keep their documented field names.

## CLI

### Covered by semver

- **Subcommand names** and their **flag names / short flags.**
- **The set of accepted `--format` values** for each command (new values may be
  added; existing ones are not removed or renamed within `1.x`).
- **Exit codes:** `0` success, `1` drift / policy failure when a `--fail-on-*`
  flag is set, `2` usage or runtime error.

### Not covered by semver

- Exact human-readable output text (see above).
- Help text and the order in which items are printed.

## Minimum Supported Rust Version (MSRV)

The MSRV is declared via `rust-version` in `Cargo.toml` (currently `1.85`, driven
by the `clap` dependency floor). Raising the MSRV is treated as a **minor**
change, not a breaking one: a `1.x` release may require a newer Rust toolchain.

## Reporting

If a release breaks something covered above without a major version bump, please
open an issue at <https://github.com/rodmen07/svccat/issues>.

[`manifest`]: https://docs.rs/svccat/latest/svccat/manifest/
[`discovery`]: https://docs.rs/svccat/latest/svccat/discovery/
[`drift`]: https://docs.rs/svccat/latest/svccat/drift/
[`report`]: https://docs.rs/svccat/latest/svccat/report/
[`config`]: https://docs.rs/svccat/latest/svccat/config/
