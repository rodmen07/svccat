use crate::manifest::Manifest;
use crate::rule_schema;
use colored::Colorize;
use std::collections::HashMap;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum LintSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct LintIssue {
    pub severity: LintSeverity,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct LintResult {
    pub issues: Vec<LintIssue>,
}

impl LintResult {
    pub fn error_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == LintSeverity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| i.severity == LintSeverity::Warning)
            .count()
    }
}

// ── Lint rules ────────────────────────────────────────────────────────────────

/// Run all lint checks against the loaded manifest.
pub fn run(manifest: &Manifest) -> LintResult {
    let mut issues = Vec::new();

    // 1. Duplicate service names.
    let mut name_counts: HashMap<&str, usize> = HashMap::new();
    for svc in &manifest.services {
        *name_counts.entry(svc.name.as_str()).or_insert(0) += 1;
    }
    let mut dupes: Vec<(&str, usize)> = name_counts
        .iter()
        .filter(|(_, &c)| c > 1)
        .map(|(&n, &c)| (n, c))
        .collect();
    dupes.sort_unstable_by_key(|(n, _)| *n);
    for (name, count) in dupes {
        issues.push(LintIssue {
            severity: LintSeverity::Error,
            message: format!("duplicate service name '{}' appears {} times", name, count),
        });
    }

    // 2. Blank/empty service names.
    for (i, svc) in manifest.services.iter().enumerate() {
        if svc.name.trim().is_empty() {
            issues.push(LintIssue {
                severity: LintSeverity::Error,
                message: format!("service at index {} has a blank or whitespace-only name", i),
            });
        }
    }

    // 3. Self-referential depends_on.
    for svc in &manifest.services {
        if svc.depends_on.iter().any(|d| d == &svc.name) {
            issues.push(LintIssue {
                severity: LintSeverity::Error,
                message: format!("'{}' lists itself in depends_on", svc.name),
            });
        }
    }

    // 4. Blank entries in depends_on list.
    for svc in &manifest.services {
        if svc.depends_on.iter().any(|d| d.trim().is_empty()) {
            issues.push(LintIssue {
                severity: LintSeverity::Error,
                message: format!("'{}' has a blank entry in its depends_on list", svc.name),
            });
        }
    }

    // 5. Duplicate depends_on entries for the same service.
    for svc in &manifest.services {
        let mut seen: HashMap<&str, usize> = HashMap::new();
        for dep in &svc.depends_on {
            *seen.entry(dep.as_str()).or_insert(0) += 1;
        }
        let mut dup_deps: Vec<&str> = seen
            .iter()
            .filter(|(_, &c)| c > 1)
            .map(|(&d, _)| d)
            .collect();
        dup_deps.sort_unstable();
        for dep in dup_deps {
            issues.push(LintIssue {
                severity: LintSeverity::Warning,
                message: format!(
                    "'{}' lists '{}' more than once in depends_on",
                    svc.name, dep
                ),
            });
        }
    }

    // 6. Unrecognised manifest version.
    if !["1", "2"].contains(&manifest.version.as_str()) {
        issues.push(LintIssue {
            severity: LintSeverity::Warning,
            message: format!(
                "manifest version '{}' is unrecognised (expected '1' or '2')",
                manifest.version
            ),
        });
    }

    // 7. Duplicate service URLs.
    let mut url_map: HashMap<&str, Vec<&str>> = HashMap::new();
    for svc in &manifest.services {
        if let Some(ref url) = svc.url {
            let u = url.as_str();
            if !u.is_empty() {
                url_map.entry(u).or_default().push(svc.name.as_str());
            }
        }
    }
    let mut dup_urls: Vec<(&str, Vec<&str>)> = url_map
        .into_iter()
        .filter(|(_, names)| names.len() > 1)
        .collect();
    dup_urls.sort_unstable_by_key(|(url, _)| *url);
    for (url, names) in dup_urls {
        issues.push(LintIssue {
            severity: LintSeverity::Warning,
            message: format!(
                "url '{}' is shared by multiple services: {}",
                url,
                names.join(", ")
            ),
        });
    }

    // 8. Cross-platform depends_on edges.
    // Warn when service A (platform X) declares a dependency on service B (platform Y != X).
    // This often indicates a misconfigured entry or an undocumented cross-environment call.
    let platform_map: HashMap<&str, &str> = manifest
        .services
        .iter()
        .filter_map(|s| s.platform.as_deref().map(|p| (s.name.as_str(), p)))
        .collect();

    for svc in &manifest.services {
        let svc_platform = match svc.platform.as_deref() {
            Some(p) if !p.is_empty() => p,
            _ => continue,
        };
        for dep in &svc.depends_on {
            if let Some(&dep_platform) = platform_map.get(dep.as_str()) {
                if dep_platform != svc_platform {
                    issues.push(LintIssue {
                        severity: LintSeverity::Warning,
                        message: format!(
                            "'{}' (platform: {}) depends on '{}' (platform: {}) \
                             - cross-platform dependency",
                            svc.name, svc_platform, dep, dep_platform
                        ),
                    });
                }
            }
        }
    }

    // 9. Services with no team owner.
    for svc in &manifest.services {
        if svc.team.as_deref().map(str::is_empty).unwrap_or(true) {
            issues.push(LintIssue {
                severity: LintSeverity::Warning,
                message: format!("'{}' has no team owner (add a `team:` field)", svc.name),
            });
        }
    }

    // 10. Services with no docs reference.
    for svc in &manifest.services {
        if svc.docs.as_deref().map(str::is_empty).unwrap_or(true) {
            issues.push(LintIssue {
                severity: LintSeverity::Warning,
                message: format!("'{}' has no docs path (add a `docs:` field)", svc.name),
            });
        }
    }

    // 11. Policy rule schema validation (`policy.rules`). Delegated to a
    // focused module: the checks involved (rule id uniqueness, `base`
    // inheritance cycle detection) are non-trivial and are documented in
    // detail there, including why a cyclic `base` chain must be caught here
    // rather than left to the rule-compilation engine downstream.
    issues.extend(rule_schema::validate(&manifest.policy.rules));

    LintResult { issues }
}

// ── Rendering ─────────────────────────────────────────────────────────────────

pub fn render(result: &LintResult) {
    if result.issues.is_empty() {
        println!("{}", "✓ No lint issues found.".green().bold());
        return;
    }

    for issue in &result.issues {
        match issue.severity {
            LintSeverity::Error => {
                println!("{} {}", "✗ [error]".red().bold(), issue.message);
            }
            LintSeverity::Warning => {
                println!("{} {}", "⚠ [warn] ".yellow().bold(), issue.message);
            }
        }
    }

    println!();
    println!(
        "found {} error(s), {} warning(s)",
        result.error_count(),
        result.warning_count()
    );
}
