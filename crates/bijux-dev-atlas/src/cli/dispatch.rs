// SPDX-License-Identifier: Apache-2.0

use crate::cli::{CheckCommand, CheckRegistryCommand, Cli, Command, FormatArg, ReleaseCommand};
use crate::{
    plugin_metadata_json, run_artifacts_command, run_build_command, run_capabilities_command,
    run_check_doctor, run_check_explain, run_check_list, run_check_registry_doctor,
    run_check_repo_doctor, run_check_root_surface_explain, run_check_run, run_check_tree_budgets, run_configs_command,
    run_contracts_command, run_demo_command, run_docker_command, run_docs_command,
    run_gates_command, run_help_inventory_command, run_make_command, run_ops_command,
    run_policies_command, run_print_boundaries_command, run_version_command, run_workflows_command,
};
use crate::{run_print_policies, CheckListOptions, CheckRunOptions};
use std::io::{self, Write};
use std::process::Command as ProcessCommand;

use crate::cli::dispatch_mutations::{apply_fail_fast, force_json_output, propagate_repo_root};
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
        Command::Contracts { command } => run_contracts_command(cli.quiet, command),
        Command::Demo { command } => run_demo_command(cli.quiet, command),
        Command::Configs { command } => run_configs_command(cli.quiet, command),
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
        Command::Release { command } => match command {
            ReleaseCommand::Check(args) => {
                let exe = match std::env::current_exe() {
                    Ok(path) => path,
                    Err(err) => {
                        let _ =
                            writeln!(io::stderr(), "bijux-dev-atlas release check failed: {err}");
                        return 1;
                    }
                };

                let mut validate_args = vec![
                    "validate".to_string(),
                    "--profile".to_string(),
                    args.profile.clone(),
                    "--format".to_string(),
                    "json".to_string(),
                ];
                if let Some(root) = &args.repo_root {
                    validate_args.push("--repo-root".to_string());
                    validate_args.push(root.display().to_string());
                }
                let validate_out = match ProcessCommand::new(&exe).args(&validate_args).output() {
                    Ok(v) => v,
                    Err(err) => {
                        let _ =
                            writeln!(io::stderr(), "bijux-dev-atlas release check failed: {err}");
                        return 1;
                    }
                };
                let validate_payload: serde_json::Value =
                    serde_json::from_slice(&validate_out.stdout).unwrap_or_else(|_| {
                        serde_json::json!({"status":"failed","stderr": String::from_utf8_lossy(&validate_out.stderr)})
                    });

                let mut readiness_args = vec![
                    "ops".to_string(),
                    "validate".to_string(),
                    "--profile".to_string(),
                    args.profile.clone(),
                    "--format".to_string(),
                    "json".to_string(),
                ];
                if let Some(root) = &args.repo_root {
                    readiness_args.push("--repo-root".to_string());
                    readiness_args.push(root.display().to_string());
                }
                let readiness_out = match ProcessCommand::new(&exe).args(&readiness_args).output() {
                    Ok(v) => v,
                    Err(err) => {
                        let _ =
                            writeln!(io::stderr(), "bijux-dev-atlas release check failed: {err}");
                        return 1;
                    }
                };
                let readiness_payload: serde_json::Value =
                    serde_json::from_slice(&readiness_out.stdout).unwrap_or_else(|_| {
                        serde_json::json!({"status":"failed","stderr": String::from_utf8_lossy(&readiness_out.stderr)})
                    });

                let ok = validate_out.status.success() && readiness_out.status.success();
                let payload = serde_json::json!({
                    "schema_version": 1,
                    "status": if ok { "ok" } else { "failed" },
                    "text": if ok { "release check passed" } else { "release check failed" },
                    "validate": validate_payload,
                    "ops_validate": readiness_payload
                });
                let rendered = match args.format {
                    FormatArg::Json => {
                        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
                    }
                    FormatArg::Text => {
                        if ok {
                            "release check passed: validate + ops validate".to_string()
                        } else {
                            "release check failed: rerun with --format json for details".to_string()
                        }
                    }
                    FormatArg::Jsonl => payload.to_string(),
                };
                if let Some(path) = args.out {
                    if let Err(err) = std::fs::write(&path, format!("{rendered}\n")) {
                        let _ =
                            writeln!(io::stderr(), "bijux-dev-atlas release check failed: {err}");
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
        },
        Command::Check { command } => {
            let result = match command {
                CheckCommand::Registry { command } => match command {
                    CheckRegistryCommand::Doctor {
                        repo_root,
                        format,
                        out,
                    } => run_check_registry_doctor(repo_root, format, out),
                },
                CheckCommand::List {
                    repo_root,
                    suite,
                    domain,
                    tag,
                    id,
                    include_internal,
                    include_slow,
                    format,
                    json,
                    out,
                } => run_check_list(CheckListOptions {
                    repo_root,
                    suite,
                    domain,
                    tag,
                    id,
                    include_internal,
                    include_slow,
                    format: if json { FormatArg::Json } else { format },
                    out,
                }),
                CheckCommand::Explain {
                    check_id,
                    repo_root,
                    format,
                    out,
                } => run_check_explain(check_id, repo_root, format, out),
                CheckCommand::Doctor {
                    repo_root,
                    include_internal,
                    include_slow,
                    format,
                    out,
                } => run_check_doctor(repo_root, include_internal, include_slow, format, out),
                CheckCommand::Run {
                    repo_root,
                    artifacts_root,
                    run_id,
                    suite,
                    domain,
                    tag,
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
                } => run_check_run(CheckRunOptions {
                    repo_root,
                    artifacts_root,
                    run_id,
                    suite,
                    domain,
                    tag,
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
                }),
                CheckCommand::TreeBudgets {
                    repo_root,
                    format,
                    out,
                } => run_check_tree_budgets(repo_root, format, out),
                CheckCommand::RepoDoctor {
                    repo_root,
                    format,
                    out,
                } => run_check_repo_doctor(repo_root, format, out),
                CheckCommand::RootSurfaceExplain {
                    repo_root,
                    format,
                    out,
                } => run_check_root_surface_explain(repo_root, format, out),
            };
            match result {
                Ok((rendered, code)) => {
                    if !cli.quiet && !rendered.is_empty() {
                        let _ = writeln!(io::stdout(), "{rendered}");
                    }
                    code
                }
                Err(err) => {
                    let _ = writeln!(io::stderr(), "bijux-dev-atlas check failed: {err}");
                    1
                }
            }
        }
        Command::Ops { command } => run_ops_command(cli.quiet, cli.debug, command),
    };

    if cli.verbose {
        let _ = writeln!(io::stderr(), "bijux-dev-atlas exit={exit}");
    }
    exit
}
