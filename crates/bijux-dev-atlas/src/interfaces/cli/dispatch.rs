// SPDX-License-Identifier: Apache-2.0

use crate::cli::{
    CheckCommand, Cli, Command, FormatArg, GlobalFormatArg, ReportsCommand, TestsCommand,
    TestsModeArg,
};
use crate::run_print_policies;
use crate::{
    plugin_metadata_json, run_api_command, run_artifacts_command, run_audit_command,
    run_build_command, run_capabilities_command, run_configs_command, run_data_command,
    run_demo_command, run_docker_command, run_docs_command, run_drift_command, run_gates_command,
    run_governance_command, run_help_inventory_command, run_invariants_command, run_load_command,
    run_make_command, run_migrations_command, run_observe_command, run_ops_command,
    run_perf_command, run_policies_command, run_print_boundaries_command, run_registry_check_by_id,
    run_registry_command, run_registry_contract_by_id, run_release_command, run_reproduce_command,
    run_runtime_command, run_security_command, run_suites_command, run_system_command,
    run_tutorials_command, run_version_command, run_workflows_command,
};
use bijux_dev_atlas::domains;
use bijux_dev_atlas::model::exit_codes::{EXIT_FAILURE, EXIT_NOT_FOUND, EXIT_SUCCESS};
use bijux_dev_atlas::model::{RunnableId, RunnableKind, RunnableMode};
use bijux_dev_atlas::registry::{ReportRegistry, RunnableRegistry};
use bijux_dev_atlas::runtime::cli_adapter::{
    command_inventory_markdown, command_inventory_payload, route_name,
};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

use crate::cli::dispatch_mutations::{apply_fail_fast, force_json_output, propagate_repo_root};
use crate::resolve_repo_root;

fn run_check_surface_command(quiet: bool, command: CheckCommand) -> i32 {
    match command {
        CheckCommand::List {
            repo_root,
            suite,
            domain,
            severity,
            mode,
            tag,
            name,
            id,
            include_internal,
            include_slow,
            format,
            out,
        } => match crate::run_check_list(crate::CheckListOptions {
            repo_root,
            suite,
            domain,
            severity,
            mode,
            tag,
            name,
            id,
            include_internal,
            include_slow,
            format,
            out,
        }) {
            Ok((rendered, code)) => {
                if !quiet && !rendered.is_empty() {
                    let _ = writeln!(io::stdout(), "{rendered}");
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas check list failed: {err}");
                1
            }
        },
        CheckCommand::Explain {
            check_id,
            repo_root,
            format,
            out,
        } => match crate::run_check_explain(check_id, repo_root, format, out) {
            Ok((rendered, code)) => {
                if !quiet && !rendered.is_empty() {
                    let _ = writeln!(io::stdout(), "{rendered}");
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas check explain failed: {err}");
                1
            }
        },
        CheckCommand::Run {
            repo_root,
            artifacts_root,
            run_id,
            suite,
            domain,
            severity,
            mode,
            tag,
            name,
            id,
            include_internal,
            include_slow,
            allow_subprocess,
            allow_git,
            allow_write,
            allow_network,
            fail_fast,
            max_failures,
            format,
            out,
            durations,
        } => match crate::run_check_run(crate::CheckRunOptions {
            repo_root,
            artifacts_root,
            run_id,
            suite,
            domain,
            severity,
            mode,
            tag,
            name,
            id,
            include_internal,
            include_slow,
            allow_subprocess,
            allow_git,
            allow_write,
            allow_network,
            fail_fast,
            max_failures,
            format,
            out,
            durations,
        }) {
            Ok((rendered, code)) => {
                if !quiet && !rendered.is_empty() {
                    if code == 0 {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    } else {
                        let _ = writeln!(io::stderr(), "{rendered}");
                    }
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas check run failed: {err}");
                1
            }
        },
        CheckCommand::Doctor {
            repo_root,
            format,
            out,
            include_internal,
            include_slow,
        } => match crate::run_check_doctor(repo_root, include_internal, include_slow, format, out)
        {
            Ok((rendered, code)) => {
                if !quiet && !rendered.is_empty() {
                    if code == 0 {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    } else {
                        let _ = writeln!(io::stderr(), "{rendered}");
                    }
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas check doctor failed: {err}");
                1
            }
        },
    }
}

pub(crate) fn run_cli(cli: Cli) -> i32 {
    let mut cli = cli;
    if let Some(command) = cli.command.as_mut() {
        propagate_repo_root(command, cli.repo_root.clone());
        if cli.json {
            force_json_output(command);
        }
        if cli.fail_fast {
            apply_fail_fast(command);
        }
    }
    if cli.bijux_plugin_metadata {
        let _ = writeln!(io::stdout(), "{}", plugin_metadata_json());
        return 0;
    }
    if cli.print_policies {
        return match run_print_policies(cli.repo_root.clone()) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    let _ = writeln!(io::stdout(), "{rendered}");
                }
                code
            }
            Err(err) => {
                let _ = writeln!(
                    io::stderr(),
                    "bijux-dev-atlas --print-policies failed: {err}"
                );
                1
            }
        };
    }
    if cli.print_boundaries {
        return match run_print_boundaries_command() {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    let _ = writeln!(io::stdout(), "{rendered}");
                }
                code
            }
            Err(err) => {
                let _ = writeln!(
                    io::stderr(),
                    "bijux-dev-atlas --print-boundaries failed: {err}"
                );
                1
            }
        };
    }

    let Some(command) = cli.command else {
        let _ = writeln!(
            io::stderr(),
            "bijux-dev-atlas requires a subcommand unless --print-policies or --print-boundaries is provided"
        );
        return 2;
    };

    let exit = match command {
        Command::Version { format, out } => match run_version_command(format, out) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    let _ = writeln!(io::stdout(), "{rendered}");
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas version failed: {err}");
                1
            }
        },
        Command::Help { format, out } => match run_help_inventory_command(format, out) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    let _ = writeln!(io::stdout(), "{rendered}");
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas help failed: {err}");
                1
            }
        },
        Command::Docs { command } => run_docs_command(cli.quiet, command),
        Command::Artifacts { command } => run_artifacts_command(cli.quiet, command),
        Command::Make { command } => run_make_command(cli.quiet, command),
        Command::Demo { command } => run_demo_command(cli.quiet, command),
        Command::Configs { command } => run_configs_command(cli.quiet, command),
        Command::Governance { command } => match run_governance_command(cli.quiet, command) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    if code == 0 {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    } else {
                        let _ = writeln!(io::stderr(), "{rendered}");
                    }
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas governance failed: {err}");
                1
            }
        },
        Command::Security { command } => match run_security_command(cli.quiet, command) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    if code == 0 {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    } else {
                        let _ = writeln!(io::stderr(), "{rendered}");
                    }
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas security failed: {err}");
                1
            }
        },
        Command::Check { command } | Command::Checks { command } => {
            run_check_surface_command(cli.quiet, command)
        }
        Command::Runtime { command } => match run_runtime_command(cli.quiet, command) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    if code == 0 {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    } else {
                        let _ = writeln!(io::stderr(), "{rendered}");
                    }
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas runtime failed: {err}");
                1
            }
        },
        Command::Tutorials { command } => run_tutorials_command(cli.quiet, command),
        Command::Migrations { command } => run_migrations_command(cli.quiet, command),
        Command::System { command } => match run_system_command(cli.quiet, command) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    if code == 0 {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    } else {
                        let _ = writeln!(io::stderr(), "{rendered}");
                    }
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas system failed: {err}");
                1
            }
        },
        Command::Audit { command } => match run_audit_command(cli.quiet, command) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    if code == 0 {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    } else {
                        let _ = writeln!(io::stderr(), "{rendered}");
                    }
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas audit failed: {err}");
                1
            }
        },
        Command::Observe { command } => match run_observe_command(cli.quiet, command) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    if code == 0 {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    } else {
                        let _ = writeln!(io::stderr(), "{rendered}");
                    }
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas observe failed: {err}");
                1
            }
        },
        Command::Api { command } => match run_api_command(cli.quiet, command) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    if code == 0 {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    } else {
                        let _ = writeln!(io::stderr(), "{rendered}");
                    }
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas api failed: {err}");
                1
            }
        },
        Command::Load { command } => match run_load_command(cli.quiet, command) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    if code == 0 {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    } else {
                        let _ = writeln!(io::stderr(), "{rendered}");
                    }
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas load failed: {err}");
                1
            }
        },
        Command::Invariants { command } => match run_invariants_command(cli.quiet, command) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    if code == 0 {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    } else {
                        let _ = writeln!(io::stderr(), "{rendered}");
                    }
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas invariants failed: {err}");
                1
            }
        },
        Command::Drift { command } => match run_drift_command(cli.quiet, command) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    if code == 0 {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    } else {
                        let _ = writeln!(io::stderr(), "{rendered}");
                    }
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas drift failed: {err}");
                1
            }
        },
        Command::Reproduce { command } => match run_reproduce_command(cli.quiet, command) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    if code == 0 {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    } else {
                        let _ = writeln!(io::stderr(), "{rendered}");
                    }
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas reproduce failed: {err}");
                1
            }
        },
        Command::Datasets { command } => match run_data_command(
            cli.quiet,
            crate::commands_data::DataCommand::Datasets(command),
        ) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    if code == 0 {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    } else {
                        let _ = writeln!(io::stderr(), "{rendered}");
                    }
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas datasets failed: {err}");
                1
            }
        },
        Command::Ingest { command } => match run_data_command(
            cli.quiet,
            crate::commands_data::DataCommand::Ingest(command),
        ) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    if code == 0 {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    } else {
                        let _ = writeln!(io::stderr(), "{rendered}");
                    }
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas ingest failed: {err}");
                1
            }
        },
        Command::Perf { command } => match run_perf_command(cli.quiet, command) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    if code == 0 {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    } else {
                        let _ = writeln!(io::stderr(), "{rendered}");
                    }
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas perf failed: {err}");
                1
            }
        },
        Command::Docker { command } => run_docker_command(cli.quiet, command),
        Command::Build { command } => run_build_command(cli.quiet, command),
        Command::Policies { command } => run_policies_command(cli.quiet, command),
        Command::Ci { command } => run_workflows_command(cli.quiet, command),
        Command::Workflows { command } => run_workflows_command(cli.quiet, command),
        Command::Gates { command } => run_gates_command(cli.quiet, command),
        Command::Capabilities { format, out } => match run_capabilities_command(format, out) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    let _ = writeln!(io::stdout(), "{rendered}");
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas capabilities failed: {err}");
                1
            }
        },
        Command::Validate {
            repo_root,
            profile,
            format,
            out,
        } => {
            let exe = match std::env::current_exe() {
                Ok(path) => path,
                Err(err) => {
                    let _ = writeln!(io::stderr(), "bijux-dev-atlas validate failed: {err}");
                    return 1;
                }
            };
            let mut check_args = vec![
                "check".to_string(),
                "run".to_string(),
                "--suite".to_string(),
                "ci_pr".to_string(),
                "--format".to_string(),
                "json".to_string(),
            ];
            if let Some(root) = &repo_root {
                check_args.push("--repo-root".to_string());
                check_args.push(root.display().to_string());
            }
            let check_out = match ProcessCommand::new(&exe).args(&check_args).output() {
                Ok(v) => v,
                Err(err) => {
                    let _ = writeln!(io::stderr(), "bijux-dev-atlas validate failed: {err}");
                    return 1;
                }
            };
            let check_payload: serde_json::Value = match serde_json::from_slice(&check_out.stdout) {
                Ok(v) => v,
                Err(_) => {
                    serde_json::json!({"status":"failed","stderr": String::from_utf8_lossy(&check_out.stderr)})
                }
            };

            let mut ops_args = vec![
                "ops".to_string(),
                "validate".to_string(),
                "--profile".to_string(),
                profile,
                "--format".to_string(),
                "json".to_string(),
            ];
            if let Some(root) = &repo_root {
                ops_args.push("--repo-root".to_string());
                ops_args.push(root.display().to_string());
            }
            let ops_out = match ProcessCommand::new(&exe).args(&ops_args).output() {
                Ok(v) => v,
                Err(err) => {
                    let _ = writeln!(io::stderr(), "bijux-dev-atlas validate failed: {err}");
                    return 1;
                }
            };
            let ops_payload: serde_json::Value = match serde_json::from_slice(&ops_out.stdout) {
                Ok(v) => v,
                Err(_) => {
                    serde_json::json!({"status":"failed","stderr": String::from_utf8_lossy(&ops_out.stderr)})
                }
            };

            let ok = check_out.status.success() && ops_out.status.success();
            let payload = serde_json::json!({
                "schema_version": 1,
                "status": if ok { "ok" } else { "failed" },
                "text": if ok { "validate completed" } else { "validate failed" },
                "checks_ci_pr": check_payload,
                "ops_validate": ops_payload,
            });
            let rendered = match format {
                FormatArg::Json => {
                    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
                }
                FormatArg::Text => {
                    if ok {
                        "validate completed: check run --suite ci_pr + ops validate".to_string()
                    } else {
                        "validate failed: rerun with --format json for details".to_string()
                    }
                }
                FormatArg::Jsonl => payload.to_string(),
            };
            if let Some(path) = out {
                if let Err(err) = std::fs::write(&path, format!("{rendered}\n")) {
                    let _ = writeln!(io::stderr(), "bijux-dev-atlas validate failed: {err}");
                    return 1;
                }
            }
            if !cli.quiet && !rendered.is_empty() {
                let _ = writeln!(io::stdout(), "{rendered}");
            }
            if ok {
                0
            } else {
                1
            }
        }
        Command::Release { command } => match run_release_command(cli.quiet, command) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    if code == 0 {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    } else {
                        let _ = writeln!(io::stderr(), "{rendered}");
                    }
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas release failed: {err}");
                1
            }
        },
        Command::Registry { command } => run_registry_command(cli.quiet, command),
        Command::Suites { command } => run_suites_command(cli.quiet, command),
        Command::Tests { command } => {
            let result = match command {
                TestsCommand::List {
                    repo_root,
                    format,
                    out,
                } => run_tests_list(repo_root, format, out),
                TestsCommand::Run {
                    repo_root,
                    artifacts_root,
                    run_id,
                    mode,
                    fail_fast,
                    format,
                    out,
                } => run_tests_run(TestsRunArgs {
                    repo_root,
                    artifacts_root,
                    run_id,
                    mode,
                    fail_fast,
                    format,
                    out,
                }),
                TestsCommand::Doctor {
                    repo_root,
                    artifacts_root,
                    run_id,
                    format,
                    out,
                } => run_tests_doctor(repo_root, artifacts_root, run_id, format, out),
            };
            match result {
                Ok((rendered, code)) => {
                    if !cli.quiet && !rendered.is_empty() {
                        if code == 0 {
                            let _ = writeln!(io::stdout(), "{rendered}");
                        } else {
                            let _ = writeln!(io::stderr(), "{rendered}");
                        }
                    }
                    code
                }
                Err(err) => {
                    let _ = writeln!(io::stderr(), "bijux-dev-atlas tests failed: {err}");
                    1
                }
            }
        }
        Command::Reports { command } => match run_reports_command(cli.output_format, command) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    let _ = writeln!(io::stdout(), "{rendered}");
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas reports failed: {err}");
                EXIT_FAILURE
            }
        },
        Command::List {
            repo_root,
            format,
            out,
        } => match run_list_command(cli.output_format, repo_root, format, out) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    let _ = writeln!(io::stdout(), "{rendered}");
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas list failed: {err}");
                EXIT_FAILURE
            }
        },
        Command::Describe {
            id,
            repo_root,
            format,
            out,
        } => match run_describe_command(cli.output_format, repo_root, &id, format, out) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    let _ = writeln!(io::stdout(), "{rendered}");
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas describe failed: {err}");
                EXIT_FAILURE
            }
        },
        Command::Run {
            id,
            repo_root,
            artifacts_root,
            run_id,
            format,
            out,
        } => match run_runnable_command(
            cli.output_format,
            repo_root,
            artifacts_root,
            run_id,
            &id,
            format,
            out,
        ) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    let _ = writeln!(io::stdout(), "{rendered}");
                }
                code
            }
            Err(err) => {
                let _ = writeln!(io::stderr(), "bijux-dev-atlas run failed: {err}");
                EXIT_FAILURE
            }
        },
        Command::Ops { command } => run_ops_command(cli.quiet, cli.debug, command),
    };

    if cli.verbose {
        let _ = writeln!(io::stderr(), "bijux-dev-atlas exit={exit}");
    }
    exit
}

fn run_tests_list(
    repo_root: Option<PathBuf>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(repo_root)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "suite": "tests",
        "entries": [
            {"id":"tests.fast","mode":"fast","target":"test"},
            {"id":"tests.all","mode":"all","target":"test-all"}
        ]
    });
    let rendered = match format {
        FormatArg::Json => serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("encode tests list failed: {err}"))?,
        FormatArg::Text | FormatArg::Jsonl => "tests.fast\ntests.all".to_string(),
    };
    if let Some(path) = out {
        fs::write(&path, format!("{rendered}\n"))
            .map_err(|err| format!("write {} failed: {err}", path.display()))?;
    }
    let _ = repo_root;
    Ok((rendered, 0))
}

fn run_tests_doctor(
    repo_root: Option<PathBuf>,
    artifacts_root: Option<PathBuf>,
    run_id: Option<String>,
    format: FormatArg,
    out: Option<PathBuf>,
) -> Result<(String, i32), String> {
    run_tests_run(TestsRunArgs {
        repo_root,
        artifacts_root,
        run_id,
        mode: TestsModeArg::Fast,
        fail_fast: true,
        format,
        out,
    })
}

struct TestsRunArgs {
    repo_root: Option<PathBuf>,
    artifacts_root: Option<PathBuf>,
    run_id: Option<String>,
    mode: TestsModeArg,
    fail_fast: bool,
    format: FormatArg,
    out: Option<PathBuf>,
}

fn run_tests_run(args: TestsRunArgs) -> Result<(String, i32), String> {
    let TestsRunArgs {
        repo_root,
        artifacts_root,
        run_id,
        mode,
        fail_fast,
        format,
        out,
    } = args;
    let repo_root = resolve_repo_root(repo_root)?;
    let artifacts_root = artifacts_root.unwrap_or_else(|| repo_root.join("artifacts"));
    let run_id = run_id.unwrap_or_else(|| "tests-local".to_string());
    fs::create_dir_all(&artifacts_root)
        .map_err(|err| format!("create {} failed: {err}", artifacts_root.display()))?;
    let target = match mode {
        TestsModeArg::Fast => "test",
        TestsModeArg::All => "test-all",
    };
    let started = std::time::Instant::now();
    let status = ProcessCommand::new("make")
        .arg("-s")
        .arg(target)
        .env("ARTIFACT_ROOT", &artifacts_root)
        .env("RUN_ID", &run_id)
        .env("FAIL_FAST", if fail_fast { "1" } else { "0" })
        .current_dir(&repo_root)
        .status()
        .map_err(|err| format!("run make {target} failed: {err}"))?;
    let duration_ms = started.elapsed().as_millis() as u64;
    let make_exit_code = status.code().unwrap_or(1);
    let make_success = status.success();
    let exit_code = if make_success { 0 } else { make_exit_code };
    let payload = serde_json::json!({
        "schema_version": 1,
        "suite": "tests",
        "run_id": run_id,
        "target": target,
        "mode": match mode { TestsModeArg::Fast => "fast", TestsModeArg::All => "all" },
        "status": if exit_code == 0 { "PASS" } else { "FAIL" },
        "exit_code": exit_code,
        "duration_ms": duration_ms,
        "artifacts_root": artifacts_root.display().to_string()
    });
    let report_dir = artifacts_root
        .join("tests")
        .join(payload["run_id"].as_str().unwrap_or("local"));
    fs::create_dir_all(&report_dir)
        .map_err(|err| format!("create {} failed: {err}", report_dir.display()))?;
    let report_path = report_dir.join("report.json");
    fs::write(
        &report_path,
        serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("encode tests report failed: {err}"))?,
    )
    .map_err(|err| format!("write {} failed: {err}", report_path.display()))?;
    let rendered = match format {
        FormatArg::Json => serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("encode tests run output failed: {err}"))?,
        FormatArg::Text | FormatArg::Jsonl => format!(
            "tests-run: mode={} target={} status={} duration_ms={} report={}",
            payload["mode"].as_str().unwrap_or("fast"),
            target,
            payload["status"].as_str().unwrap_or("FAIL"),
            duration_ms,
            report_path.display()
        ),
    };
    if let Some(path) = out {
        fs::write(&path, format!("{rendered}\n"))
            .map_err(|err| format!("write {} failed: {err}", path.display()))?;
    }
    Ok((rendered, if exit_code == 0 { 0 } else { 1 }))
}

fn effective_format(global: Option<GlobalFormatArg>, local: FormatArg) -> GlobalFormatArg {
    global.unwrap_or(match local {
        FormatArg::Text => GlobalFormatArg::Human,
        FormatArg::Json | FormatArg::Jsonl => GlobalFormatArg::Json,
    })
}

fn run_reports_command(
    global: Option<GlobalFormatArg>,
    command: ReportsCommand,
) -> Result<(String, i32), String> {
    match command {
        ReportsCommand::List(args) => {
            let repo_root = resolve_repo_root(args.repo_root)?;
            let registry = ReportRegistry::load(&repo_root)?;
            let catalog = ReportRegistry::validate_catalog(&repo_root)?;
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "reports_catalog",
                "summary": {
                    "report_count": catalog.report_count,
                    "validation_errors": catalog.errors.len(),
                },
                "reports": registry.reports.iter().map(|entry| serde_json::json!({
                    "report_id": entry.report_id,
                    "version": entry.version,
                    "schema_path": entry.schema_path,
                    "example_path": entry.example_path,
                })).collect::<Vec<_>>(),
                "validation_errors": catalog.errors,
            });
            let rendered = match effective_format(global, args.format) {
                GlobalFormatArg::Human => {
                    let mut lines = vec!["Reports".to_string()];
                    for report in payload["reports"].as_array().into_iter().flatten() {
                        lines.push(format!(
                            "- {} v{} -> {}",
                            report["report_id"].as_str().unwrap_or_default(),
                            report["version"].as_u64().unwrap_or(0),
                            report["schema_path"].as_str().unwrap_or_default()
                        ));
                    }
                    if let Some(errors) = payload["validation_errors"].as_array() {
                        if !errors.is_empty() {
                            lines.push(String::new());
                            lines.push("Validation errors".to_string());
                            for error in errors {
                                lines.push(format!("- {}", error.as_str().unwrap_or_default()));
                            }
                        }
                    }
                    lines.join("\n")
                }
                GlobalFormatArg::Json => serde_json::to_string_pretty(&payload)
                    .map_err(|err| format!("encode reports list failed: {err}"))?,
                GlobalFormatArg::Both => format!(
                    "{}\n{}",
                    payload["reports"]
                        .as_array()
                        .map(|rows| rows
                            .iter()
                            .map(|row| format!(
                                "{} v{}",
                                row["report_id"].as_str().unwrap_or_default(),
                                row["version"].as_u64().unwrap_or(0)
                            ))
                            .collect::<Vec<_>>()
                            .join("\n"))
                        .unwrap_or_default(),
                    serde_json::to_string_pretty(&payload)
                        .map_err(|err| format!("encode reports list failed: {err}"))?
                ),
            };
            if let Some(path) = args.out {
                std::fs::write(&path, format!("{rendered}\n"))
                    .map_err(|err| format!("write {} failed: {err}", path.display()))?;
            }
            Ok((
                rendered,
                if catalog.errors.is_empty() {
                    EXIT_SUCCESS
                } else {
                    EXIT_FAILURE
                },
            ))
        }
        ReportsCommand::Index(args) => {
            let repo_root = resolve_repo_root(args.repo_root)?;
            let markdown = ReportRegistry::render_index_markdown(&repo_root)?;
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "reports_index",
                "markdown": markdown,
            });
            let rendered = match effective_format(global, args.format) {
                GlobalFormatArg::Human => {
                    payload["markdown"].as_str().unwrap_or_default().to_string()
                }
                GlobalFormatArg::Json => serde_json::to_string_pretty(&payload)
                    .map_err(|err| format!("encode reports index failed: {err}"))?,
                GlobalFormatArg::Both => format!(
                    "{}\n{}",
                    payload["markdown"].as_str().unwrap_or_default(),
                    serde_json::to_string_pretty(&payload)
                        .map_err(|err| format!("encode reports index failed: {err}"))?
                ),
            };
            if let Some(path) = args.out {
                std::fs::write(&path, format!("{rendered}\n"))
                    .map_err(|err| format!("write {} failed: {err}", path.display()))?;
            }
            Ok((rendered, EXIT_SUCCESS))
        }
        ReportsCommand::Progress(args) => {
            let repo_root = resolve_repo_root(args.repo_root)?;
            let progress = ReportRegistry::progress(&repo_root)?;
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "reports_progress",
                "summary": {
                    "total_reports": progress.total_reports,
                    "missing_example_paths": progress.missing_example_paths,
                    "missing_schema_files": progress.missing_schema_files,
                    "rows": progress.rows.len(),
                },
                "rows": progress.rows.iter().map(|row| serde_json::json!({
                    "report_id": row.report_id,
                    "missing": row.missing,
                })).collect::<Vec<_>>()
            });
            let rendered = match effective_format(global, args.format) {
                GlobalFormatArg::Human => {
                    let mut lines = vec![format!(
                        "Reports: {} total, {} missing example paths, {} missing schema files",
                        progress.total_reports,
                        progress.missing_example_paths,
                        progress.missing_schema_files
                    )];
                    for row in &progress.rows {
                        lines.push(format!("- {}: {}", row.report_id, row.missing.join(", ")));
                    }
                    lines.join("\n")
                }
                GlobalFormatArg::Json => serde_json::to_string_pretty(&payload)
                    .map_err(|err| format!("encode reports progress failed: {err}"))?,
                GlobalFormatArg::Both => format!(
                    "{}\n{}",
                    payload["summary"]["rows"].as_u64().unwrap_or(0),
                    serde_json::to_string_pretty(&payload)
                        .map_err(|err| format!("encode reports progress failed: {err}"))?
                ),
            };
            if let Some(path) = args.out {
                std::fs::write(&path, format!("{rendered}\n"))
                    .map_err(|err| format!("write {} failed: {err}", path.display()))?;
            }
            Ok((rendered, EXIT_SUCCESS))
        }
        ReportsCommand::Validate(args) => {
            let repo_root = resolve_repo_root(args.repo_root)?;
            let validation = ReportRegistry::validate_reports_dir(&repo_root, &args.dir)?;
            let payload = serde_json::json!({
                "schema_version": 1,
                "kind": "reports_validation",
                "summary": {
                    "scanned_reports": validation.scanned_reports,
                    "errors": validation.errors.len(),
                },
                "reports_dir": args.dir.display().to_string(),
                "errors": validation.errors,
            });
            let rendered = match effective_format(global, args.format) {
                GlobalFormatArg::Human => {
                    let mut lines = vec![format!(
                        "Validated {} reports in {}",
                        payload["summary"]["scanned_reports"].as_u64().unwrap_or(0),
                        args.dir.display()
                    )];
                    for error in payload["errors"].as_array().into_iter().flatten() {
                        lines.push(format!("- {}", error.as_str().unwrap_or_default()));
                    }
                    lines.join("\n")
                }
                GlobalFormatArg::Json => serde_json::to_string_pretty(&payload)
                    .map_err(|err| format!("encode reports validation failed: {err}"))?,
                GlobalFormatArg::Both => format!(
                    "Validated {} reports\n{}",
                    payload["summary"]["scanned_reports"].as_u64().unwrap_or(0),
                    serde_json::to_string_pretty(&payload)
                        .map_err(|err| format!("encode reports validation failed: {err}"))?
                ),
            };
            if let Some(path) = args.out {
                std::fs::write(&path, format!("{rendered}\n"))
                    .map_err(|err| format!("write {} failed: {err}", path.display()))?;
            }
            Ok((
                rendered,
                if validation.errors.is_empty() {
                    EXIT_SUCCESS
                } else {
                    EXIT_FAILURE
                },
            ))
        }
    }
}

fn run_list_command(
    global: Option<GlobalFormatArg>,
    repo_root: Option<std::path::PathBuf>,
    local: FormatArg,
    out: Option<std::path::PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let registry = RunnableRegistry::load(&root)?;
    let domains = domains::load_domains(&root)?;
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "runnable_inventory",
        "domains": domains.into_iter().map(|catalog| serde_json::json!({
            "name": catalog.registration.name,
            "runnables": catalog.runnables.len(),
        })).collect::<Vec<_>>(),
        "suites": registry.suites().iter().map(|suite| serde_json::json!({
            "id": suite.id.as_str(),
            "runnables": suite.runnables.iter().map(|id| id.as_str()).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "runnables": registry.all().iter().map(|entry| serde_json::json!({
            "id": entry.id.as_str(),
            "suite": entry.suite.as_str(),
            "kind": match entry.kind {
                RunnableKind::Check => "check",
                RunnableKind::Contract => "contract",
            },
            "mode": match entry.mode {
                RunnableMode::Pure => "pure",
                RunnableMode::Effect => "effect",
            },
            "group": entry.group,
        })).collect::<Vec<_>>(),
    });
    let rendered = match effective_format(global, local) {
        GlobalFormatArg::Human => {
            let mut lines = vec!["Domains".to_string()];
            for row in payload["domains"].as_array().into_iter().flatten() {
                lines.push(format!(
                    "- {} ({})",
                    row["name"].as_str().unwrap_or_default(),
                    row["runnables"].as_u64().unwrap_or(0)
                ));
            }
            lines.push(String::new());
            lines.push(command_inventory_markdown());
            lines.join("\n")
        }
        GlobalFormatArg::Json => serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("encode runnable inventory failed: {err}"))?,
        GlobalFormatArg::Both => format!(
            "{}\n{}",
            command_inventory_markdown(),
            serde_json::to_string_pretty(&payload)
                .map_err(|err| format!("encode runnable inventory failed: {err}"))?
        ),
    };
    if let Some(path) = out {
        std::fs::write(&path, format!("{rendered}\n"))
            .map_err(|err| format!("write {} failed: {err}", path.display()))?;
    }
    Ok((rendered, EXIT_SUCCESS))
}

fn run_describe_command(
    global: Option<GlobalFormatArg>,
    repo_root: Option<std::path::PathBuf>,
    id: &str,
    local: FormatArg,
    out: Option<std::path::PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let registry = RunnableRegistry::load(&root)?;
    let runnable_id = RunnableId::parse(id)?;
    let Some(entry) = registry.get(&runnable_id) else {
        return Ok((format!("unknown runnable `{id}`"), EXIT_NOT_FOUND));
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "runnable_description",
        "id": entry.id.as_str(),
        "suite": entry.suite.as_str(),
        "kind_name": match entry.kind {
            RunnableKind::Check => "check",
            RunnableKind::Contract => "contract",
        },
        "mode": match entry.mode {
            RunnableMode::Pure => "pure",
            RunnableMode::Effect => "effect",
        },
        "summary": entry.summary,
        "owner": entry.owner,
        "group": entry.group,
        "tags": entry.tags.iter().map(|tag| tag.as_str()).collect::<Vec<_>>(),
        "required_tools": entry.required_tools,
        "route": route_name(entry.suite.as_str()),
    });
    let rendered = match effective_format(global, local) {
        GlobalFormatArg::Human => format!(
            "{}\nsuite: {}\nkind: {}\nmode: {}\ngroup: {}\nsummary: {}",
            entry.id,
            entry.suite,
            payload["kind_name"].as_str().unwrap_or_default(),
            payload["mode"].as_str().unwrap_or_default(),
            entry.group,
            entry.summary
        ),
        GlobalFormatArg::Json => serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("encode runnable description failed: {err}"))?,
        GlobalFormatArg::Both => format!(
            "{}\n{}",
            entry.id,
            serde_json::to_string_pretty(&payload)
                .map_err(|err| format!("encode runnable description failed: {err}"))?
        ),
    };
    if let Some(path) = out {
        std::fs::write(&path, format!("{rendered}\n"))
            .map_err(|err| format!("write {} failed: {err}", path.display()))?;
    }
    Ok((rendered, EXIT_SUCCESS))
}

fn run_runnable_command(
    global: Option<GlobalFormatArg>,
    repo_root: Option<std::path::PathBuf>,
    artifacts_root: Option<std::path::PathBuf>,
    run_id: Option<String>,
    id: &str,
    local: FormatArg,
    out: Option<std::path::PathBuf>,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(repo_root)?;
    let registry = RunnableRegistry::load(&root)?;
    let runnable_id = RunnableId::parse(id)?;
    let Some(entry) = registry.get(&runnable_id) else {
        return Ok((format!("unknown runnable `{id}`"), EXIT_NOT_FOUND));
    };
    let requested = match effective_format(global, local) {
        GlobalFormatArg::Human => FormatArg::Text,
        GlobalFormatArg::Json | GlobalFormatArg::Both => FormatArg::Json,
    };
    let result = match entry.kind {
        RunnableKind::Check => run_registry_check_by_id(
            Some(root),
            artifacts_root,
            run_id,
            id.to_string(),
            false,
            requested,
            out,
        )?,
        RunnableKind::Contract => run_registry_contract_by_id(
            Some(root),
            artifacts_root,
            run_id,
            id.to_string(),
            false,
            requested,
            out,
        )?,
    };
    if matches!(effective_format(global, local), GlobalFormatArg::Both) {
        let summary = serde_json::json!({
            "schema_version": 1,
            "kind": "runnable_execution",
            "id": id,
            "route": route_name(entry.suite.as_str()),
            "status_code": result.1,
            "command_registry": command_inventory_payload(),
        });
        let rendered = format!(
            "{}\n{}",
            result.0,
            serde_json::to_string_pretty(&summary)
                .map_err(|err| format!("encode runnable execution summary failed: {err}"))?
        );
        return Ok((rendered, result.1));
    }
    Ok(result)
}
