use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use std::io;
use std::process;
use svccat::cli::{Cli, Commands, ExportFormat, GraphFormat, OutputFormat, ReportFormat};
use svccat::{
    config, diff, discovery, drift, init, lint, manifest, output, ping, report, since, watch,
};

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
            team,
            since,
            fail_on_new_drift,
        } => {
            let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
            let full_m = manifest::Manifest::load(&path)?;

            // Build the working manifest, applying team filter when requested.
            let mut m = full_m.clone();
            if let Some(ref t) = team {
                m.services.retain(|s| {
                    s.team
                        .as_deref()
                        .map(|v| v.eq_ignore_ascii_case(t))
                        .unwrap_or(false)
                });
            }

            // Merge config ignore + CLI ignore patterns.
            let mut ignore: Vec<String> = cfg.ignore.clone();
            ignore.extend(cli_ignore);

            let discovered_all = discovery::discover_services_with_ignore(&root, &full_m, &ignore);

            // When a team filter is active, exclude discovered services that are known to
            // belong to other teams so they don't show up as UndeclaredInRepo noise.
            let in_scope_names: std::collections::HashSet<&str> =
                m.services.iter().map(|s| s.name.as_str()).collect();
            let other_declared_names: std::collections::HashSet<&str> = full_m
                .services
                .iter()
                .filter(|s| !in_scope_names.contains(s.name.as_str()))
                .map(|s| s.name.as_str())
                .collect();
            let discovered: Vec<_> = discovered_all
                .into_iter()
                .filter(|d| !other_declared_names.contains(d.name.as_str()))
                .collect();

            let mut report = drift::analyze(&m, &discovered, &root);
            report.manifest = path.display().to_string();

            let ping_results = if do_ping {
                ping::ping_services(&m)
            } else {
                vec![]
            };

            // --since: load the old manifest at the given git ref and diff.
            if let Some(ref git_ref) = since {
                let old_m = since::load_at_ref(&root, &path, git_ref)?;
                let mut old_report = drift::analyze(&old_m, &discovered, &root);
                old_report.manifest = path.display().to_string();

                let new_count = match format {
                    OutputFormat::Markdown => {
                        let md = output::markdown::render_since_diff_markdown(
                            &old_report,
                            &report,
                            git_ref,
                        );
                        print!("{}", md);
                        // Count new items for exit code
                        use std::collections::HashSet;
                        let old_keys: HashSet<String> = old_report
                            .drifts
                            .iter()
                            .map(|d| {
                                format!(
                                    "{:?}|{}|{}",
                                    d.kind,
                                    d.service,
                                    d.detail.as_deref().unwrap_or("")
                                )
                            })
                            .collect();
                        report
                            .drifts
                            .iter()
                            .filter(|d| {
                                let k = format!(
                                    "{:?}|{}|{}",
                                    d.kind,
                                    d.service,
                                    d.detail.as_deref().unwrap_or("")
                                );
                                !old_keys.contains(&k)
                            })
                            .count()
                    }
                    OutputFormat::GithubAnnotation => {
                        output::github_annotation::render_since_annotations(&old_report, &report)
                    }
                    _ => {
                        let (new_count, _) =
                            output::terminal::render_since_diff(&old_report, &report, git_ref);
                        new_count
                    }
                };

                if fail_on_new_drift && new_count > 0 {
                    return Ok(1);
                }
            } else {
                match format {
                    OutputFormat::Terminal => {
                        output::terminal::render_check(&report, &ping_results)
                    }
                    OutputFormat::Json => output::json::render_check(&report, &ping_results)?,
                    OutputFormat::Sarif => output::sarif::render_check(&report, &ping_results)?,
                    OutputFormat::Markdown => {
                        let md = output::markdown::render_check_markdown(&report, &ping_results);
                        print!("{}", md);
                    }
                    OutputFormat::GithubAnnotation => {
                        output::github_annotation::render_check(&report);
                    }
                }
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
            team,
        } => {
            let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
            let m = manifest::Manifest::load(&path)?;

            match format {
                GraphFormat::Mermaid => output::mermaid::render_graph_filtered(&m, team.as_deref()),
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

        Commands::Watch {
            manifest: manifest_path,
            fail_on_drift,
            team,
            ignore: cli_ignore,
        } => {
            let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
            let mut ignore: Vec<String> = cfg.ignore.clone();
            ignore.extend(cli_ignore);

            let initial_errors = watch::run(&path, &root, &ignore, team.as_deref())?;

            let should_fail = fail_on_drift || cfg.fail_on_drift;
            if should_fail && initial_errors > 0 {
                Ok(1)
            } else {
                Ok(0)
            }
        }

        Commands::Report {
            manifest: manifest_path,
            format,
            output: output_path,
            ignore: cli_ignore,
            history,
            badge,
        } => {
            let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
            let m = manifest::Manifest::load(&path)?;

            let mut ignore: Vec<String> = cfg.ignore.clone();
            ignore.extend(cli_ignore);

            let discovered = discovery::discover_services_with_ignore(&root, &m, &ignore);
            let mut drift_report = drift::analyze(&m, &discovered, &root);
            drift_report.manifest = path.display().to_string();

            // --badge takes priority: emit a Markdown badge snippet and exit.
            if badge {
                println!("{}", report::render_badge(&drift_report));
                return Ok(0);
            }

            let content = if let Some(n) = history {
                report::render_history_markdown(&root, &path, &discovered, n)?
            } else {
                match format {
                    ReportFormat::Markdown => report::render_markdown(&m, &drift_report),
                    ReportFormat::Html => report::render_html(&m, &drift_report),
                }
            };

            if let Some(out_path) = output_path {
                std::fs::write(&out_path, &content)?;
                eprintln!("wrote report to {}", out_path.display());
            } else {
                print!("{}", content);
            }
            Ok(0)
        }

        Commands::Lint {
            manifest: manifest_path,
        } => {
            let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
            let m = manifest::Manifest::load(&path)?;
            let result = lint::run(&m);
            lint::render(&result);
            if result.error_count() > 0 {
                Ok(1)
            } else {
                Ok(0)
            }
        }

        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "svccat", &mut io::stdout());
            Ok(0)
        }
    }
}
