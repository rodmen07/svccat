use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

/// Workspace-level defaults loaded from `svccat.toml` in the repo root.
/// All fields are optional; CLI flags always take precedence.
///
/// Example `svccat.toml`:
/// ```toml
/// format = "terminal"
/// fail_on_drift = true
/// ignore = ["examples/*", "vendor/*", "test-fixtures/*"]
/// ```
#[derive(Debug, Default, Deserialize)]
pub struct SvccatConfig {
    /// Default output format for `svccat check` ("terminal" or "json").
    pub format: Option<String>,

    /// Whether to exit 1 on drift by default (equivalent to --fail-on-drift).
    #[serde(default)]
    pub fail_on_drift: bool,

    /// Discovery ignore patterns applied to every command that runs discovery.
    #[serde(default)]
    pub ignore: Vec<String>,
}

impl SvccatConfig {
    /// Load `svccat.toml` from `root`. Returns a default config if the file
    /// doesn't exist; returns an error if the file exists but can't be parsed.
    pub fn load(root: &Path) -> Result<Self> {
        let path = root.join("svccat.toml");
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("cannot read svccat.toml: {e}"))?;
        let cfg: Self =
            toml::from_str(&text).map_err(|e| anyhow::anyhow!("cannot parse svccat.toml: {e}"))?;
        Ok(cfg)
    }
}
