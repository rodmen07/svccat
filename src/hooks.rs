use crate::{discovery, drift, manifest};
use anyhow::{anyhow, Result};
use colored::Colorize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct HookConfig {
    pub fail_on_drift: bool,
    pub check_uncommitted: bool,
    pub manifest_path: PathBuf,
    pub root: PathBuf,
    pub ignore: Vec<String>,
    pub depth: u32,
}

impl Default for HookConfig {
    fn default() -> Self {
        HookConfig {
            fail_on_drift: true,
            check_uncommitted: true,
            manifest_path: PathBuf::from("services.yaml"),
            root: PathBuf::from("."),
            ignore: vec![],
            depth: 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HookCheckResult {
    pub passed: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub messages: Vec<String>,
    pub block_commit: bool,
}

impl HookCheckResult {
    pub fn success() -> Self {
        HookCheckResult {
            passed: true,
            error_count: 0,
            warning_count: 0,
            messages: vec!["✓ Manifest is in sync - ready to commit".to_string()],
            block_commit: false,
        }
    }

    pub fn failure(error_count: usize, warning_count: usize, messages: Vec<String>) -> Self {
        HookCheckResult {
            passed: false,
            error_count,
            warning_count,
            messages,
            block_commit: true,
        }
    }
}

pub fn run_pre_commit_check(config: &HookConfig) -> Result<HookCheckResult> {
    if !config.manifest_path.exists() {
        return Ok(HookCheckResult::failure(
            1,
            0,
            vec![
                format!(
                    "✗ Manifest file not found: {}",
                    config.manifest_path.display()
                ),
                "Create services.yaml with service declarations before committing".to_string(),
            ],
        ));
    }

    let manifest = manifest::Manifest::load(&config.manifest_path)?;
    let discovered = discovery::discover_services_with_opts(
        &config.root,
        &manifest,
        &config.ignore,
        config.depth,
    );
    let report = drift::analyze(&manifest, &discovered, &config.root);

    let error_count = report.error_count();

    if error_count == 0 {
        Ok(HookCheckResult::success())
    } else {
        let mut messages = vec![format!("✗ {} drift error(s) detected", error_count)];

        for drift_item in &report.drifts {
            if drift_item.severity == crate::drift::Severity::Error {
                messages.push(format!(
                    "  - {}: {}",
                    drift_item.service, drift_item.message
                ));
            }
        }

        messages.push("\nRun 'svccat check' to review and fix drift before committing".to_string());

        if config.fail_on_drift {
            messages.push("\nTo bypass this check: git commit --no-verify".to_string());
        }

        Ok(HookCheckResult::failure(error_count, 0, messages))
    }
}

pub fn install(repo_root: &Path, hook_type: &str, _fail_on_drift: bool) -> Result<()> {
    let git_dir = repo_root.join(".git");
    if !git_dir.exists() {
        return Err(anyhow!(
            "Not a git repository: {} not found",
            git_dir.display()
        ));
    }

    let hooks_dir = git_dir.join("hooks");
    fs::create_dir_all(&hooks_dir)?;

    match hook_type {
        "pre-commit" => {
            let path = hooks_dir.join("pre-commit");
            let script = "#!/bin/sh\nsvccat check > /dev/null 2>&1 || exit 1";
            fs::write(&path, script)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
            }
            println!("{}  Pre-commit hook installed", "✓".green());
        }
        "pre-push" => {
            let path = hooks_dir.join("pre-push");
            let script = "#!/bin/sh\nsvccat check > /dev/null 2>&1 || exit 1";
            fs::write(&path, script)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
            }
            println!("{}  Pre-push hook installed", "✓".green());
        }
        _ => return Err(anyhow!("Unknown hook type: {}", hook_type)),
    }

    Ok(())
}

pub fn install_pre_commit_hook(repo_root: &Path) -> Result<()> {
    install(repo_root, "pre-commit", true)
}

pub fn uninstall_pre_commit_hook(repo_root: &Path) -> Result<()> {
    let pre_commit_path = repo_root.join(".git/hooks/pre-commit");
    if !pre_commit_path.exists() {
        return Err(anyhow!("Pre-commit hook not found"));
    }
    fs::remove_file(&pre_commit_path)?;
    println!("{}  Pre-commit hook removed", "✓".green());
    Ok(())
}

pub fn is_hook_installed(repo_root: &Path) -> bool {
    repo_root.join(".git/hooks/pre-commit").exists()
}

#[derive(Debug, Clone)]
pub struct GitStatus {
    pub staged_changes: Vec<String>,
    pub unstaged_changes: Vec<String>,
}

impl GitStatus {
    pub fn manifest_changed(&self, manifest_path: &Path) -> bool {
        let filename = manifest_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("services.yaml");
        self.staged_changes.iter().any(|f| f.ends_with(filename))
            || self.unstaged_changes.iter().any(|f| f.ends_with(filename))
    }

    pub fn has_changes(&self) -> bool {
        !self.staged_changes.is_empty() || !self.unstaged_changes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_config_default() {
        let config = HookConfig::default();
        assert!(config.fail_on_drift);
        assert_eq!(config.depth, 3);
    }

    #[test]
    fn test_hook_check_result_success() {
        let result = HookCheckResult::success();
        assert!(result.passed);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_hook_check_result_failure() {
        let result = HookCheckResult::failure(2, 1, vec!["Error".to_string()]);
        assert!(!result.passed);
        assert!(result.block_commit);
        assert_eq!(result.error_count, 2);
    }

    #[test]
    fn test_git_status_manifest_changed() {
        let status = GitStatus {
            staged_changes: vec!["services.yaml".to_string()],
            unstaged_changes: vec![],
        };
        assert!(status.manifest_changed(Path::new("services.yaml")));
    }

    #[test]
    fn test_git_status_has_changes() {
        let status = GitStatus {
            staged_changes: vec!["file.txt".to_string()],
            unstaged_changes: vec![],
        };
        assert!(status.has_changes());
    }
}
