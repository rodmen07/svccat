use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "svccat",
    version,
    about = "Service catalog drift detection for multi-service repositories",
    long_about = "\
svccat reads your declared service manifest and compares it against what \
actually exists in the repo, flagging drift before it becomes toil.\n\n\
Typical workflow:\n  \
  svccat check                   # inspect drift in the current repo\n  \
  svccat check --fail-on-drift   # gate CI on zero drift\n  \
  svccat graph                   # emit a Mermaid diagram for docs\n  \
  svccat export --format json    # machine-readable catalog snapshot"
)]
pub struct Cli {
    /// Repository root (defaults to current directory)
    #[arg(short, long, global = true)]
    pub root: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Check declared services against the repo and report drift
    Check {
        /// Path to the manifest file (auto-detected if omitted)
        #[arg(short, long)]
        manifest: Option<PathBuf>,

        /// Output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Terminal)]
        format: OutputFormat,

        /// Exit with code 1 when drift is detected (useful in CI)
        #[arg(long)]
        fail_on_drift: bool,

        /// Ping each service URL and report reachability
        #[arg(long)]
        ping: bool,

        /// Glob patterns to exclude from discovery (repeatable, e.g. --ignore "examples/*")
        #[arg(long, value_name = "PATTERN")]
        ignore: Vec<String>,

        /// Only check services owned by this team (matches the `team:` field)
        #[arg(long, value_name = "TEAM")]
        team: Option<String>,

        /// Compare against the manifest at this git ref; show only new / resolved drift
        ///
        /// Example: --since HEAD~1   or   --since main
        #[arg(long, value_name = "GIT_REF")]
        since: Option<String>,

        /// Exit with code 1 when --since reveals new drift items (ignores pre-existing drift)
        #[arg(long, requires = "since")]
        fail_on_new_drift: bool,

        /// Maximum directory depth to scan for services (default: 1 - direct children of each
        /// discovery path). Use 2 to also detect services nested one level deeper, e.g.
        /// services/team/auth-service.
        #[arg(long, value_name = "N", default_value_t = 1)]
        depth: u32,

        /// Only report drift absent from this saved baseline snapshot (JSON from svccat export).
        ///
        /// Generate a baseline with: svccat export --format json > baseline.json
        /// Then gate on regressions only: svccat check --baseline baseline.json --fail-on-drift
        #[arg(long, value_name = "FILE")]
        baseline: Option<PathBuf>,
    },

    /// Generate a Mermaid or Markdown view of the service catalog
    Graph {
        /// Path to the manifest file (auto-detected if omitted)
        #[arg(short, long)]
        manifest: Option<PathBuf>,

        /// Output format
        #[arg(short, long, value_enum, default_value_t = GraphFormat::Mermaid)]
        format: GraphFormat,

        /// Only include services owned by this team; cross-team depends_on targets
        /// are shown as external nodes
        #[arg(long, value_name = "TEAM")]
        team: Option<String>,
    },

    /// Export the full service catalog with drift summary
    Export {
        /// Path to the manifest file (auto-detected if omitted)
        #[arg(short, long)]
        manifest: Option<PathBuf>,

        /// Export format
        #[arg(short, long, value_enum, default_value_t = ExportFormat::Json)]
        format: ExportFormat,

        /// Glob patterns to exclude from discovery (repeatable)
        #[arg(long, value_name = "PATTERN")]
        ignore: Vec<String>,

        /// Maximum directory depth to scan for services (default: 1)
        #[arg(long, value_name = "N", default_value_t = 1)]
        depth: u32,
    },

    /// Scaffold a services.yaml by auto-discovering services in the repo
    Init {
        /// Where to write the manifest (default: services.yaml in repo root)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Overwrite an existing manifest file
        #[arg(long)]
        force: bool,
    },

    /// Compare two svccat export JSON snapshots and show what changed
    ///
    /// Generate snapshots with: svccat export --format json > snapshot.json
    Diff {
        /// Path to the older snapshot (JSON)
        before: std::path::PathBuf,

        /// Path to the newer snapshot (JSON)
        after: std::path::PathBuf,
    },

    /// Watch the manifest and service directories for changes and re-run drift checks
    ///
    /// Continuously monitors the manifest file and service directories.
    /// Re-runs drift analysis whenever a file change is detected and prints
    /// a timestamped report. Press Ctrl-C to stop.
    Watch {
        /// Path to the manifest file (auto-detected if omitted)
        #[arg(short, long)]
        manifest: Option<PathBuf>,

        /// Exit with code 1 when drift is detected on the first check
        #[arg(long)]
        fail_on_drift: bool,

        /// Only watch services owned by this team
        #[arg(long, value_name = "TEAM")]
        team: Option<String>,

        /// Glob patterns to exclude from discovery (repeatable)
        #[arg(long, value_name = "PATTERN")]
        ignore: Vec<String>,

        /// Maximum directory depth to scan for services (default: 1)
        #[arg(long, value_name = "N", default_value_t = 1)]
        depth: u32,

        /// On each check, compare against the manifest at this git ref and show only new drift
        ///
        /// Example: --since HEAD~1   or   --since main
        #[arg(long, value_name = "GIT_REF")]
        since: Option<String>,
    },

    /// Generate a full ownership and drift report
    ///
    /// Outputs a per-team breakdown of every service with language, platform, oncall,
    /// and drift status, plus a dependency summary and full drift details.
    Report {
        /// Path to the manifest file (auto-detected if omitted)
        #[arg(short, long)]
        manifest: Option<PathBuf>,

        /// Output format
        #[arg(short, long, value_enum, default_value_t = ReportFormat::Markdown)]
        format: ReportFormat,

        /// Write output to this file instead of stdout
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,

        /// Glob patterns to exclude from discovery (repeatable)
        #[arg(long, value_name = "PATTERN")]
        ignore: Vec<String>,

        /// Show drift evolution across the last N git commits
        ///
        /// Requires a git repository. Loads the manifest at each of the last N commits
        /// and emits a Markdown table of errors/warnings over time.
        #[arg(long, value_name = "N")]
        history: Option<usize>,

        /// Emit a Shields.io badge Markdown snippet reflecting current drift status.
        ///
        /// Prints a Markdown image link you can paste into your README:
        ///   [![svccat drift: clean](…)](…)
        #[arg(long)]
        badge: bool,
    },

    /// Validate the manifest for structural issues
    ///
    /// Checks for duplicate service names, blank names, self-referential
    /// depends_on entries, duplicate depends_on entries, and unknown manifest versions.
    Lint {
        /// Path to the manifest file (auto-detected if omitted)
        #[arg(short, long)]
        manifest: Option<PathBuf>,
    },

    /// Import services from an external catalog format and merge into services.yaml
    ///
    /// Supported sources:
    ///   backstage   Walk the repo for catalog-info.yaml files (Backstage format)
    ///               and generate service entries from every Kind: Component found.
    ///   docker-compose  Parse services from docker-compose.yml / compose.yaml
    ///   openapi     Walk the repo for openapi.yaml / swagger.yaml spec files
    Import {
        /// Source catalog format to import from
        #[arg(value_enum)]
        from: ImportSource,

        /// Where to write the merged manifest (default: services.yaml in repo root)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Overwrite an existing manifest instead of merging
        #[arg(long)]
        force: bool,
    },

    /// Auto-remediate simple drift in the manifest
    ///
    /// Adds undeclared services found in the repo and, with --prune, removes
    /// declared services whose directories are no longer present.
    Fix {
        /// Path to the manifest file (auto-detected if omitted)
        #[arg(short, long)]
        manifest: Option<PathBuf>,

        /// Also remove declared services whose directories are missing from the repo
        #[arg(long)]
        prune: bool,

        /// Preview changes without writing anything
        #[arg(long)]
        dry_run: bool,

        /// Glob patterns to exclude from discovery (repeatable)
        #[arg(long, value_name = "PATTERN")]
        ignore: Vec<String>,

        /// Maximum directory depth to scan for services (default: 1)
        #[arg(long, value_name = "N", default_value_t = 1)]
        depth: u32,
    },

    /// Install a git hook that runs svccat check on commit or push
    ///
    /// Writes a shell script to .git/hooks/<hook> that runs `svccat check`.
    /// The hook file must not already exist.
    InstallHooks {
        /// Which git hook to install
        #[arg(long, value_enum, default_value_t = HookKind::PreCommit)]
        hook: HookKind,

        /// Pass --fail-on-drift to svccat check in the hook script
        #[arg(long, default_value_t = true)]
        fail_on_drift: bool,
    },

    /// Show field-coverage statistics for the service manifest
    ///
    /// Displays how many services have each metadata field set, with a
    /// percentage and ASCII bar chart per field, plus an overall health score.
    Stats {
        /// Path to the manifest file (auto-detected if omitted)
        #[arg(short, long)]
        manifest: Option<PathBuf>,
    },

    /// Serve a live HTML drift report over HTTP
    ///
    /// Starts a local HTTP server that renders the drift report as HTML on
    /// every request, so you always see up-to-date information.
    /// Browse to http://localhost:<port> (default 7777).
    Serve {
        /// Path to the manifest file (auto-detected if omitted)
        #[arg(short, long)]
        manifest: Option<PathBuf>,

        /// TCP port to listen on (default: 7777)
        #[arg(short, long, default_value_t = 7777)]
        port: u16,

        /// Auto-refresh the browser page every N seconds (0 = disabled)
        #[arg(long, value_name = "N", default_value_t = 0)]
        refresh: u32,
    },

    /// Print shell completion script to stdout
    ///
    /// Source the output to enable tab completion, e.g.:
    ///   source <(svccat completions bash)
    ///   svccat completions zsh > ~/.zfunc/_svccat
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum OutputFormat {
    Terminal,
    /// One line per service: status icon, name, and drift summary
    Compact,
    Json,
    Sarif,
    Markdown,
    Junit,
    GithubAnnotation,
    /// Comma-separated values: service, severity, kind, message, detail
    Csv,
    /// Slack Block Kit JSON payload for posting to a channel or webhook
    Slack,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum GraphFormat {
    Mermaid,
    Markdown,
    /// Graphviz DOT digraph (pipe to `dot -Tsvg` to render)
    Dot,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum ReportFormat {
    Markdown,
    Html,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum ExportFormat {
    Json,
    Markdown,
    /// Comma-separated values: name, language, platform, role, url, team, oncall
    Csv,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum ImportSource {
    /// Backstage catalog-info.yaml files (Kind: Component)
    Backstage,
    /// Docker Compose services (docker-compose.yml or compose.yaml)
    DockerCompose,
    /// OpenAPI / Swagger spec files (openapi.yaml, swagger.yaml)
    Openapi,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum HookKind {
    /// Run on every git commit (pre-commit hook)
    PreCommit,
    /// Run before git push (pre-push hook)
    PrePush,
}
