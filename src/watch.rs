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

/// Detect what changed between two manifest states
fn detect_changes(
    prev_services: &[manifest::ServiceEntry],
    new_services: &[manifest::ServiceEntry],
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let prev_names: HashSet<_> = prev_services.iter().map(|s| s.name.clone()).collect();
    let new_names: HashSet<_> = new_services.iter().map(|s| s.name.clone()).collect();

    let added: Vec<String> = new_names.difference(&prev_names).cloned().collect();
    let removed: Vec<String> = prev_names.difference(&new_names).cloned().collect();

    // Detect modifications by comparing common services
    let mut modified = Vec::new();
    for new_svc in new_services {
        if let Some(prev_svc) = prev_services.iter().find(|s| s.name == new_svc.name) {
            if !services_equal(prev_svc, new_svc) {
                modified.push(new_svc.name.clone());
            }
        }
    }

    (added, removed, modified)
}

/// Compare two services for equality (ignoring derived fields)
fn services_equal(a: &manifest::ServiceEntry, b: &manifest::ServiceEntry) -> bool {
    a.name == b.name
        && a.language == b.language
        && a.platform == b.platform
        && a.team == b.team
        && a.role == b.role
        && a.url == b.url
        && a.oncall == b.oncall
        && a.docs == b.docs
        && a.ci == b.ci
        && a.depends_on == b.depends_on
        && a.tags == b.tags
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
