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
/// When `since` is `Some(git_ref)`, each check is compared against the
/// manifest at that git ref so only new drift is displayed.
///
/// When `notify` is true, fires a native OS desktop notification whenever
/// the drift count changes.
///
/// Returns the initial drift error count so callers can honour `--fail-on-drift`
/// on the *first* run.
pub fn run(
    manifest_path: &Path,
    root: &Path,
    ignore: &[String],
    team: Option<&str>,
    depth: u32,
    since: Option<&str>,
    notify: bool,
) -> Result<usize> {
    let manifest_path = manifest_path.to_path_buf();
    let root = root.to_path_buf();
    let ignore = ignore.to_vec();
    let team = team.map(str::to_owned);
    let since = since.map(str::to_owned);

    let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

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

    // First run immediately.
    let initial_errors = run_once(&manifest_path, &root, &ignore, team.as_deref(), depth, since.as_deref());

    eprintln!(
        "\n{} Watching {} and service directories. Press Ctrl-C to stop.\n",
        "●".cyan().bold(),
        manifest_path.display()
    );

    let debounce = Duration::from_millis(500);
    let mut last_trigger = Instant::now() - debounce * 2;
    let mut prev_errors = initial_errors;

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

                let new_errors = run_once(&manifest_path, &root, &ignore, team.as_deref(), depth, since.as_deref());

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

fn run_once(manifest_path: &Path, root: &Path, ignore: &[String], team: Option<&str>, depth: u32, since: Option<&str>) -> usize {
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
    if n == 1 { "" } else { "s" }
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
            .args(["-WindowStyle", "Hidden", "-NonInteractive", "-Command", &script])
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
