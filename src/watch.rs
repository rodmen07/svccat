use crate::{discovery, drift, manifest, output};
use anyhow::Result;
use colored::Colorize;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Represents changes detected in the manifest between two runs
#[derive(Debug, Clone, Default)]
pub struct ManifestChange {
    /// Services added since last run
    pub added_services: Vec<String>,
    /// Services removed since last run
    pub removed_services: Vec<String>,
    /// Services with modified fields
    pub modified_services: Vec<String>,
    /// Whether the discovery paths changed
    pub paths_changed: bool,
    /// Total drift errors before change
    pub prev_error_count: usize,
    /// Total drift errors after change
    pub new_error_count: usize,
}

/// Maintains state for real-time watch mode
#[derive(Debug, Clone, Default)]
struct WatchState {
    last_services: Vec<manifest::ServiceEntry>,
    #[allow(dead_code)]
    last_error_count: usize,
    last_paths: Vec<PathBuf>,
}

impl ManifestChange {
    /// Check if there were any meaningful changes
    pub fn has_changes(&self) -> bool {
        !self.added_services.is_empty()
            || !self.removed_services.is_empty()
            || !self.modified_services.is_empty()
            || self.paths_changed
            || self.prev_error_count != self.new_error_count
    }

    /// Generate a human-readable summary of changes
    pub fn summary(&self) -> String {
        let mut summary = Vec::new();

        if !self.added_services.is_empty() {
            summary.push(format!("+ {} added", self.added_services.len()));
        }
        if !self.removed_services.is_empty() {
            summary.push(format!("- {} removed", self.removed_services.len()));
        }
        if !self.modified_services.is_empty() {
            summary.push(format!("~ {} modified", self.modified_services.len()));
        }
        if self.paths_changed {
            summary.push("paths changed".to_string());
        }

        if summary.is_empty() {
            "drift status changed".to_string()
        } else {
            summary.join(", ")
        }
    }
}

/// Run a continuous drift-check loop, re-triggering on file-system events.
///
/// Watches the manifest file and every effective discovery path for changes.
/// On each detected change (debounced to 500 ms) it reloads the manifest and
/// reruns the full drift analysis, printing a timestamped report.
///
/// When `since` is `Some(git_ref)`, each check is compared against the
/// manifest at that git ref so only new drift is displayed.
///
/// When `notify` is true, fires a native OS desktop notification whenever
/// the drift count changes.
///
/// Returns the initial drift error count so callers can honour `--fail-on-drift`
/// on the *first* run.
#[allow(clippy::too_many_arguments)]
pub fn run(
    manifest_path: &Path,
    root: &Path,
    ignore: &[String],
    team: Option<&str>,
    depth: u32,
    since: Option<&str>,
    notify: bool,
    interval: Option<u64>,
) -> Result<usize> {
    let manifest_path = manifest_path.to_path_buf();
    let root = root.to_path_buf();
    let ignore = ignore.to_vec();
    let team = team.map(str::to_owned);
    let since = since.map(str::to_owned);

    let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();

    let mut watcher = RecommendedWatcher::new(tx.clone(), Config::default())?;

    // Always watch the manifest file itself.
    watcher.watch(&manifest_path, RecursiveMode::NonRecursive)?;

    // Pre-load the manifest once so we can register its discovery paths.
    let initial_m = manifest::Manifest::load(&manifest_path)?;
    let watch_paths = effective_watch_paths(&initial_m, &root);
    for p in &watch_paths {
        if p.exists() {
            // Best-effort - directory may not exist yet.
            let _ = watcher.watch(p, RecursiveMode::Recursive);
        }
    }

    // Spawn a polling thread when --interval is set.
    if let Some(secs) = interval {
        let tx2 = tx.clone();
        let path_clone = manifest_path.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_secs(secs));
            // Synthesise a synthetic modify event on the manifest to trigger a recheck.
            use notify::{event::ModifyKind, Event, EventKind};
            let synthetic = Event {
                kind: EventKind::Modify(ModifyKind::Any),
                paths: vec![path_clone.clone()],
                attrs: Default::default(),
            };
            if tx2.send(Ok(synthetic)).is_err() {
                break;
            }
        });
    }

    // First run immediately.
    let initial_errors = run_once(
        &manifest_path,
        &root,
        &ignore,
        team.as_deref(),
        depth,
        since.as_deref(),
    );

    let interval_note = interval
        .map(|s| format!(" (polling every {s}s)"))
        .unwrap_or_default();
    eprintln!(
        "\n{} Watching {} and service directories{interval_note}. Press Ctrl-C to stop.\n",
        "●".cyan().bold(),
        manifest_path.display()
    );

    let debounce = Duration::from_millis(500);
    let mut last_trigger = Instant::now() - debounce * 2;
    let mut prev_errors = initial_errors;
    let mut watch_state = WatchState::default();

    // Load initial state
    if let Ok(m) = manifest::Manifest::load(&manifest_path) {
        watch_state.last_services = m.services.clone();
        watch_state.last_paths = effective_watch_paths(&m, &root);
    }

    for res in rx {
        match res {
            Ok(event) => {
                if !is_relevant(&event) {
                    continue;
                }
                // Debounce: skip if we just fired within the window.
                let now = Instant::now();
                if now.duration_since(last_trigger) < debounce {
                    continue;
                }
                last_trigger = now;

                // Re-register watch paths in case services.yaml changed.
                let mut paths_changed = false;
                if let Ok(new_m) = manifest::Manifest::load(&manifest_path) {
                    let new_paths = effective_watch_paths(&new_m, &root);
                    if new_paths != watch_state.last_paths {
                        paths_changed = true;
                        watch_state.last_paths = new_paths.clone();
                    }
                    for p in &new_paths {
                        if p.exists() {
                            let _ = watcher.watch(p, RecursiveMode::Recursive);
                        }
                    }
                }

                let new_errors = run_once(
                    &manifest_path,
                    &root,
                    &ignore,
                    team.as_deref(),
                    depth,
                    since.as_deref(),
                );

                // Detect service changes
                if let Ok(m) = manifest::Manifest::load(&manifest_path) {
                    let (added, removed, modified) =
                        detect_changes(&watch_state.last_services, &m.services);

                    let change = ManifestChange {
                        added_services: added,
                        removed_services: removed,
                        modified_services: modified,
                        paths_changed,
                        prev_error_count: prev_errors,
                        new_error_count: new_errors,
                    };

                    if change.has_changes() {
                        display_change_summary(&change);
                    }

                    watch_state.last_services = m.services;
                }

                if notify && new_errors != prev_errors {
                    let body = if new_errors == 0 {
                        "Drift cleared - all services are in sync.".to_string()
                    } else {
                        format!("{new_errors} drift error{} detected.", plural(new_errors))
                    };
                    send_os_notification(&body);
                }
                prev_errors = new_errors;
            }
            Err(e) => eprintln!("{} watcher error: {e}", "!".red()),
        }
    }

    Ok(initial_errors)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Detect what changed between two manifest states.
///
/// All three lists come back in the order the services appear in the manifest
/// they were read from — `added` and `modified` follow `new_services`,
/// `removed` follows `prev_services` — and each name appears at most once.
///
/// That ordering is part of the contract rather than an accident of the
/// implementation. `added` and `removed` used to be collected straight out of
/// `HashSet::difference`, whose iteration order is unspecified and is
/// re-randomised per set by the default hasher, so watch mode printed the same
/// change set in a different order on every reload (`+ 2 service(s): cache,
/// worker` on one run, `worker, cache` on the next) even though nothing about
/// the manifest had changed. `modified` was always built by walking
/// `new_services`, so the three lines of a change summary now share one rule
/// instead of following two.
fn detect_changes(
    prev_services: &[manifest::ServiceEntry],
    new_services: &[manifest::ServiceEntry],
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let prev_names: HashSet<&str> = prev_services.iter().map(|s| s.name.as_str()).collect();
    let new_names: HashSet<&str> = new_services.iter().map(|s| s.name.as_str()).collect();

    // The sets are membership tests only; the manifest slices decide the order.
    let added =
        names_in_manifest_order(new_services, |svc| !prev_names.contains(svc.name.as_str()));
    let removed =
        names_in_manifest_order(prev_services, |svc| !new_names.contains(svc.name.as_str()));
    let modified = names_in_manifest_order(new_services, |new_svc| {
        prev_services
            .iter()
            .find(|s| s.name == new_svc.name)
            .is_some_and(|prev_svc| !services_equal(prev_svc, new_svc))
    });

    (added, removed, modified)
}

/// Names of the services matching `keep`, in the order they appear in
/// `services`, keeping only the first entry of a duplicated name.
///
/// Deduplication is by name and independent of `keep`, so a manifest that
/// declares the same `name:` twice contributes it to at most one position of at
/// most one list.
fn names_in_manifest_order(
    services: &[manifest::ServiceEntry],
    mut keep: impl FnMut(&manifest::ServiceEntry) -> bool,
) -> Vec<String> {
    let mut seen: HashSet<&str> = HashSet::new();
    let mut names = Vec::new();
    for svc in services {
        if seen.insert(svc.name.as_str()) && keep(svc) {
            names.push(svc.name.clone());
        }
    }
    names
}

/// Compare two services for equality.
///
/// Delegates to `ServiceEntry`'s derived `PartialEq` instead of listing fields
/// by hand. The hand-written list this replaces compared 11 of the struct's 13
/// fields and silently omitted `path` and `submodule` — the two fields that
/// decide where a service lives on disk (`ServiceEntry::declared_path`) — so
/// re-pointing a service in `services.yaml` was never reported as a
/// modification. A derived comparison reads the struct definition itself, so a
/// field added later cannot fall out of change detection the same way.
fn services_equal(a: &manifest::ServiceEntry, b: &manifest::ServiceEntry) -> bool {
    a == b
}

/// Display a summary of changes in a visually clear format
fn display_change_summary(change: &ManifestChange) {
    if change.added_services.is_empty()
        && change.removed_services.is_empty()
        && change.modified_services.is_empty()
        && !change.paths_changed
    {
        return; // No service changes, only drift status
    }

    eprintln!("\n{} Manifest changes detected:", "○".cyan());

    if !change.added_services.is_empty() {
        eprintln!(
            "  {} {} service(s): {}",
            "+".green(),
            change.added_services.len(),
            change.added_services.join(", ")
        );
    }

    if !change.removed_services.is_empty() {
        eprintln!(
            "  {} {} service(s): {}",
            "-".red(),
            change.removed_services.len(),
            change.removed_services.join(", ")
        );
    }

    if !change.modified_services.is_empty() {
        eprintln!(
            "  {} {} service(s): {}",
            "~".yellow(),
            change.modified_services.len(),
            change.modified_services.join(", ")
        );
    }

    if change.paths_changed {
        eprintln!("  {} Discovery paths changed", "↻".cyan());
    }

    // Display drift status change
    if change.prev_error_count != change.new_error_count {
        let diff = change.new_error_count as i32 - change.prev_error_count as i32;
        if diff > 0 {
            eprintln!("  {} {} new drift error(s)", "⚠".yellow(), diff.abs());
        } else if diff < 0 {
            eprintln!("  {} {} drift error(s) resolved", "✓".green(), diff.abs());
        }
    }
}

fn run_once(
    manifest_path: &Path,
    root: &Path,
    ignore: &[String],
    team: Option<&str>,
    depth: u32,
    since: Option<&str>,
) -> usize {
    let ts = timestamp();
    match manifest::Manifest::load(manifest_path) {
        Err(e) => {
            eprintln!("\n[{ts}] {} reloading manifest: {e:#}", "error".red());
            0
        }
        Ok(mut m) => {
            // Apply team filter.
            if let Some(t) = team {
                m.services.retain(|s| {
                    s.team
                        .as_deref()
                        .map(|team_val| team_val.eq_ignore_ascii_case(t))
                        .unwrap_or(false)
                });
            }

            let discovered = discovery::discover_services_with_opts(root, &m, ignore, depth);
            let mut report = drift::analyze(&m, &discovered, root);
            report.manifest = manifest_path.display().to_string();

            eprintln!("\n[{ts}] change detected - re-running drift check");

            if let Some(git_ref) = since {
                if let Ok(old_m) = crate::since::load_at_ref(root, manifest_path, git_ref) {
                    let mut old_report = drift::analyze(&old_m, &discovered, root);
                    old_report.manifest = manifest_path.display().to_string();
                    let (new_count, _) =
                        output::terminal::render_since_diff(&old_report, &report, git_ref);
                    return new_count;
                }
            }

            output::terminal::render_check(&report, &[]);
            report.error_count()
        }
    }
}

fn effective_watch_paths(m: &manifest::Manifest, root: &Path) -> Vec<PathBuf> {
    m.effective_discovery_paths()
        .iter()
        .filter_map(|pat| {
            // Strip the glob wildcard to get the parent directory to watch.
            let without_glob = pat
                .split('*')
                .next()
                .unwrap_or("")
                .trim_end_matches('/')
                .to_string();
            if without_glob.is_empty() {
                None
            } else {
                Some(root.join(without_glob))
            }
        })
        .collect()
}

fn is_relevant(event: &notify::Event) -> bool {
    matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    )
}

fn timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let (h, m, s) = (secs % 86400 / 3600, secs % 3600 / 60, secs % 60);
    format!("{h:02}:{m:02}:{s:02} UTC")
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

/// Fire a native desktop notification using platform-specific tooling.
///
/// Best-effort: any error launching the notification process is silently ignored.
fn send_os_notification(body: &str) {
    #[cfg(target_os = "windows")]
    {
        // Use PowerShell to show a Windows balloon notification via NotifyIcon.
        let script = format!(
            "Add-Type -AssemblyName System.Windows.Forms; \
             $n = New-Object System.Windows.Forms.NotifyIcon; \
             $n.Icon = [System.Drawing.SystemIcons]::Information; \
             $n.Visible = $true; \
             $n.ShowBalloonTip(5000, 'svccat', '{}', \
               [System.Windows.Forms.ToolTipIcon]::None); \
             Start-Sleep 6; $n.Dispose()",
            body.replace('\'', "''")
        );
        let _ = std::process::Command::new("powershell")
            .args([
                "-WindowStyle",
                "Hidden",
                "-NonInteractive",
                "-Command",
                &script,
            ])
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let script = format!("display notification {:?} with title \"svccat\"", body);
        let _ = std::process::Command::new("osascript")
            .args(["-e", &script])
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("notify-send")
            .args(["svccat", body])
            .spawn();
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::{
        AccessKind, CreateKind, DataChange, EventAttributes, ModifyKind, RemoveKind,
    };
    use notify::Event;

    fn event_of(kind: EventKind) -> Event {
        Event {
            kind,
            paths: Vec::new(),
            attrs: EventAttributes::default(),
        }
    }

    /// One field of `ServiceEntry` plus the edit that changes it.
    type FieldMutator = (&'static str, fn(&mut manifest::ServiceEntry));

    /// A service with EVERY `ServiceEntry` field populated, so a mutation test
    /// changes a value rather than filling in a `None`.
    ///
    /// Written as an exhaustive struct literal with no `..Default::default()`
    /// on purpose: adding a 14th field to `ServiceEntry` then stops this file
    /// COMPILING until the field is given a value here and a mutator in
    /// `every_non_name_field_of_service_entry_counts_as_a_modification` below.
    /// That build failure is the signal `path` and `submodule` never got when
    /// they were left out of the change comparison.
    fn populated_service(name: &str) -> manifest::ServiceEntry {
        manifest::ServiceEntry {
            name: name.to_string(),
            language: Some("Rust".to_string()),
            platform: Some("Cloud Run".to_string()),
            url: Some("https://api.example.com".to_string()),
            role: Some("Service".to_string()),
            team: Some("platform".to_string()),
            oncall: Some("@platform-oncall".to_string()),
            submodule: Some("vendor/api".to_string()),
            path: Some("services/api".to_string()),
            docs: Some("docs/api.md".to_string()),
            ci: Some(".github/workflows/api.yml".to_string()),
            tags: vec!["critical".to_string()],
            depends_on: vec!["db".to_string()],
        }
    }

    /// The regression test for the defect this module shipped with: editing a
    /// service's `path` or `submodule` in `services.yaml` was never reported by
    /// watch mode, because `services_equal` hand-listed 11 of `ServiceEntry`'s
    /// 13 fields and those two were missing. Against the old comparison the
    /// `path` and `submodule` iterations of this loop fail; the other ten pass,
    /// which is what makes the assertion non-tautological.
    #[test]
    fn every_non_name_field_of_service_entry_counts_as_a_modification() {
        // One entry per field `populated_service` sets, except `name`; the
        // exhaustive literal there is what forces a new field to arrive here.
        let mutators: Vec<FieldMutator> = vec![
            ("language", |s| s.language = Some("Go".to_string())),
            ("platform", |s| s.platform = Some("Fly.io".to_string())),
            ("url", |s| s.url = Some("https://api.internal".to_string())),
            ("role", |s| s.role = Some("Worker".to_string())),
            ("team", |s| s.team = Some("growth".to_string())),
            ("oncall", |s| s.oncall = Some("@growth-oncall".to_string())),
            ("submodule", |s| {
                s.submodule = Some("vendor/api-v2".to_string())
            }),
            ("path", |s| s.path = Some("services/api-v2".to_string())),
            ("docs", |s| s.docs = Some("docs/api-v2.md".to_string())),
            ("ci", |s| {
                s.ci = Some(".github/workflows/v2.yml".to_string())
            }),
            ("tags", |s| s.tags = vec!["beta".to_string()]),
            ("depends_on", |s| s.depends_on = vec!["cache".to_string()]),
        ];
        assert_eq!(
            mutators.len(),
            12,
            "one mutator per `ServiceEntry` field except `name` (a name change is \
             an add plus a remove, asserted separately)"
        );

        let base = populated_service("api");
        for (field, mutate) in mutators {
            let mut changed = base.clone();
            mutate(&mut changed);

            let (added, removed, modified) =
                detect_changes(std::slice::from_ref(&base), std::slice::from_ref(&changed));

            assert!(
                added.is_empty() && removed.is_empty(),
                "editing `{field}` is neither an addition nor a removal, got \
                 added={added:?} removed={removed:?}"
            );
            assert_eq!(
                modified,
                vec!["api".to_string()],
                "editing `{field}` in services.yaml is not reported as a \
                 modification by watch mode"
            );
        }
    }

    /// A name change is the one edit that is NOT a modification: the service is
    /// keyed by name, so it reads as one removal plus one addition.
    #[test]
    fn detect_changes_reports_a_renamed_service_as_added_plus_removed() {
        let prev = vec![populated_service("api")];
        let new = vec![populated_service("api-gateway")];

        let (added, removed, modified) = detect_changes(&prev, &new);

        assert_eq!(added, vec!["api-gateway".to_string()]);
        assert_eq!(removed, vec!["api".to_string()]);
        assert!(modified.is_empty(), "got {modified:?}");
    }

    /// Additions, removals and modifications in a single reload, which is what
    /// a real `services.yaml` edit looks like. The modification here is a
    /// `path` re-point, i.e. the exact edit the shipped comparison ignored.
    #[test]
    fn detect_changes_reports_additions_removals_and_modifications_together() {
        let prev = vec![
            populated_service("api"),
            populated_service("web"),
            populated_service("legacy"),
        ];

        let mut api_moved = populated_service("api");
        api_moved.path = Some("services/api-v2".to_string());
        let new = vec![
            api_moved,
            populated_service("web"),
            populated_service("worker"),
        ];

        let (added, removed, modified) = detect_changes(&prev, &new);

        assert_eq!(added, vec!["worker".to_string()]);
        assert_eq!(removed, vec!["legacy".to_string()]);
        assert_eq!(modified, vec!["api".to_string()]);
    }

    /// Ordering is user-visible, not an internal detail: `display_change_summary`
    /// joins these lists straight into the "Manifest changes detected" summary.
    /// Both lists here are in an order that is neither alphabetical nor
    /// reverse-alphabetical, so sorting the results fails this assertion exactly
    /// as loudly as the `HashSet::difference` iteration it replaced.
    #[test]
    fn added_and_removed_are_reported_in_manifest_order() {
        let removed_order = ["zulu", "alpha", "mike", "bravo", "yankee", "charlie"];
        let added_order = ["sierra", "delta", "november", "echo", "victor", "foxtrot"];

        let prev: Vec<_> = removed_order.iter().map(|n| populated_service(n)).collect();
        let new: Vec<_> = added_order.iter().map(|n| populated_service(n)).collect();

        let (added, removed, modified) = detect_changes(&prev, &new);

        assert_eq!(added, added_order, "added must follow the new manifest");
        assert_eq!(
            removed, removed_order,
            "removed must follow the previous manifest"
        );
        assert!(modified.is_empty(), "got {modified:?}");
    }

    /// The defect being pinned is a *nondeterminism*, so a single call proves
    /// nothing: the previous implementation returned the correct set in an
    /// arbitrary order, and the default hasher re-randomises that order for
    /// every set it builds. Identical input must therefore produce an identical
    /// answer on every call, not merely an equivalent one.
    #[test]
    fn detect_changes_returns_the_same_order_on_every_call() {
        let prev: Vec<_> = ["zulu", "alpha", "mike", "bravo"]
            .iter()
            .map(|n| populated_service(n))
            .collect();
        let mut new: Vec<_> = ["sierra", "delta", "november", "echo"]
            .iter()
            .map(|n| populated_service(n))
            .collect();
        // One survivor, edited, so the same loop also covers `modified`.
        let mut alpha_moved = populated_service("alpha");
        alpha_moved.path = Some("services/alpha-v2".to_string());
        new.push(alpha_moved);

        let first = detect_changes(&prev, &new);
        assert_eq!(first.0.len(), 4, "added: {:?}", first.0);
        assert_eq!(first.1.len(), 3, "removed: {:?}", first.1);
        assert_eq!(first.2, vec!["alpha".to_string()]);

        for run in 1..64 {
            assert_eq!(
                detect_changes(&prev, &new),
                first,
                "run {run} disagreed with run 0 on byte-identical input"
            );
        }
    }

    /// A `services.yaml` that declares the same `name:` twice must not print the
    /// service twice in one summary line. `added` and `removed` used to
    /// deduplicate implicitly because they were built from sets while `modified`
    /// did not, so the three lists disagreed about what a duplicate meant.
    #[test]
    fn a_duplicated_service_name_is_reported_once() {
        let prev = vec![populated_service("api"), populated_service("api")];
        let mut api_moved = populated_service("api");
        api_moved.path = Some("services/api-v2".to_string());
        let new = vec![api_moved.clone(), api_moved];

        let (added, removed, modified) = detect_changes(&prev, &new);

        assert!(
            added.is_empty() && removed.is_empty(),
            "added={added:?} removed={removed:?}"
        );
        assert_eq!(modified, vec!["api".to_string()]);
    }

    /// Nothing changed, including a reordered `services:` list: watch mode must
    /// stay quiet, otherwise every unrelated file event prints a change summary.
    #[test]
    fn detect_changes_is_silent_when_nothing_changed_including_reordering() {
        let prev = vec![populated_service("api"), populated_service("web")];
        let new = vec![populated_service("web"), populated_service("api")];

        let (added, removed, modified) = detect_changes(&prev, &new);

        assert!(added.is_empty(), "got {added:?}");
        assert!(removed.is_empty(), "got {removed:?}");
        assert!(modified.is_empty(), "got {modified:?}");
    }

    /// Both empty-list directions: the first service added to an empty manifest
    /// and the last one removed from it.
    #[test]
    fn detect_changes_handles_empty_manifests_in_both_directions() {
        let empty: Vec<manifest::ServiceEntry> = Vec::new();
        let one = vec![populated_service("first")];

        let (added, removed, modified) = detect_changes(&empty, &one);
        assert_eq!(added, vec!["first".to_string()]);
        assert!(removed.is_empty() && modified.is_empty());

        let (added, removed, modified) = detect_changes(&one, &empty);
        assert!(added.is_empty() && modified.is_empty());
        assert_eq!(removed, vec!["first".to_string()]);
    }

    /// `is_relevant` is the only place svccat interprets `notify`'s event
    /// vocabulary, so it is the surface a `notify` major bump can silently
    /// change. Both directions are asserted: the three kinds that must trigger
    /// a re-check, and the kinds that must NOT (an `Access` storm re-running
    /// the whole drift analysis on every read is the failure this guards).
    #[test]
    fn is_relevant_accepts_create_modify_remove_and_rejects_everything_else() {
        assert!(is_relevant(&event_of(EventKind::Create(CreateKind::File))));
        assert!(is_relevant(&event_of(EventKind::Modify(ModifyKind::Data(
            DataChange::Content
        )))));
        assert!(is_relevant(&event_of(EventKind::Modify(ModifyKind::Any))));
        assert!(is_relevant(&event_of(EventKind::Remove(RemoveKind::File))));

        // Negative control. `Access` matters specifically because notify 7.0
        // started reporting inotify open/access events on Linux; svccat must
        // keep ignoring them.
        assert!(!is_relevant(&event_of(EventKind::Access(
            AccessKind::Open(notify::event::AccessMode::Read)
        ))));
        assert!(!is_relevant(&event_of(EventKind::Access(AccessKind::Read))));
        assert!(!is_relevant(&event_of(EventKind::Any)));
        assert!(!is_relevant(&event_of(EventKind::Other)));
    }

    /// End-to-end proof that the backend `RecommendedWatcher` resolves to on
    /// THIS platform actually delivers a relevant event for a real file write,
    /// wired exactly the way `run` wires it (`mpsc::Sender` event handler,
    /// `Config::default()`, `RecursiveMode::Recursive`).
    ///
    /// The three `Build & Test (This Checkout)` legs run this on ubuntu,
    /// windows and macos, i.e. on inotify, `ReadDirectoryChangesW` and
    /// FSEvents respectively — three different implementations behind one
    /// type alias. Without this, "it compiled" is the only evidence that
    /// `svccat watch` / `svccat ci --watch` still see anything at all.
    #[test]
    fn recommended_watcher_delivers_a_relevant_event_for_a_real_write() {
        let dir = tempfile::tempdir().expect("temp dir");
        // macOS hands out /var/... temp dirs that FSEvents reports back as
        // /private/var/..., so watch the canonical path.
        let root = dir.path().canonicalize().expect("canonicalize temp dir");

        let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();
        let mut watcher =
            RecommendedWatcher::new(tx, Config::default()).expect("build recommended watcher");
        watcher
            .watch(&root, RecursiveMode::Recursive)
            .expect("watch temp dir");

        let target = root.join("services.yaml");
        let deadline = Instant::now() + Duration::from_secs(30);
        let mut writes = 0u32;
        let mut saw_relevant = false;

        while Instant::now() < deadline && !saw_relevant {
            // Re-write on every idle tick so a slow watcher start-up cannot
            // permanently lose the single event this test is waiting for.
            writes += 1;
            std::fs::write(&target, format!("services: []\n# write {writes}\n"))
                .expect("write manifest");

            loop {
                match rx.recv_timeout(Duration::from_millis(500)) {
                    Ok(Ok(event)) => {
                        if is_relevant(&event)
                            && event
                                .paths
                                .iter()
                                .any(|p| p.file_name() == target.file_name())
                        {
                            saw_relevant = true;
                            break;
                        }
                    }
                    Ok(Err(e)) => panic!("watcher reported an error: {e}"),
                    Err(mpsc::RecvTimeoutError::Timeout) => break,
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        panic!("watcher channel disconnected before any event arrived")
                    }
                }
            }
        }

        assert!(
            saw_relevant,
            "no Create/Modify/Remove event for {} arrived within 30s after {writes} write(s); \
             the notify backend for this platform is not delivering events to svccat",
            target.display()
        );
    }
}
