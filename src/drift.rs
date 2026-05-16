use crate::discovery::DiscoveredService;
use crate::manifest::Manifest;
use serde::{Deserialize, Serialize};
use std::path::Path;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftItem {
    pub kind: DriftKind,
    pub severity: Severity,
    pub service: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
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
    let declared_paths: Vec<&str> = manifest
        .services
        .iter()
        .filter_map(|s| s.declared_path())
        .collect();

    let declared_names: Vec<&str> = manifest.services.iter().map(|s| s.name.as_str()).collect();

    for disc in discovered {
        let matched_by_path = declared_paths.iter().any(|p| *p == disc.path);
        let matched_by_name = declared_names.contains(&disc.name.as_str());

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

    report
}
