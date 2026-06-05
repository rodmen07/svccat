use crate::discovery::DiscoveredService;
use crate::manifest::Manifest;
use crate::rules::RuleEngine;
use serde::{Deserialize, Serialize};
use std::path::Path;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum DriftKind {
    /// Service is listed in the manifest but its directory is missing from the repo.
    DeclaredMissingFromRepo,
    /// A service directory was discovered in the repo but is absent from the manifest.
    UndeclaredInRepo,
    /// A recommended metadata field is absent from the service entry.
    MissingField,
    /// A file referenced by the service entry (docs, ci) does not exist.
    MissingReferencedFile,
    /// A field required by the policy section is absent from the service entry.
    PolicyViolation,
    /// A `depends_on` entry references a service that is not declared in the manifest.
    DanglingDependency,
    /// A circular dependency was detected in the `depends_on` graph.
    CircularDependency,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DriftItem {
    pub kind: DriftKind,
    pub severity: Severity,
    pub service: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct DriftReport {
    pub manifest: String,
    pub declared: usize,
    pub discovered: usize,
    pub drifts: Vec<DriftItem>,
}

impl DriftReport {
    pub fn error_count(&self) -> usize {
        self.drifts
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.drifts
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count()
    }
}

// ── Drift rules ───────────────────────────────────────────────────────────────

/// Metadata fields that every service should declare.
/// Tuple: (field_name, is_error_if_missing)
const RECOMMENDED_FIELDS: &[(&str, bool)] =
    &[("role", true), ("language", false), ("platform", false)];

/// Run all drift checks and return a populated `DriftReport`.
pub fn analyze(manifest: &Manifest, discovered: &[DiscoveredService], root: &Path) -> DriftReport {
    let mut report = DriftReport {
        manifest: String::new(), // caller fills in the manifest path
        declared: manifest.services.len(),
        discovered: discovered.len(),
        drifts: Vec::new(),
    };

    // 1. Check every declared service.
    for svc in &manifest.services {
        // ── Presence check ─────────────────────────────────────────────────
        let found = if let Some(explicit_path) = &svc.path {
            root.join(explicit_path).exists()
        } else if let Some(sub) = &svc.submodule {
            root.join(sub).exists()
        } else {
            discovered.iter().any(|d| d.name == svc.name)
        };

        if !found {
            report.drifts.push(DriftItem {
                kind: DriftKind::DeclaredMissingFromRepo,
                severity: Severity::Error,
                service: svc.name.clone(),
                message: format!(
                    "'{}' is declared in the manifest but not found in the repo",
                    svc.name
                ),
                detail: svc.declared_path().map(str::to_owned),
            });
        }

        // ── Recommended fields ─────────────────────────────────────────────
        for (field, is_error) in RECOMMENDED_FIELDS {
            let missing = match *field {
                "role" => svc.role.is_none(),
                "language" => svc.language.is_none(),
                "platform" => svc.platform.is_none(),
                "team" => svc.team.is_none(),
                _ => false,
            };
            if missing {
                report.drifts.push(DriftItem {
                    kind: DriftKind::MissingField,
                    severity: if *is_error {
                        Severity::Error
                    } else {
                        Severity::Warning
                    },
                    service: svc.name.clone(),
                    message: format!("'{}' is missing recommended field: {}", svc.name, field),
                    detail: Some(field.to_string()),
                });
            }
        }

        // ── Referenced file checks ─────────────────────────────────────────
        for (label, opt_path) in [("docs", &svc.docs), ("ci", &svc.ci)] {
            if let Some(ref_path) = opt_path {
                if !root.join(ref_path).exists() {
                    report.drifts.push(DriftItem {
                        kind: DriftKind::MissingReferencedFile,
                        severity: Severity::Warning,
                        service: svc.name.clone(),
                        message: format!(
                            "'{}' references {} path '{}' which does not exist",
                            svc.name, label, ref_path
                        ),
                        detail: Some(ref_path.clone()),
                    });
                }
            }
        }
    }

    // 2. Flag service directories found in the repo but absent from the manifest.
    let declared_paths: std::collections::HashSet<&str> = manifest
        .services
        .iter()
        .filter_map(|s| s.declared_path())
        .collect();

    let declared_names: std::collections::HashSet<&str> =
        manifest.services.iter().map(|s| s.name.as_str()).collect();

    for disc in discovered {
        let matched_by_path = declared_paths.contains(disc.path.as_str());
        let matched_by_name = declared_names.contains(disc.name.as_str());

        if !matched_by_path && !matched_by_name {
            report.drifts.push(DriftItem {
                kind: DriftKind::UndeclaredInRepo,
                severity: Severity::Warning,
                service: disc.name.clone(),
                message: format!(
                    "'{}' exists in the repo but is not listed in the manifest",
                    disc.path
                ),
                detail: Some(disc.path.clone()),
            });
        }
    }

    // 3. Apply policy rules (require_fields).
    if !manifest.policy.require_fields.is_empty() {
        for svc in &manifest.services {
            for field in &manifest.policy.require_fields {
                let missing = match field.as_str() {
                    "url" => svc.url.is_none(),
                    "language" => svc.language.is_none(),
                    "platform" => svc.platform.is_none(),
                    "role" => svc.role.is_none(),
                    "team" => svc.team.is_none(),
                    "oncall" => svc.oncall.is_none(),
                    "docs" => svc.docs.is_none(),
                    "ci" => svc.ci.is_none(),
                    _ => false,
                };
                if missing {
                    report.drifts.push(DriftItem {
                        kind: DriftKind::PolicyViolation,
                        severity: Severity::Error,
                        service: svc.name.clone(),
                        message: format!(
                            "'{}' violates policy: required field '{}' is missing",
                            svc.name, field
                        ),
                        detail: Some(field.clone()),
                    });
                }
            }
        }
    }

    // 3b. Apply custom validation rules.
    if !manifest.policy.rules.is_empty() {
        match RuleEngine::compile(&manifest.policy.rules) {
            Ok(engine) => {
                for svc in &manifest.services {
                    for violation in engine.evaluate(svc) {
                        let severity = match violation.severity.as_str() {
                            "error" => Severity::Error,
                            "warning" => Severity::Warning,
                            _ => Severity::Warning,
                        };
                        report.drifts.push(DriftItem {
                            kind: DriftKind::PolicyViolation,
                            severity,
                            service: violation.service_name.clone(),
                            message: format!(
                                "'{}' violates policy: {}",
                                violation.service_name, violation.message
                            ),
                            detail: Some(violation.rule_id),
                        });
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to compile custom rules: {}", e);
            }
        }
    }

    // 4. Validate depends_on references and detect cycles.
    check_dependencies(manifest, &mut report);

    report
}

// ── Dependency validation ─────────────────────────────────────────────────────

fn check_dependencies(manifest: &Manifest, report: &mut DriftReport) {
    let declared: std::collections::HashSet<&str> =
        manifest.services.iter().map(|s| s.name.as_str()).collect();

    // 4a. Dangling references.
    for svc in &manifest.services {
        for dep in &svc.depends_on {
            if !declared.contains(dep.as_str()) {
                report.drifts.push(DriftItem {
                    kind: DriftKind::DanglingDependency,
                    severity: Severity::Error,
                    service: svc.name.clone(),
                    message: format!(
                        "'{}' depends_on '{}' which is not declared in the manifest",
                        svc.name, dep
                    ),
                    detail: Some(dep.clone()),
                });
            }
        }
    }

    // 4b. Cycle detection via iterative DFS.
    // Build adjacency map (only over declared services to avoid double-reporting).
    let adj: std::collections::HashMap<&str, Vec<&str>> = manifest
        .services
        .iter()
        .map(|s| {
            let deps: Vec<&str> = s
                .depends_on
                .iter()
                .filter(|d| declared.contains(d.as_str()))
                .map(String::as_str)
                .collect();
            (s.name.as_str(), deps)
        })
        .collect();

    // Track global visit state: 0 = unvisited, 1 = in-stack, 2 = done.
    let mut state: std::collections::HashMap<&str, u8> =
        declared.iter().map(|&n| (n, 0u8)).collect();
    let mut reported_cycles: std::collections::HashSet<String> = std::collections::HashSet::new();

    for start in declared.iter() {
        if state[start] != 0 {
            continue;
        }
        // Iterative DFS with explicit stack: (node, iterator-over-children).
        let mut stack: Vec<(&str, usize)> = vec![(start, 0)];
        // path_set tracks nodes currently on the DFS stack for cycle detection.
        let mut path: Vec<&str> = vec![start];
        let mut path_set: std::collections::HashSet<&str> =
            std::collections::HashSet::from([*start]);
        state.insert(start, 1);

        while let Some((node, child_idx)) = stack.last_mut() {
            let node = *node;
            let children = adj.get(node).map(Vec::as_slice).unwrap_or(&[]);
            if *child_idx < children.len() {
                let child = children[*child_idx];
                *child_idx += 1;
                match state.get(child).copied().unwrap_or(0) {
                    1 => {
                        // Back edge → cycle. Find the cycle start in `path`.
                        if let Some(pos) = path.iter().position(|&n| n == child) {
                            let cycle: Vec<&str> = path[pos..].to_vec();
                            // Use the lexicographically smallest node as the canonical key.
                            let mut key = cycle.clone();
                            key.sort_unstable();
                            let key_str = key.join(",");
                            if reported_cycles.insert(key_str) {
                                let cycle_str = cycle.join(" → ");
                                report.drifts.push(DriftItem {
                                    kind: DriftKind::CircularDependency,
                                    severity: Severity::Error,
                                    service: child.to_string(),
                                    message: format!(
                                        "circular dependency detected: {} → {}",
                                        cycle_str, child
                                    ),
                                    detail: Some(format!("{} → {}", cycle_str, child)),
                                });
                            }
                        }
                    }
                    0 => {
                        state.insert(child, 1);
                        path.push(child);
                        path_set.insert(child);
                        stack.push((child, 0));
                    }
                    _ => {} // already fully visited
                }
            } else {
                // All children visited; pop.
                state.insert(node, 2);
                stack.pop();
                path.pop();
                path_set.remove(node);
            }
        }
    }
}
