use anyhow::{bail, Context, Result};
use colored::Colorize;
use std::path::Path;

/// Write a git hook script that runs `svccat check`.
///
/// `hook_name` should be `"pre-commit"` or `"pre-push"`.
/// Fails if the hook file already exists (user must remove it manually).
pub fn install(root: &Path, hook_name: &str, fail_on_drift: bool) -> Result<()> {
    let git_dir = root.join(".git");
    if !git_dir.is_dir() {
        bail!(
            "no .git directory found at {} - is this a git repository?",
            root.display()
        );
    }

    let hooks_dir = git_dir.join("hooks");
    std::fs::create_dir_all(&hooks_dir).context("failed to create .git/hooks directory")?;

    let hook_path = hooks_dir.join(hook_name);
    if hook_path.exists() {
        bail!(
            "{} already exists. Remove or rename it before running install-hooks.",
            hook_path.display()
        );
    }

    let drift_flag = if fail_on_drift {
        " --fail-on-drift"
    } else {
        ""
    };
    let script = format!(
        "#!/usr/bin/env sh\n\
         # Installed by `svccat install-hooks`\n\
         # Remove this file to disable the hook.\n\
         \n\
         echo 'svccat: checking for catalog drift...'\n\
         svccat check{drift_flag}\n"
    );

    std::fs::write(&hook_path, &script)
        .with_context(|| format!("failed to write {}", hook_path.display()))?;

    // On Unix the hook file must be executable.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&hook_path, perms)?;
    }

    println!(
        "  {}  installed {} hook at {}",
        "✓".green().bold(),
        hook_name.bold(),
        hook_path.display()
    );
    Ok(())
}
