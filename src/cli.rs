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
    },

    /// Generate a Mermaid or Markdown view of the service catalog
    Graph {
        /// Path to the manifest file (auto-detected if omitted)
        #[arg(short, long)]
        manifest: Option<PathBuf>,

        /// Output format
        #[arg(short, long, value_enum, default_value_t = GraphFormat::Mermaid)]
        format: GraphFormat,
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
    Json,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum GraphFormat {
    Mermaid,
    Markdown,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum ExportFormat {
    Json,
    Markdown,
}
