use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use std::io;
use std::process;
use svccat::cli::{
    AuditFormat, CiFormat, Cli, Commands, DepsFormat, DiffFormat, ExportFormat, GraphFormat,
    HookKind, ImportSource, OutputFormat, PolicyFormat, ReportFormat, ScorecardFormat,
    SnapshotAction, TagAction, WorkspaceAction,
};
use svccat::{
    audit, ci, config, demo, deps, diff, discovery, drift, fix, hooks, import, init, lint,
    manifest, output, ping, policy, report, scorecard, search, serve, since, snapshot, stats, tag,
    watch, webhook, workspace,
};

fn main() {
    // The `Commands` enum is large, and clap's command-tree construction can
    // exceed the default 1 MB main-thread stack on Windows (Linux defaults to
    // 8 MB, which is why CI never tripped it). Run on a thread with a bigger
    // stack so the CLI behaves identically across platforms.
    let worker = std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(run)
        .expect("failed to spawn worker thread");

    match worker.join() {
        Ok(Ok(code)) => process::exit(code),
        Ok(Err(e)) => {
            eprintln!("error: {e:#}");
            process::exit(2);
        }
        Err(_) => process::exit(101), // worker panicked; the default hook already printed it
    }
}

fn render_check_output_to_string(
    format: &OutputFormat,
    report: &drift::DriftReport,
    ping_results: &[ping::PingResult],
) -> Result<Option<String>> {
    let maybe_string = match format {
        OutputFormat::Json => Some(output::json::render_check_to_string(report, ping_results)?),
        OutputFormat::Markdown => Some(output::markdown::render_check_markdown(report, ping_results)),
        OutputFormat::Slack => Some(output::slack::render_check_to_string(report)?),
        OutputFormat::Teams => Some(output::teams::render_check_to_string(report)?),
        OutputFormat::Datadog => Some(output::datadog::render_check_to_string(report)?),
        _ => None,
    };

    Ok(maybe_string)
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
            depth,
            baseline,
            output: output_path,
        } => {
            // When running inside GitHub Actions and no explicit format was chosen,
            // default to github-annotation so drift items appear as inline PR comments.
            let format = if format == OutputFormat::Terminal
                && std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true")
            {
                OutputFormat::GithubAnnotation
            } else {
                format
            };
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

            let discovered_all =
                discovery::discover_services_with_opts(&root, &full_m, &ignore, depth);

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

            // --baseline: filter drift to only items absent from the saved baseline snapshot.
            if let Some(ref baseline_path) = baseline {
                use std::collections::HashSet;

                #[derive(serde::Deserialize)]
                struct BaselineFile {
                    drift: Vec<drift::DriftItem>,
                }

                let text = std::fs::read_to_string(baseline_path).map_err(|e| {
                    anyhow::anyhow!("cannot read baseline {}: {e}", baseline_path.display())
                })?;
                let snap: BaselineFile = serde_json::from_str(&text)
                    .map_err(|e| anyhow::anyhow!("cannot parse baseline JSON: {e}"))?;

                let baseline_keys: HashSet<String> = snap
                    .drift
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

                report.drifts.retain(|d| {
                    !baseline_keys.contains(&format!(
                        "{:?}|{}|{}",
                        d.kind,
                        d.service,
                        d.detail.as_deref().unwrap_or("")
                    ))
                });
            }

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
                    OutputFormat::Junit => {
                        output::junit::render_since(&old_report, &report, git_ref)
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
                // For string-renderable formats, capture once so we can write to --output.
                let maybe_string = render_check_output_to_string(&format, &report, &ping_results)?;

                if let Some(content) = maybe_string {
                    if let Some(ref out_path) = output_path {
                        std::fs::write(out_path, &content)?;
                        eprintln!("wrote output to {}", out_path.display());
                    } else {
                        print!("{}", content);
                    }
                } else {
                    match format {
                        OutputFormat::Terminal => {
                            output::terminal::render_check(&report, &ping_results)
                        }
                        OutputFormat::Compact => {
                            output::terminal::render_compact(&m, &report);
                        }
                        OutputFormat::Sarif => output::sarif::render_check(&report, &ping_results)?,
                        OutputFormat::Junit => output::junit::render_check(&report, &ping_results)?,
                        OutputFormat::GithubAnnotation => {
                            output::github_annotation::render_check(&report);
                        }
                        OutputFormat::Csv => output::csv::render_check(&report),
                        OutputFormat::Slack => output::slack::render_check(&report)?,
                        OutputFormat::Teams => output::teams::render_check(&report)?,
                        OutputFormat::Datadog => output::datadog::render_check(&report)?,
                        // Already handled above:
                        OutputFormat::Json | OutputFormat::Markdown => unreachable!(),
                    }
                }
            }

            let should_fail = fail_on_drift || cfg.fail_on_drift;

            // Fire webhook when configured.
            let wh_cfg = webhook::load_config(&root);
            let _ = webhook::fire_from_drift(&root, &wh_cfg, &report);

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
            filter,
            output: output_path,
        } => {
            let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
            let mut m = manifest::Manifest::load(&path)?;

            // Apply --filter: keep only services whose name contains the substring.
            if let Some(ref pat) = filter {
                let pat_lower = pat.to_lowercase();
                m.services
                    .retain(|s| s.name.to_lowercase().contains(&pat_lower));
            }

            let content = match format {
                GraphFormat::Mermaid => {
                    output::mermaid::render_graph_filtered_string(&m, team.as_deref())
                }
                GraphFormat::Markdown => output::mermaid::render_markdown_table_string(&m),
                GraphFormat::Dot => output::mermaid::render_dot_string(&m, team.as_deref()),
                GraphFormat::Plantuml => {
                    output::mermaid::render_plantuml_string(&m, team.as_deref())
                }
                GraphFormat::Html => output::mermaid::render_html_graph(&m, team.as_deref()),
            };

            if let Some(out_path) = output_path {
                std::fs::write(&out_path, &content)?;
                eprintln!("wrote graph to {}", out_path.display());
            } else {
                print!("{}", content);
            }
            Ok(0)
        }

        Commands::Export {
            manifest: manifest_path,
            format,
            ignore: cli_ignore,
            depth,
            since: since_ref,
        } => {
            let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
            let mut m = manifest::Manifest::load(&path)?;

            let mut ignore: Vec<String> = cfg.ignore.clone();
            ignore.extend(cli_ignore);

            let discovered = discovery::discover_services_with_opts(&root, &m, &ignore, depth);
            let mut report = drift::analyze(&m, &discovered, &root);
            report.manifest = path.display().to_string();

            // Filter to services that changed since the given git ref
            if let Some(ref git_ref) = since_ref {
                if let Ok(old_m) = since::load_at_ref(&root, &path, git_ref) {
                    let old_map: std::collections::HashMap<String, &manifest::ServiceEntry> =
                        old_m.services.iter().map(|s| (s.name.clone(), s)).collect();
                    m.services.retain(|svc| {
                        if let Some(old_svc) = old_map.get(&svc.name) {
                            svc != *old_svc
                        } else {
                            true // new service
                        }
                    });
                    // Rebuild report with the filtered manifest
                    let discovered2 =
                        discovery::discover_services_with_opts(&root, &m, &ignore, depth);
                    report = drift::analyze(&m, &discovered2, &root);
                    report.manifest = path.display().to_string();
                }
            }

            match format {
                ExportFormat::Json => output::json::render_export(&m, &report)?,
                ExportFormat::Markdown => output::mermaid::render_export_markdown(&m, &report),
                ExportFormat::Csv => output::csv::render_export(&m),
            }
            Ok(0)
        }

        Commands::Init { output, force } => {
            let output_path = output.unwrap_or_else(|| root.join("services.yaml"));
            init::run(&root, output_path, force)?;
            Ok(0)
        }

        Commands::Diff {
            before,
            after,
            format,
        } => {
            let report = diff::diff_snapshots(&before, &after)?;
            match format {
                DiffFormat::Terminal => diff::render_diff(&report),
                DiffFormat::Markdown => diff::render_diff_markdown(&report),
            }
            Ok(0)
        }

        Commands::Watch {
            manifest: manifest_path,
            fail_on_drift,
            team,
            ignore: cli_ignore,
            depth,
            since: watch_since,
            notify,
            interval,
        } => {
            let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
            let mut ignore: Vec<String> = cfg.ignore.clone();
            ignore.extend(cli_ignore);

            let initial_errors = watch::run(
                &path,
                &root,
                &ignore,
                team.as_deref(),
                depth,
                watch_since.as_deref(),
                notify,
                interval,
            )?;

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
                    ReportFormat::Json => report::render_json(&m, &drift_report)?,
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

        Commands::Import {
            from,
            output: output_path,
            force,
        } => {
            let out = output_path.unwrap_or_else(|| root.join("services.yaml"));
            match from {
                ImportSource::Backstage => import::run_backstage(&root, out, force)?,
                ImportSource::DockerCompose => import::run_docker_compose(&root, out, force)?,
                ImportSource::Openapi => import::run_openapi(&root, out, force)?,
            }
            Ok(0)
        }

        Commands::Fix {
            manifest: manifest_path,
            prune,
            dry_run,
            ignore: cli_ignore,
            depth,
        } => {
            let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
            let mut ignore: Vec<String> = cfg.ignore.clone();
            ignore.extend(cli_ignore);
            fix::run(&path, &root, &ignore, depth, prune, dry_run)?;
            Ok(0)
        }

        Commands::InstallHooks {
            hook,
            fail_on_drift,
        } => {
            let hook_name = match hook {
                HookKind::PreCommit => "pre-commit",
                HookKind::PrePush => "pre-push",
            };
            hooks::install(&root, hook_name, fail_on_drift)?;
            Ok(0)
        }

        Commands::Policy {
            manifest: manifest_path,
            format,
            fail_on_violations,
        } => {
            let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
            let m = manifest::Manifest::load(&path)?;
            let policy_cfg = policy::PolicyConfig::load(&root).unwrap_or_default();
            if policy_cfg.is_empty() {
                eprintln!(
                    "No policy file found. Create .svccat/policy.yaml to define required/recommended fields."
                );
                return Ok(0);
            }
            let result = policy::check(&m, &policy_cfg);
            match format {
                PolicyFormat::Terminal => policy::render_terminal(&result, &policy_cfg),
                PolicyFormat::Json => policy::render_json(&result)?,
            }
            if fail_on_violations && !result.passed() {
                Ok(1)
            } else {
                Ok(0)
            }
        }

        Commands::Snapshot { action } => match action {
            SnapshotAction::Save {
                name,
                manifest: manifest_path,
                ignore: cli_ignore,
                depth,
            } => {
                let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
                let m = manifest::Manifest::load(&path)?;
                let mut ignore: Vec<String> = cfg.ignore.clone();
                ignore.extend(cli_ignore);
                let discovered = discovery::discover_services_with_opts(&root, &m, &ignore, depth);
                let mut drift_report = drift::analyze(&m, &discovered, &root);
                drift_report.manifest = path.display().to_string();
                snapshot::save(&root, &name, &m, &drift_report)?;
                Ok(0)
            }
            SnapshotAction::List => {
                let snaps = snapshot::list(&root)?;
                snapshot::render_list(&snaps);
                Ok(0)
            }
            SnapshotAction::Delete { name } => {
                snapshot::delete(&root, &name)?;
                Ok(0)
            }
            SnapshotAction::Compare {
                before,
                after,
                format,
            } => {
                let diff_report = snapshot::compare(&root, &before, &after)?;
                match format {
                    DiffFormat::Terminal => diff::render_diff(&diff_report),
                    DiffFormat::Markdown => diff::render_diff_markdown(&diff_report),
                }
                Ok(0)
            }
            SnapshotAction::Diff {
                name,
                ignore: cli_ignore,
                depth,
                format,
            } => {
                // Load the named snapshot as the "before" baseline.
                let snap = snapshot::load(&root, &name)?;

                // Build the current state as the "after" payload.
                let manifest_path = manifest::find_default(&root);
                let m = manifest::Manifest::load(&manifest_path)?;
                let mut ignore: Vec<String> = cfg.ignore.clone();
                ignore.extend(cli_ignore);
                let discovered = discovery::discover_services_with_opts(&root, &m, &ignore, depth);
                let mut current_report = drift::analyze(&m, &discovered, &root);
                current_report.manifest = manifest_path.display().to_string();

                let after_payload = serde_json::json!({
                    "services": m.services,
                    "drift": current_report.drifts,
                });

                let diff_report =
                    diff::diff_from_json(&snap.payload, &after_payload, &name, "current")?;

                match format {
                    DiffFormat::Terminal => diff::render_diff(&diff_report),
                    DiffFormat::Markdown => diff::render_diff_markdown(&diff_report),
                }
                Ok(0)
            }
        },

        Commands::Audit {
            manifest: manifest_path,
            format,
            ping: do_ping,
            cost_estimate,
            ignore: cli_ignore,
            depth,
        } => {
            let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
            let mut ignore: Vec<String> = cfg.ignore.clone();
            ignore.extend(cli_ignore);
            let (result, lint_result, drift_report, ping_results) =
                audit::run(&path, &root, &ignore, depth, do_ping, cost_estimate)?;
            match format {
                AuditFormat::Terminal => {
                    audit::render_terminal(&result, &lint_result, &drift_report, &ping_results)
                }
                AuditFormat::Json => audit::render_json(&result)?,
            }
            if result.passed {
                Ok(0)
            } else {
                Ok(1)
            }
        }

        Commands::Ci {
            manifest: manifest_path,
            ignore: cli_ignore,
            depth,
            format,
            watch: do_watch,
            interval,
        } => {
            let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
            let mut ignore: Vec<String> = cfg.ignore.clone();
            ignore.extend(cli_ignore);

            if do_watch {
                let errors = ci::watch(&path, &root, &ignore, depth, interval)?;
                return if errors > 0 { Ok(1) } else { Ok(0) };
            }

            let m = manifest::Manifest::load(&path)?;
            let result = ci::run(&m, &root, &ignore, depth);
            match format {
                CiFormat::Terminal => ci::render_terminal(&result),
                CiFormat::Json => ci::render_json(&result)?,
            }

            // Fire webhook when configured.
            let wh_cfg = webhook::load_config(&root);
            let _ = webhook::fire_from_ci(&root, &wh_cfg, &result);

            if result.passed() {
                Ok(0)
            } else {
                Ok(1)
            }
        }

        Commands::Search {
            query: query_raw,
            manifest: manifest_path,
            output: output_path,
        } => {
            let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
            let m = manifest::Manifest::load(&path)?;
            let total = m.services.len();
            let q = search::Query::parse(&query_raw);
            let matches = search::run(&m, &q);
            if let Some(out_path) = output_path {
                let content = search::render_json(&matches, &query_raw, total)?;
                std::fs::write(&out_path, &content)?;
                eprintln!("wrote search results to {}", out_path.display());
            } else {
                search::render(&matches, &query_raw, total);
            }
            Ok(0)
        }

        Commands::Deps {
            manifest: manifest_path,
            format,
            output: output_path,
        } => {
            let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
            let m = manifest::Manifest::load(&path)?;
            let dep_report = deps::analyze(&m);
            if let Some(out_path) = output_path {
                let content = deps::render_json_to_string(&dep_report)?;
                std::fs::write(&out_path, &content)?;
                eprintln!("wrote deps report to {}", out_path.display());
            } else {
                match format {
                    DepsFormat::Terminal => deps::render_terminal(&dep_report),
                    DepsFormat::Mermaid => deps::render_mermaid(&dep_report),
                    DepsFormat::Json => deps::render_json(&dep_report)?,
                }
            }
            if dep_report.has_errors() {
                Ok(1)
            } else {
                Ok(0)
            }
        }

        Commands::Tag { action } => match action {
            TagAction::Add {
                service,
                tag,
                manifest: manifest_path,
            } => {
                let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
                tag::add(&path, &service, &tag)?;
                Ok(0)
            }
            TagAction::Remove {
                service,
                tag,
                manifest: manifest_path,
            } => {
                let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
                tag::remove(&path, &service, &tag)?;
                Ok(0)
            }
        },

        Commands::Scorecard {
            manifest: manifest_path,
            ignore: cli_ignore,
            depth,
            format,
            output: output_path,
        } => {
            let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
            let m = manifest::Manifest::load(&path)?;
            let mut ignore: Vec<String> = cfg.ignore.clone();
            ignore.extend(cli_ignore);
            let sc = scorecard::run(&m, &root, &ignore, depth);
            match &output_path {
                Some(out_path) => {
                    let content = match format {
                        ScorecardFormat::Terminal | ScorecardFormat::Markdown => {
                            scorecard::render_markdown(&sc)
                        }
                        ScorecardFormat::Json => scorecard::render_json_to_string(&sc)?,
                    };
                    std::fs::write(out_path, &content)?;
                    eprintln!("wrote scorecard to {}", out_path.display());
                }
                None => match format {
                    ScorecardFormat::Terminal => scorecard::render_terminal(&sc),
                    ScorecardFormat::Json => scorecard::render_json(&sc)?,
                    ScorecardFormat::Markdown => {
                        print!("{}", scorecard::render_markdown(&sc));
                    }
                },
            }
            Ok(0)
        }

        Commands::Webhook {
            manifest: manifest_path,
            ignore: cli_ignore,
            depth,
            url: url_override,
        } => {
            let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
            let m = manifest::Manifest::load(&path)?;
            let mut ignore: Vec<String> = cfg.ignore.clone();
            ignore.extend(cli_ignore);
            let discovered = discovery::discover_services_with_opts(&root, &m, &ignore, depth);
            let drift_report = drift::analyze(&m, &discovered, &root);

            let mut wh_cfg = webhook::load_config(&root);
            // URL from --url overrides config.
            if let Some(u) = url_override {
                wh_cfg.url = Some(u);
                wh_cfg.on_errors = true;
                wh_cfg.on_warnings = true;
            }
            webhook::fire_from_drift(&root, &wh_cfg, &drift_report)?;
            Ok(0)
        }

        Commands::Stats {
            manifest: manifest_path,
        } => {
            let path = manifest_path.unwrap_or_else(|| manifest::find_default(&root));
            let m = manifest::Manifest::load(&path)?;
            stats::run(&m);
            Ok(0)
        }

        Commands::Serve {
            manifest: _manifest_path,
            port,
            refresh,
        } => {
            serve::serve(&root, port, refresh)?;
            Ok(0)
        }

        Commands::Workspace { action } => {
            match action {
                WorkspaceAction::Check {
                    config: config_path,
                    filter: _filter,
                    format,
                    fail_on_drift,
                    ignore: cli_ignore,
                    depth,
                    output: output_path,
                } => {
                    // Load workspace configuration
                    let (workspace_config, workspace_root) =
                        workspace::load_workspace_config(&config_path)?;

                    // Merge ignore patterns
                    let mut ignore = cfg.ignore.clone();
                    ignore.extend(cli_ignore);

                    // Analyze all repositories
                    let report = workspace::analyze_workspace(
                        &workspace_config,
                        &workspace_root,
                        &ignore,
                        depth,
                    )?;

                    // Render output
                    let content = match format {
                        svccat::cli::OutputFormat::Json => output::workspace::render_json(&report)?,
                        svccat::cli::OutputFormat::Markdown => {
                            output::workspace::render_markdown(&report)
                        }
                        _ => {
                            output::workspace::render_terminal(&report);
                            String::new()
                        }
                    };

                    if let Some(out_path) = output_path {
                        std::fs::write(&out_path, &content)?;
                        eprintln!("wrote workspace report to {}", out_path.display());
                    } else if !content.is_empty() {
                        print!("{}", content);
                    }

                    if fail_on_drift && report.has_errors() {
                        Ok(1)
                    } else {
                        Ok(0)
                    }
                }
            }
        }

        Commands::Demo { keep } => demo::run(keep),

        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "svccat", &mut io::stdout());
            Ok(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_report() -> drift::DriftReport {
        serde_json::from_value(serde_json::json!({
            "manifest": "services.yaml",
            "declared": 2,
            "discovered": 2,
            "drifts": [
                {
                    "kind": "declared_missing_from_repo",
                    "severity": "error",
                    "service": "api",
                    "message": "missing service directory",
                    "detail": null
                }
            ]
        }))
        .unwrap()
    }

    #[test]
    fn string_output_helper_supports_slack_teams_and_datadog() {
        let report = sample_report();

        let slack = render_check_output_to_string(&OutputFormat::Slack, &report, &[])
            .unwrap()
            .unwrap();
        assert!(slack.contains("blocks"));

        let teams = render_check_output_to_string(&OutputFormat::Teams, &report, &[])
            .unwrap()
            .unwrap();
        assert!(teams.contains("attachments"));

        let datadog = render_check_output_to_string(&OutputFormat::Datadog, &report, &[])
            .unwrap()
            .unwrap();
        assert!(datadog.contains("events"));
    }

    #[test]
    fn string_output_helper_skips_terminal_format() {
        let report = sample_report();
        let out = render_check_output_to_string(&OutputFormat::Terminal, &report, &[]).unwrap();
        assert!(out.is_none());
    }
}
