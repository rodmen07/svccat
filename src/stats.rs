use crate::manifest::Manifest;
use colored::Colorize;

/// Metadata fields the coverage table reports on, in display order.
///
/// Every name here must be one `ServiceEntry::field_value` recognises, or the
/// row would read 0% for every service no matter what the manifest declares;
/// that agreement is pinned by `manifest`'s
/// `field_names_every_surface_checks_are_known`.
pub(crate) const FIELDS: &[&str] = &[
    "language", "platform", "team", "docs", "url", "role", "oncall",
];

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

    const BAR_WIDTH: usize = 20;
    let mut sum_pct: usize = 0;

    for name in FIELDS {
        let count = manifest
            .services
            .iter()
            .filter(|s| s.has_field(name))
            .count();
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

    let overall = sum_pct / FIELDS.len();
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
