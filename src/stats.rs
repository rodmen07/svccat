use crate::manifest::{Manifest, ServiceEntry};
use colored::Colorize;

/// Print a field-coverage summary table with ASCII bar charts.
///
/// For each tracked metadata field, shows how many services have it set
/// along with a percentage and a coloured progress bar.  Finishes with
/// an overall health score (average coverage across all fields).
pub fn run(manifest: &Manifest) {
    let total = manifest.services.len();
    if total == 0 {
        println!("svccat stats: no services declared.");
        return;
    }

    let s = if total == 1 { "" } else { "s" };
    println!("{}", format!("svccat stats  [{total} service{s}]").bold());
    println!();
    println!("Field Coverage:");
    println!();

    type CheckFn = fn(&ServiceEntry) -> bool;
    let fields: &[(&str, CheckFn)] = &[
        ("language", |s| filled(s.language.as_deref())),
        ("platform", |s| filled(s.platform.as_deref())),
        ("team", |s| filled(s.team.as_deref())),
        ("docs", |s| filled(s.docs.as_deref())),
        ("url", |s| filled(s.url.as_deref())),
        ("role", |s| filled(s.role.as_deref())),
        ("oncall", |s| filled(s.oncall.as_deref())),
    ];

    const BAR_WIDTH: usize = 20;
    let mut sum_pct: usize = 0;

    for (name, check) in fields {
        let count = manifest.services.iter().filter(|s| check(s)).count();
        let pct = count * 100 / total;
        let filled_len = count * BAR_WIDTH / total;
        let bar = format!(
            "{}{}",
            "█".repeat(filled_len),
            "░".repeat(BAR_WIDTH - filled_len)
        );

        let pct_label = format!("{pct:>3}%");
        let coloured_pct = if pct == 100 {
            pct_label.green().bold()
        } else if pct >= 50 {
            pct_label.yellow().bold()
        } else {
            pct_label.red().bold()
        };

        println!(
            "  {:<10}  {:>3}/{:<3}  {}  {}",
            name, count, total, bar, coloured_pct
        );

        sum_pct += pct;
    }

    let overall = sum_pct / fields.len();
    println!();
    let health_label = format!("Overall health: {overall}%");
    let coloured_health = if overall == 100 {
        health_label.green().bold()
    } else if overall >= 50 {
        health_label.yellow().bold()
    } else {
        health_label.red().bold()
    };
    println!("  {coloured_health}");
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn filled(v: Option<&str>) -> bool {
    v.map(|s| !s.is_empty()).unwrap_or(false)
}
