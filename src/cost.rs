use crate::manifest::Manifest;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CostBreakdown {
    pub total_monthly: f64,
    pub by_platform: HashMap<String, f64>,
    pub services_count: usize,
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
}
