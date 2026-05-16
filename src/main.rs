use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use std::io;
use std::process;
use svccat::cli::{Cli, Commands, ExportFormat, GraphFormat, OutputFormat};
use svccat::{config, diff, discovery, drift, init, manifest, output, ping};

fn main() {
    match run() {
        Ok(code) => process::exit(code),
        Err(e) => {
            eprintln!("error: {e:#}");
            process::exit(2);
        }
    }
}

fn run() -> Result<i32> {
    let cli = Cli::parse();
    let root = cli.root.unwrap_or_else(|| std::path::PathBuf::from("."));

    // Load workspace config (svccat.toml), falling back to defaults.
    let cfg = config::SvccatConfig::load(&root)?;

    match cli.command {
        Commands::Check {
            manifest: manifest_path,
            format,
            fail_on_drift,
            ping: do_ping,
            ignore: cli_ignore,
        } => {
            let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
            let m = manifest::Manifest::load(&path)?;

            // Merge config ignore + CLI ignore patterns.
            let mut ignore: Vec<String> = cfg.ignore.clone();
            ignore.extend(cli_ignore);

            let discovered = discovery::discover_services_with_ignore(&root, &m, &ignore);
            let mut report = drift::analyze(&m, &discovered, &root);
            report.manifest = path.display().to_string();

            let ping_results = if do_ping {
                ping::ping_services(&m)
            } else {
                vec![]
            };

            match format {
                OutputFormat::Terminal => output::terminal::render_check(&report, &ping_results),
                OutputFormat::Json => output::json::render_check(&report, &ping_results)?,
            }

            let should_fail = fail_on_drift || cfg.fail_on_drift;
            if should_fail && !report.drifts.is_empty() {
                Ok(1)
            } else {
                Ok(0)
            }
        }

        Commands::Graph {
            manifest: manifest_path,
            format,
        } => {
            let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
            let m = manifest::Manifest::load(&path)?;

            match format {
                GraphFormat::Mermaid => output::mermaid::render_graph(&m),
                GraphFormat::Markdown => output::mermaid::render_markdown_table(&m),
            }
            Ok(0)
        }

        Commands::Export {
            manifest: manifest_path,
            format,
            ignore: cli_ignore,
        } => {
            let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
            let m = manifest::Manifest::load(&path)?;

            let mut ignore: Vec<String> = cfg.ignore.clone();
            ignore.extend(cli_ignore);

            let discovered = discovery::discover_services_with_ignore(&root, &m, &ignore);
            let mut report = drift::analyze(&m, &discovered, &root);
            report.manifest = path.display().to_string();

            match format {
                ExportFormat::Json => output::json::render_export(&m, &report)?,
                ExportFormat::Markdown => output::mermaid::render_export_markdown(&m, &report),
            }
            Ok(0)
        }

        Commands::Init { output, force } => {
            let output_path = output.unwrap_or_else(|| root.join("services.yaml"));
            init::run(&root, output_path, force)?;
            Ok(0)
        }

        Commands::Diff { before, after } => {
            let report = diff::diff_snapshots(&before, &after)?;
            diff::render_diff(&report);
            Ok(0)
        }

        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "svccat", &mut io::stdout());
            Ok(0)
        }
    }
}
