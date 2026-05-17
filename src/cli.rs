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

        /// Write output to this file instead of stdout
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
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

        /// Only include services whose name contains this substring (case-insensitive)
        #[arg(long, value_name = "PATTERN")]
        filter: Option<String>,

        /// Write output to this file instead of stdout
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
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

        /// Only export services that changed since this git ref (new or modified fields)
        ///
        /// Example: --since HEAD~1   or   --since main
        #[arg(long, value_name = "GIT_REF")]
        since: Option<String>,
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

        /// Output format
        #[arg(short, long, value_enum, default_value_t = DiffFormat::Terminal)]
        format: DiffFormat,
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

        /// Send a native OS desktop notification when the drift count changes
        #[arg(long)]
        notify: bool,

        /// Also re-run every N seconds regardless of file-system events
        ///
        /// Useful in Docker containers, network mounts, or other environments
        /// where inotify is unreliable.
        #[arg(long, value_name = "N")]
        interval: Option<u64>,
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

    /// Run a full audit: lint + drift + optional URL ping, with a health score
    ///
    /// Combines `svccat lint`, `svccat check`, and optionally `svccat check --ping`
    /// into a single pass, emitting a scored summary.  Exits with code 1 when
    /// drift errors or lint errors are present.
    Audit {
        /// Path to the manifest file (auto-detected if omitted)
        #[arg(short, long)]
        manifest: Option<PathBuf>,

        /// Ping each service URL and include reachability in the score
        #[arg(long)]
        ping: bool,

        /// Glob patterns to exclude from discovery (repeatable)
        #[arg(long, value_name = "PATTERN")]
        ignore: Vec<String>,

        /// Maximum directory depth to scan for services (default: 1)
        #[arg(long, value_name = "N", default_value_t = 1)]
        depth: u32,

        /// Output format
        #[arg(short, long, value_enum, default_value_t = AuditFormat::Terminal)]
        format: AuditFormat,
    },

    /// Enforce metadata policies defined in .svccat/policy.yaml
    ///
    /// Policies declare which fields are `required` (errors) or `recommended`
    /// (warnings) for every service in the manifest.
    ///
    /// Example policy file (.svccat/policy.yaml):
    ///   required:
    ///     - team
    ///     - oncall
    ///   recommended:
    ///     - language
    ///     - docs
    Policy {
        /// Path to the manifest file (auto-detected if omitted)
        #[arg(short, long)]
        manifest: Option<PathBuf>,

        /// Output format
        #[arg(short, long, value_enum, default_value_t = PolicyFormat::Terminal)]
        format: PolicyFormat,

        /// Exit with code 1 when policy errors are present
        #[arg(long)]
        fail_on_violations: bool,
    },

    /// Manage named snapshots of the service catalog
    ///
    /// Snapshots are saved to .svccat/snapshots/ and can be compared with
    /// `svccat diff` by using the snapshot file path.
    ///
    /// Commands:
    ///   save <name>    - Save the current catalog as a named snapshot
    ///   list           - List all saved snapshots
    ///   delete <name>  - Remove a snapshot
    Snapshot {
        #[command(subcommand)]
        action: SnapshotAction,
    },

    /// Run lint + drift + policy in a single CI-friendly pass
    ///
    /// Combines `svccat lint`, `svccat check`, and `svccat policy` (when a
    /// policy file exists) into one command.  Exits with code 1 on any errors.
    Ci {
        /// Path to the manifest file (auto-detected if omitted)
        #[arg(short, long)]
        manifest: Option<PathBuf>,

        /// Glob patterns to exclude from discovery (repeatable)
        #[arg(long, value_name = "PATTERN")]
        ignore: Vec<String>,

        /// Maximum directory depth to scan for services (default: 1)
        #[arg(long, value_name = "N", default_value_t = 1)]
        depth: u32,

        /// Output format
        #[arg(short, long, value_enum, default_value_t = CiFormat::Terminal)]
        format: CiFormat,

        /// Watch the manifest and service directories; re-run on every change
        #[arg(long)]
        watch: bool,

        /// Also re-run every N seconds regardless of file-system events
        #[arg(long, value_name = "N")]
        interval: Option<u64>,
    },

    /// Search the service catalog
    ///
    /// Supports `field:value` syntax (e.g. `team:payments`) and plain
    /// substring matching against all fields.
    ///
    /// Searchable fields: name, language, platform, url, role, team, oncall,
    /// docs, ci, path, tags, depends_on.
    Search {
        /// Search query, e.g. `team:platform` or `auth`
        query: String,

        /// Path to the manifest file (auto-detected if omitted)
        #[arg(short, long)]
        manifest: Option<PathBuf>,

        /// Write output to this file instead of stdout
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// Analyse inter-service dependencies declared in the manifest
    ///
    /// Detects undeclared dependency targets (services referenced in
    /// `depends_on` that are not in the manifest) and circular dependency
    /// chains.  Exits with code 1 when errors are found.
    Deps {
        /// Path to the manifest file (auto-detected if omitted)
        #[arg(short, long)]
        manifest: Option<PathBuf>,

        /// Output format
        #[arg(short, long, value_enum, default_value_t = DepsFormat::Terminal)]
        format: DepsFormat,

        /// Write output to this file instead of stdout
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// Add or remove tags on a service entry in the manifest
    ///
    /// Modifies the manifest file in-place.
    Tag {
        #[command(subcommand)]
        action: TagAction,
    },

    /// Score every service on completeness, drift health, and policy compliance
    ///
    /// Each service receives three dimension scores (0-100):
    ///   completeness - how many optional metadata fields are populated
    ///   drift        - 100 when clean, reduced by drift errors
    ///   policy       - 100 when no policy errors are present
    ///
    /// A composite total is computed as: completeness*40% + drift*40% + policy*20%.
    Scorecard {
        /// Path to the manifest file (auto-detected if omitted)
        #[arg(short, long)]
        manifest: Option<PathBuf>,

        /// Glob patterns to exclude from discovery (repeatable)
        #[arg(long, value_name = "PATTERN")]
        ignore: Vec<String>,

        /// Maximum directory depth to scan for services (default: 1)
        #[arg(long, value_name = "N", default_value_t = 1)]
        depth: u32,

        /// Output format
        #[arg(short, long, value_enum, default_value_t = ScorecardFormat::Terminal)]
        format: ScorecardFormat,

        /// Write output to this file instead of stdout
        #[arg(short, long, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// Fire a webhook manually or configure automatic webhook notifications
    ///
    /// When invoked without a subcommand, runs the current drift check and fires
    /// the configured webhook if thresholds are met.
    ///
    /// Configure a webhook URL in `svccat.toml`:
    ///   [webhook]
    ///   url = "https://hooks.slack.com/services/..."
    ///   on_errors = true
    ///   on_warnings = false
    Webhook {
        /// Path to the manifest file (auto-detected if omitted)
        #[arg(short, long)]
        manifest: Option<PathBuf>,

        /// Glob patterns to exclude from discovery (repeatable)
        #[arg(long, value_name = "PATTERN")]
        ignore: Vec<String>,

        /// Maximum directory depth to scan for services (default: 1)
        #[arg(long, value_name = "N", default_value_t = 1)]
        depth: u32,

        /// Override the webhook URL from svccat.toml
        #[arg(long, value_name = "URL")]
        url: Option<String>,
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
    /// Microsoft Teams Adaptive Card JSON payload
    Teams,
    /// Datadog Events API JSON payload
    Datadog,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum GraphFormat {
    Mermaid,
    Markdown,
    /// Graphviz DOT digraph (pipe to `dot -Tsvg` to render)
    Dot,
    /// PlantUML component diagram (paste at plantuml.com or pipe to plantuml)
    Plantuml,
    /// Interactive self-contained HTML file with a D3.js force-directed graph
    Html,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum ReportFormat {
    Markdown,
    Html,
    /// Machine-readable JSON: manifest summary + per-team service breakdown
    Json,
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

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum DiffFormat {
    /// Coloured terminal output (default)
    Terminal,
    /// GitHub-flavoured Markdown table
    Markdown,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum AuditFormat {
    /// Coloured terminal output with bar chart (default)
    Terminal,
    /// Machine-readable JSON object
    Json,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum PolicyFormat {
    /// Coloured terminal output (default)
    Terminal,
    /// Machine-readable JSON array of violations
    Json,
}

#[derive(Subcommand, Debug)]
pub enum SnapshotAction {
    /// Save the current catalog state as a named snapshot
    Save {
        /// Name for this snapshot (e.g. v1.0, pre-migration)
        name: String,

        /// Path to the manifest file (auto-detected if omitted)
        #[arg(short, long)]
        manifest: Option<PathBuf>,

        /// Glob patterns to exclude from discovery (repeatable)
        #[arg(long, value_name = "PATTERN")]
        ignore: Vec<String>,

        /// Maximum directory depth to scan for services (default: 1)
        #[arg(long, value_name = "N", default_value_t = 1)]
        depth: u32,
    },

    /// List all saved snapshots
    List,

    /// Delete a named snapshot
    Delete {
        /// Name of the snapshot to delete
        name: String,
    },

    /// Compare two named snapshots against each other
    Compare {
        /// Name of the older ("before") snapshot
        before: String,
        /// Name of the newer ("after") snapshot
        after: String,
        /// Output format
        #[arg(short, long, value_enum, default_value_t = DiffFormat::Terminal)]
        format: DiffFormat,
    },

    /// Compare a named snapshot against the current catalog state
    Diff {
        /// Name of the snapshot to compare against
        name: String,

        /// Glob patterns to exclude from discovery (repeatable)
        #[arg(long, value_name = "PATTERN")]
        ignore: Vec<String>,

        /// Maximum directory depth to scan for services (default: 1)
        #[arg(long, value_name = "N", default_value_t = 1)]
        depth: u32,

        /// Output format
        #[arg(short, long, value_enum, default_value_t = DiffFormat::Terminal)]
        format: DiffFormat,
    },
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum CiFormat {
    /// Coloured terminal output (default)
    Terminal,
    /// Machine-readable JSON
    Json,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum DepsFormat {
    /// Coloured terminal output (default)
    Terminal,
    /// Mermaid flowchart
    Mermaid,
    /// Machine-readable JSON
    Json,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum ScorecardFormat {
    /// Coloured table output (default)
    Terminal,
    /// Machine-readable JSON
    Json,
    /// GitHub-flavoured Markdown table
    Markdown,
}

#[derive(Subcommand, Debug)]
pub enum TagAction {
    /// Add a tag to a service
    Add {
        /// Service name
        service: String,
        /// Tag to add
        tag: String,
        /// Path to the manifest file (auto-detected if omitted)
        #[arg(short, long)]
        manifest: Option<PathBuf>,
    },
    /// Remove a tag from a service
    Remove {
        /// Service name
        service: String,
        /// Tag to remove
        tag: String,
        /// Path to the manifest file (auto-detected if omitted)
        #[arg(short, long)]
        manifest: Option<PathBuf>,
    },
}
