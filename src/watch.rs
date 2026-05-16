use crate::{discovery, drift, manifest, output};
use anyhow::Result;
use colored::Colorize;
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Run a continuous drift-check loop, re-triggering on file-system events.
///
/// Watches the manifest file and every effective discovery path for changes.
/// On each detected change (debounced to 500 ms) it reloads the manifest and
/// reruns the full drift analysis, printing a timestamped report.
///
/// Returns the initial drift error count so callers can honour `--fail-on-drift`
/// on the *first* run.
pub fn run(
    manifest_path: &Path,
    root: &Path,
    ignore: &[String],
    team: Option<&str>,
) -> Result<usize> {
    let manifest_path = manifest_path.to_path_buf();
    let root = root.to_path_buf();
    let ignore = ignore.to_vec();
    let team = team.map(str::to_owned);

    let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    // Always watch the manifest file itself.
    watcher.watch(&manifest_path, RecursiveMode::NonRecursive)?;

    // Pre-load the manifest once so we can register its discovery paths.
    let initial_m = manifest::Manifest::load(&manifest_path)?;
    let watch_paths = effective_watch_paths(&initial_m, &root);
    for p in &watch_paths {
        if p.exists() {
            // Best-effort – directory may not exist yet.
            let _ = watcher.watch(p, RecursiveMode::Recursive);
        }
    }

    // First run immediately.
    let initial_errors = run_once(&manifest_path, &root, &ignore, team.as_deref());

    eprintln!(
        "\n{} Watching {} and service directories. Press Ctrl-C to stop.\n",
        "●".cyan().bold(),
        manifest_path.display()
    );

    let debounce = Duration::from_millis(500);
    let mut last_trigger = Instant::now() - debounce * 2;

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
                if let Ok(new_m) = manifest::Manifest::load(&manifest_path) {
                    for p in effective_watch_paths(&new_m, &root) {
                        if p.exists() {
                            let _ = watcher.watch(&p, RecursiveMode::Recursive);
                        }
                    }
                }

                run_once(&manifest_path, &root, &ignore, team.as_deref());
            }
            Err(e) => eprintln!("{} watcher error: {e}", "!".red()),
        }
    }

    Ok(initial_errors)
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn run_once(manifest_path: &Path, root: &Path, ignore: &[String], team: Option<&str>) -> usize {
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

            let discovered = discovery::discover_services_with_ignore(root, &m, ignore);
            let mut report = drift::analyze(&m, &discovered, root);
            report.manifest = manifest_path.display().to_string();

            eprintln!("\n[{ts}] change detected — re-running drift check");
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
