//! # svccat
//!
//! Detect drift between your declared service catalog and what actually lives in
//! the repository.
//!
//! `svccat` is primarily a command-line tool (the `svccat` binary), but its core
//! engine is usable as a library. The typical flow is:
//!
//! 1. Load the declared catalog with [`manifest::Manifest::load`].
//! 2. Discover the services that actually exist on disk with
//!    [`discovery::discover_services`].
//! 3. Compute drift between the two with [`drift::analyze`], which returns a
//!    [`drift::DriftReport`].
//! 4. Optionally render the report with the [`report`] module.
//!
//! ```no_run
//! use std::path::Path;
//! use svccat::{discovery, drift, manifest::Manifest};
//!
//! # fn main() -> anyhow::Result<()> {
//! let root = Path::new(".");
//! let manifest = Manifest::load(&root.join("services.yaml"))?;
//! let discovered = discovery::discover_services(root, &manifest);
//! let report = drift::analyze(&manifest, &discovered, root);
//! println!("{} drift item(s)", report.drifts.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## API stability
//!
//! Only the modules documented here are part of the stable, semver-covered
//! public API: [`config`], [`discovery`], [`drift`], [`manifest`], and
//! [`report`]. Every other module is an implementation detail of the `svccat`
//! binary; such modules are hidden from these docs and may change in any
//! release. See `docs/API_STABILITY.md` for the full policy.

// ── Stable public API (covered by semver) ──────────────────────────────────
pub mod config;
pub mod discovery;
pub mod drift;
pub mod manifest;
pub mod report;

// ── Internal: used by the `svccat` binary, not part of the stable library API.
//    These remain `pub` so the binary crate can reach them, but they are hidden
//    from the docs and may change in any release. ────────────────────────────
#[doc(hidden)]
pub mod audit;
#[doc(hidden)]
pub mod ci;
#[doc(hidden)]
pub mod cli;
#[doc(hidden)]
pub mod cost;
#[doc(hidden)]
pub mod demo;
#[doc(hidden)]
pub mod deps;
#[doc(hidden)]
pub mod deps_graph;
#[doc(hidden)]
pub mod diff;
#[doc(hidden)]
pub mod fix;
#[doc(hidden)]
pub mod hooks;
#[doc(hidden)]
pub mod import;
#[doc(hidden)]
pub mod infer;
#[doc(hidden)]
pub mod init;
#[doc(hidden)]
pub mod lint;
#[doc(hidden)]
pub mod output;
#[doc(hidden)]
pub mod pathredaction;
#[doc(hidden)]
pub mod ping;
#[doc(hidden)]
pub mod policy;
#[doc(hidden)]
pub mod rules;
#[doc(hidden)]
pub mod scorecard;
#[doc(hidden)]
pub mod search;
#[doc(hidden)]
pub mod serve;
#[doc(hidden)]
pub mod since;
#[doc(hidden)]
pub mod snapshot;
#[doc(hidden)]
pub mod stats;
#[doc(hidden)]
pub mod tag;
#[doc(hidden)]
pub mod timefmt;
#[doc(hidden)]
pub mod urlvalidation;
#[doc(hidden)]
pub mod watch;
#[doc(hidden)]
pub mod webhook;
#[doc(hidden)]
pub mod workspace;
