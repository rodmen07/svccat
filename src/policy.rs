use crate::manifest::{Manifest, ServiceEntry};
use anyhow::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::Path;

// ── Config ─────────────────────────────────────────────────────────────────────

/// Policy definition loaded from `.svccat/policy.yaml` or `svccat.policy.yaml`.
///
/// Example file:
/// ```yaml
/// required:
///   - team
///   - oncall
/// recommended:
///   - language
///   - platform
///   - docs
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyConfig {
    /// Fields every service MUST declare.  Violations are errors.
    #[serde(default)]
    pub required: Vec<String>,

    /// Fields every service SHOULD declare.  Violations are warnings.
    #[serde(default)]
    pub recommended: Vec<String>,
}

impl PolicyConfig {
    /// Try to load a policy file from the repo root.
    /// Returns `None` when no policy file exists (not an error).
    pub fn load(root: &Path) -> Option<Self> {
        let candidates = [
            root.join(".svccat").join("policy.yaml"),
            root.join(".svccat").join("policy.yml"),
            root.join("svccat.policy.yaml"),
            root.join("svccat.policy.yml"),
        ];
        for path in &candidates {
            if path.exists() {
                if let Ok(text) = std::fs::read_to_string(path) {
                    if let Ok(cfg) = serde_yaml::from_str::<PolicyConfig>(&text) {
                        return Some(cfg);
                    }
                }
            }
        }
        None
    }

    pub fn is_empty(&self) -> bool {
        self.required.is_empty() && self.recommended.is_empty()
    }
}

// ── Report ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicySeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize)]
pub struct PolicyViolation {
    pub service: String,
    pub field: String,
    pub severity: PolicySeverity,
    pub message: String,
}

pub struct PolicyReport {
    pub violations: Vec<PolicyViolation>,
    pub services_checked: usize,
}

impl PolicyReport {
    pub fn error_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|v| matches!(v.severity, PolicySeverity::Error))
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.violations
            .iter()
            .filter(|v| matches!(v.severity, PolicySeverity::Warning))
            .count()
    }

    pub fn passed(&self) -> bool {
        self.error_count() == 0
    }
}

// ── Engine ─────────────────────────────────────────────────────────────────────

/// Run policy checks against every service in the manifest.
pub fn check(manifest: &Manifest, config: &PolicyConfig) -> PolicyReport {
    let mut violations = Vec::new();

    for svc in &manifest.services {
        for field in &config.required {
            if !has_field(svc, field) {
                violations.push(PolicyViolation {
                    service: svc.name.clone(),
                    field: field.clone(),
                    severity: PolicySeverity::Error,
                    message: format!(
                        "service '{}' is missing required field '{}'",
                        svc.name, field
                    ),
                });
            }
        }
        for field in &config.recommended {
            if !has_field(svc, field) {
                violations.push(PolicyViolation {
                    service: svc.name.clone(),
                    field: field.clone(),
                    severity: PolicySeverity::Warning,
                    message: format!(
                        "service '{}' is missing recommended field '{}'",
                        svc.name, field
                    ),
                });
            }
        }
    }

    PolicyReport {
        violations,
        services_checked: manifest.services.len(),
    }
}

fn has_field(svc: &ServiceEntry, field: &str) -> bool {
    match field {
        "name" => !svc.name.is_empty(),
        "language" => svc.language.is_some(),
        "platform" => svc.platform.is_some(),
        "role" => svc.role.is_some(),
        "url" => svc.url.is_some(),
        "team" => svc.team.is_some(),
        "oncall" => svc.oncall.is_some(),
        "docs" => svc.docs.is_some(),
        "ci" => svc.ci.is_some(),
        _ => false,
    }
}

// ── Renderers ──────────────────────────────────────────────────────────────────

pub fn render_terminal(report: &PolicyReport, config: &PolicyConfig) {
    let errors = report.error_count();
    let warnings = report.warning_count();

    println!("{}", "svccat policy check".bold());
    println!();

    if !config.required.is_empty() {
        println!("  {}  {}", "Required:".bold(), config.required.join(", "));
    }
    if !config.recommended.is_empty() {
        println!(
            "  {}  {}",
            "Recommended:".bold(),
            config.recommended.join(", ")
        );
    }
    println!("  {}  {}", "Services:".bold(), report.services_checked);
    println!();

    if report.violations.is_empty() {
        println!(
            "  {} All {} service{} comply with policy",
            "✓".green().bold(),
            report.services_checked,
            if report.services_checked == 1 {
                ""
            } else {
                "s"
            }
        );
        return;
    }

    for v in &report.violations {
        match v.severity {
            PolicySeverity::Error => {
                println!("  {}  {}", "✗".red().bold(), v.message.red())
            }
            PolicySeverity::Warning => {
                println!("  {}  {}", "⚠".yellow(), v.message.yellow())
            }
        }
    }
    println!();
    println!(
        "  {} error{}, {} warning{}",
        errors,
        plural(errors),
        warnings,
        plural(warnings)
    );
}

pub fn render_json(report: &PolicyReport) -> Result<()> {
    let json = serde_json::to_string_pretty(&report.violations)?;
    println!("{json}");
    Ok(())
}

fn plural(n: usize) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}
