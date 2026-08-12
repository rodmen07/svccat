use crate::manifest::Manifest;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CostBreakdown {
    pub total_monthly: f64,
    pub by_platform: HashMap<String, f64>,
    pub services_count: usize,
}

impl CostBreakdown {
    /// The platforms in display order: dearest first, ties broken by name.
    ///
    /// `by_platform` is a `HashMap`, so its iteration order is randomised per
    /// process and carries no meaning. Any renderer that walks it must impose a
    /// TOTAL order or it prints something different on every run — which is what
    /// the terminal renderer did, ordering by `(cost as i32).wrapping_neg()` with
    /// a stable sort, so equal keys simply kept the hash order. The truncation
    /// widened the tie class beyond exact equality as well: `$10.90` and `$10.20`
    /// both became `10`.
    ///
    /// Here the key is the real `f64` compared with `total_cmp` (a total order
    /// over every `f64`, unlike `partial_cmp`, which has no answer for NaN), and
    /// the platform name breaks the remaining ties. The result therefore depends
    /// only on the data.
    ///
    /// `audit::render_json` deliberately keeps its own `BTreeMap`, i.e. name
    /// order: a JSON object is addressed by key rather than read top to bottom,
    /// and it is already deterministic, so re-ordering that document would change
    /// a machine-readable output for no gain.
    pub fn platforms_by_cost(&self) -> Vec<(&str, f64)> {
        let mut platforms: Vec<(&str, f64)> = self
            .by_platform
            .iter()
            .map(|(platform, &cost)| (platform.as_str(), cost))
            .collect();
        platforms.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(b.0)));
        platforms
    }
}

/// Platform name (case-insensitive prefix matching) to estimated monthly cost (USD).
fn cost_estimates() -> HashMap<&'static str, f64> {
    [
        ("cloud run", 50.0),
        ("cloud sql", 75.0),
        ("gcp", 75.0),
        ("fly.io", 20.0),
        ("vercel", 10.0),
        ("github pages", 0.0),
        ("aws lambda", 15.0),
        ("aws ec2", 100.0),
        ("kubernetes", 200.0),
        ("heroku", 50.0),
        ("render", 30.0),
    ]
    .iter()
    .cloned()
    .collect()
}

/// Estimate the cost for a single platform.
fn estimate_platform_cost(platform: &str) -> f64 {
    let estimates = cost_estimates();
    let lower = platform.to_lowercase();

    // Try exact match first
    if let Some(cost) = estimates.get(lower.as_str()) {
        return *cost;
    }

    // Try prefix match (case-insensitive)
    for (key, cost) in estimates {
        if lower.starts_with(key) {
            return cost;
        }
    }

    // Default for unknown platforms: assume minimal cost
    10.0
}

/// Analyze cost based on the manifest's declared platforms.
pub fn analyze(manifest: &Manifest) -> CostBreakdown {
    let mut by_platform: HashMap<String, f64> = HashMap::new();
    let mut services_count = 0;

    for service in &manifest.services {
        services_count += 1;
        if let Some(ref platform) = service.platform {
            let cost = estimate_platform_cost(platform);
            *by_platform.entry(platform.clone()).or_insert(0.0) += cost;
        }
    }

    let total_monthly: f64 = by_platform.values().sum();

    CostBreakdown {
        total_monthly,
        by_platform,
        services_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_cost_exact_match() {
        assert_eq!(estimate_platform_cost("Cloud Run"), 50.0);
        assert_eq!(estimate_platform_cost("Fly.io"), 20.0);
    }

    #[test]
    fn test_platform_cost_prefix_match() {
        assert_eq!(estimate_platform_cost("GCP Cloud Run"), 75.0);
        assert_eq!(estimate_platform_cost("AWS EC2"), 100.0);
    }

    #[test]
    fn test_platform_cost_unknown() {
        assert_eq!(estimate_platform_cost("Unknown Platform"), 10.0);
    }

    /// Build a breakdown straight from `(platform, cost)` pairs.
    fn breakdown(rows: &[(&str, f64)]) -> CostBreakdown {
        let by_platform: HashMap<String, f64> =
            rows.iter().map(|(p, c)| ((*p).to_string(), *c)).collect();
        CostBreakdown {
            total_monthly: by_platform.values().sum(),
            by_platform,
            services_count: rows.len(),
        }
    }

    fn names(rows: &[(&str, f64)]) -> Vec<String> {
        rows.iter().map(|(p, _)| (*p).to_string()).collect()
    }

    #[test]
    fn platforms_by_cost_puts_the_dearest_platform_first() {
        let b = breakdown(&[("Vercel", 10.0), ("Kubernetes", 200.0), ("AWS EC2", 100.0)]);
        assert_eq!(
            names(&b.platforms_by_cost()),
            vec!["Kubernetes", "AWS EC2", "Vercel"]
        );
    }

    #[test]
    fn platforms_of_equal_cost_are_ordered_by_name() {
        let b = breakdown(&[("Vercel", 10.0), ("Railway", 10.0), ("Netlify", 10.0)]);
        assert_eq!(
            names(&b.platforms_by_cost()),
            vec!["Netlify", "Railway", "Vercel"]
        );
    }

    /// The old key was `(cost as i32).wrapping_neg()`, which truncated cents, so
    /// costs that differ by less than a dollar compared EQUAL and fell back to
    /// hash order. Nothing in `analyze` produces fractional estimates today, but
    /// `by_platform` is a public field a caller may populate itself, and the
    /// ordering contract should hold for whatever it holds.
    #[test]
    fn platforms_by_cost_does_not_truncate_cents() {
        let b = breakdown(&[("cheap", 10.20), ("dear", 10.90)]);
        assert_eq!(names(&b.platforms_by_cost()), vec!["dear", "cheap"]);
    }

    /// Every `HashMap` gets its own random seed, so building the same breakdown
    /// repeatedly inside one process still samples different iteration orders.
    /// One agreement proves nothing; sixty-four is the point.
    #[test]
    fn platforms_by_cost_is_the_same_whatever_order_the_map_iterates() {
        let rows = [
            ("Vercel", 10.0),
            ("Railway", 10.0),
            ("Netlify", 10.0),
            ("Kubernetes", 200.0),
        ];
        let expected = vec!["Kubernetes", "Netlify", "Railway", "Vercel"];
        for i in 0..64 {
            let b = breakdown(&rows);
            assert_eq!(
                names(&b.platforms_by_cost()),
                expected,
                "iteration {i} disagreed"
            );
        }
    }

    /// `estimate_platform_cost` falls through to a prefix scan over the estimate
    /// table, and that table is a `HashMap` too — so if any key were a prefix of
    /// another, which of the two matched would depend on hash order and the same
    /// platform string could be priced differently from run to run. It is not a
    /// live defect today, and this is what keeps it from becoming one when a
    /// platform is added to the table.
    #[test]
    fn no_estimate_key_is_a_prefix_of_another_so_the_scan_order_cannot_matter() {
        let keys: Vec<&str> = cost_estimates().keys().copied().collect();
        for a in &keys {
            for b in &keys {
                assert!(
                    a == b || !b.starts_with(a),
                    "estimate key `{a}` is a prefix of `{b}`: which one a platform \
                     matches would depend on HashMap iteration order"
                );
            }
        }
    }
}
