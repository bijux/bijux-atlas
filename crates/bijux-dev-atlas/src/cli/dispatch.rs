// SPDX-License-Identifier: Apache-2.0

use crate::cli::{
    CheckCommand, CheckRegistryCommand, Cli, Command, ContractCommand, ContractEffectsPolicyArg,
    ContractRunModeArg, ContractsColorArg, FormatArg, GlobalFormatArg, ReportsCommand,
};
use crate::{
    plugin_metadata_json, run_artifacts_command, run_build_command, run_capabilities_command,
    run_check_doctor, run_check_explain, run_check_list, run_check_registry_doctor,
    run_check_repo_doctor, run_check_root_surface_explain, run_check_run, run_check_tree_budgets,
    run_configs_command, run_contracts_command, run_data_command, run_demo_command,
    run_docker_command, run_docs_command, run_gates_command, run_governance_command,
    run_help_inventory_command, run_make_command, run_ops_command, run_perf_command,
    run_policies_command, run_print_boundaries_command, run_registry_check_by_id,
    run_registry_command, run_registry_contract_by_id, run_release_command, run_security_command,
    run_suites_command, run_version_command, run_workflows_command,
};
use crate::{run_print_policies, CheckListOptions, CheckRunOptions};
use bijux_dev_atlas::adapters::cli::{
    command_inventory_markdown, command_inventory_payload, route_name,
};
use bijux_dev_atlas::contracts;
use bijux_dev_atlas::domains;
use bijux_dev_atlas::engine;
use bijux_dev_atlas::model::exit_codes::{EXIT_FAILURE, EXIT_NOT_FOUND, EXIT_SUCCESS};
use bijux_dev_atlas::model::engine::{
    CaseStatus, ContractMode as EngineContractMode, Mode as EngineMode, RunOptions, TestKind,
};
use bijux_dev_atlas::model::{RunnableId, RunnableKind, RunnableMode};
use bijux_dev_atlas::registry::{ContractModesFile, ReportRegistry, RunnableRegistry};
use bijux_dev_atlas::ui::terminal::nextest_style::{self, RenderOptions};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command as ProcessCommand;

use crate::cli::dispatch_mutations::{apply_fail_fast, force_json_output, propagate_repo_root};
use crate::resolve_repo_root;
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
        Command::Contracts { command } => {
            if !cli.no_deprecation_warn {
                let _ = writeln!(io::stderr(), "{}", deprecated_contracts_warning());
            }
            run_contracts_command(cli.quiet, command)
        }
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
                    check_id,
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
                } => {
                    if let Some(check_id) = check_id {
                        run_registry_check_by_id(
                            repo_root,
                            artifacts_root,
                            run_id,
                            check_id,
                            fail_fast,
                            format,
                            out,
                        )
                    } else {
                        run_check_run(CheckRunOptions {
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
                        })
                    }
                }
                CheckCommand::TreeBudgets {
                    repo_root,
                    format,
                    out,
                } => run_check_tree_budgets(repo_root, format, out),
                CheckCommand::RepoDoctor {
                    repo_root,
                    format,
                    json,
                    explain,
                    out,
                } => run_check_repo_doctor(
                    repo_root,
                    if json { FormatArg::Json } else { format },
                    explain,
                    out,
                ),
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
        Command::Contract { command } => match run_contract_command(cli.output_format, command) {
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
                let _ = writeln!(io::stderr(), "bijux-dev-atlas contract failed: {err}");
                EXIT_FAILURE
            }
        },
        Command::Registry { command } => run_registry_command(cli.quiet, command),
        Command::Suites { command } => run_suites_command(cli.quiet, command),
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

pub(crate) fn deprecated_contracts_warning() -> &'static str {
    "bijux-dev-atlas: `contracts` is deprecated; use `contract` instead"
}

fn effective_format(global: Option<GlobalFormatArg>, local: FormatArg) -> GlobalFormatArg {
    global.unwrap_or(match local {
        FormatArg::Text => GlobalFormatArg::Human,
        FormatArg::Json | FormatArg::Jsonl => GlobalFormatArg::Json,
    })
}

struct ContractDomainSource {
    name: &'static str,
    load: fn(&Path) -> Result<Vec<contracts::Contract>, String>,
}

fn contract_domain_sources() -> [ContractDomainSource; 10] {
    [
        ContractDomainSource {
            name: "root",
            load: contracts::root::contracts,
        },
        ContractDomainSource {
            name: "repo",
            load: contracts::repo::contracts,
        },
        ContractDomainSource {
            name: "crates",
            load: contracts::crates::contracts,
        },
        ContractDomainSource {
            name: "runtime",
            load: contracts::runtime::contracts,
        },
        ContractDomainSource {
            name: "control_plane",
            load: contracts::control_plane::contracts,
        },
        ContractDomainSource {
            name: "configs",
            load: contracts::configs::contracts,
        },
        ContractDomainSource {
            name: "docs",
            load: contracts::docs::contracts,
        },
        ContractDomainSource {
            name: "docker",
            load: contracts::docker::contracts,
        },
        ContractDomainSource {
            name: "make",
            load: contracts::make::contracts,
        },
        ContractDomainSource {
            name: "ops",
            load: contracts::ops::contracts,
        },
    ]
}

fn contract_mode_matches_engine(mode: ContractRunModeArg, contract_mode: EngineContractMode) -> bool {
    match mode {
        ContractRunModeArg::Static => {
            matches!(contract_mode, EngineContractMode::Static | EngineContractMode::Both)
        }
        ContractRunModeArg::Effect => {
            matches!(contract_mode, EngineContractMode::Effect | EngineContractMode::Both)
        }
        ContractRunModeArg::All => true,
    }
}

fn contract_effects_allowed(mode: ContractRunModeArg, policy: ContractEffectsPolicyArg) -> bool {
    !matches!(mode, ContractRunModeArg::Effect | ContractRunModeArg::All)
        || policy == ContractEffectsPolicyArg::Allow
}

fn collect_available_tools() -> BTreeSet<&'static str> {
    const COMMON_TOOLS: &[&str] = &[
        "cargo", "docker", "git", "helm", "jq", "just", "kind", "kubectl", "make", "mkdocs",
        "node", "npm", "python3", "rg", "rustc",
    ];
    COMMON_TOOLS
        .iter()
        .copied()
        .filter(|tool| ProcessCommand::new("sh")
            .args(["-lc", &format!("command -v {tool} >/dev/null 2>&1")])
            .status()
            .is_ok_and(|status| status.success()))
        .collect()
}

fn render_contract_list_human(rows: &[serde_json::Value]) -> String {
    rows.iter()
        .filter_map(|row| row.get("id").and_then(serde_json::Value::as_str))
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>()
        .join("\n")
}

fn load_contract_catalog(
    repo_root: &Path,
) -> Result<Vec<(&'static str, contracts::Contract, EngineContractMode)>, String> {
    let mut rows = Vec::new();
    for source in contract_domain_sources() {
        for contract in (source.load)(repo_root)? {
            let mode = engine::contract_mode(&contract);
            rows.push((source.name, contract, mode));
        }
    }
    rows.sort_by(|(left_domain, left_contract, _), (right_domain, right_contract, _)| {
        left_domain
            .cmp(right_domain)
            .then_with(|| left_contract.id.0.cmp(&right_contract.id.0))
    });
    Ok(rows)
}

fn write_output(path: Option<std::path::PathBuf>, rendered: &str) -> Result<(), String> {
    if let Some(path) = path {
        std::fs::write(&path, format!("{rendered}\n"))
            .map_err(|err| format!("write {} failed: {err}", path.display()))?;
    }
    Ok(())
}

fn run_contract_command(
    global: Option<GlobalFormatArg>,
    command: ContractCommand,
) -> Result<(String, i32), String> {
    match command {
        ContractCommand::List(args) => run_contract_list_command(global, args),
        ContractCommand::Describe(args) => run_contract_describe_command(global, args),
        ContractCommand::Run(args) => run_contract_run_command(global, args),
    }
}

fn run_contract_list_command(
    global: Option<GlobalFormatArg>,
    args: crate::cli::ContractListArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let validation = ContractModesFile::validate(&root)?;
    let rows = load_contract_catalog(&root)?
        .into_iter()
        .filter(|(_, _, mode)| contract_mode_matches_engine(args.mode, *mode))
        .map(|(domain, contract, mode)| {
            let tags = match mode {
                EngineContractMode::Static => vec!["ci", "static"],
                EngineContractMode::Effect => vec!["ci", "effect", "slow"],
                EngineContractMode::Both => vec!["ci", "static", "effect", "slow"],
            };
            serde_json::json!({
                "id": contract.id.0,
                "mode": match mode {
                    EngineContractMode::Static => "static",
                    EngineContractMode::Effect => "effect",
                    EngineContractMode::Both => "all",
                },
                "domain": domain,
                "summary": contract.title,
                "required_tools": Vec::<String>::new(),
                "tags": tags,
            })
        })
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "contract_list",
        "mode": format!("{:?}", args.mode).to_ascii_lowercase(),
        "summary": {
            "contracts": rows.len(),
            "validation_errors": validation.errors.len(),
        },
        "contracts": rows,
        "validation_errors": validation.errors,
    });
    let rendered = match effective_format(global, args.format) {
        GlobalFormatArg::Human => render_contract_list_human(&rows),
        GlobalFormatArg::Json => serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("encode contract list failed: {err}"))?,
        GlobalFormatArg::Both => format!(
            "{}\n{}",
            render_contract_list_human(&rows),
            serde_json::to_string_pretty(&payload)
                .map_err(|err| format!("encode contract list failed: {err}"))?
        ),
    };
    write_output(args.out, &rendered)?;
    Ok((
        rendered,
        if validation.errors.is_empty() {
            EXIT_SUCCESS
        } else {
            EXIT_FAILURE
        },
    ))
}

fn run_contract_describe_command(
    global: Option<GlobalFormatArg>,
    args: crate::cli::ContractDescribeArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let validation = ContractModesFile::validate(&root)?;
    let Some((domain, contract, mode)) = load_contract_catalog(&root)?
        .into_iter()
        .find(|(_, contract, _)| contract.id.0 == args.id)
    else {
        return Ok((format!("unknown contract `{}`", args.id), EXIT_NOT_FOUND));
    };
    let payload = serde_json::json!({
        "schema_version": 1,
        "kind": "contract_description",
        "id": contract.id.0,
        "summary": contract.title,
        "mode": match mode {
            EngineContractMode::Static => "static",
            EngineContractMode::Effect => "effect",
            EngineContractMode::Both => "all",
        },
        "domain": domain,
        "outputs": Vec::<String>::new(),
        "effect_scope": contract.tests.iter().filter_map(|case| match case.kind {
            TestKind::Pure => None,
            TestKind::Subprocess => Some("subprocess"),
            TestKind::Network => Some("network"),
        }).collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>(),
        "tags": match mode {
            EngineContractMode::Static => vec!["ci", "static"],
            EngineContractMode::Effect => vec!["ci", "effect", "slow"],
            EngineContractMode::Both => vec!["ci", "static", "effect", "slow"],
        },
        "required_tools": Vec::<String>::new(),
        "validation_errors": validation.errors,
    });
    let rendered = match effective_format(global, args.format) {
        GlobalFormatArg::Human => format!(
            "{}\nsummary: {}\nmode: {}\ndomain: {}\noutputs: {}\neffect-scope: {}",
            payload["id"].as_str().unwrap_or_default(),
            payload["summary"].as_str().unwrap_or_default(),
            payload["mode"].as_str().unwrap_or_default(),
            payload["domain"].as_str().unwrap_or_default(),
            payload["outputs"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(serde_json::Value::as_str)
                .collect::<Vec<_>>()
                .join(", "),
            payload["effect_scope"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(serde_json::Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        GlobalFormatArg::Json => serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("encode contract description failed: {err}"))?,
        GlobalFormatArg::Both => format!(
            "{}\n{}",
            payload["id"].as_str().unwrap_or_default(),
            serde_json::to_string_pretty(&payload)
                .map_err(|err| format!("encode contract description failed: {err}"))?
        ),
    };
    write_output(args.out, &rendered)?;
    Ok((
        rendered,
        if validation.errors.is_empty() {
            EXIT_SUCCESS
        } else {
            EXIT_FAILURE
        },
    ))
}

fn run_contract_run_command(
    global: Option<GlobalFormatArg>,
    args: crate::cli::ContractRunArgs,
) -> Result<(String, i32), String> {
    let root = resolve_repo_root(args.repo_root)?;
    let validation = ContractModesFile::validate(&root)?;
    if !validation.errors.is_empty() {
        return Ok((validation.errors.join("\n"), EXIT_FAILURE));
    }

    let allowed_effects = contract_effects_allowed(args.mode, args.effects_policy);
    let domain_filter = args
        .domains
        .iter()
        .map(|value| value.trim().to_ascii_lowercase())
        .collect::<BTreeSet<_>>();
    let include = args
        .include
        .iter()
        .map(|value| value.trim().to_string())
        .collect::<BTreeSet<_>>();
    let exclude = args
        .exclude
        .iter()
        .map(|value| value.trim().to_string())
        .collect::<BTreeSet<_>>();
    let direct_id = args.id.as_deref().map(str::trim).filter(|value| !value.is_empty());

    let selected_ids = load_contract_catalog(&root)?
        .into_iter()
        .filter(|(domain, contract, mode)| {
            direct_id.is_none_or(|id| contract.id.0 == id)
                && (include.is_empty() || include.contains(contract.id.0.as_str()))
                && !exclude.contains(contract.id.0.as_str())
                && contract_mode_matches_engine(args.mode, *mode)
                && (domain_filter.is_empty() || domain_filter.contains(&domain.to_string()))
                && (args.tags.is_empty()
                    || args.tags.iter().any(|tag| match tag.as_str() {
                        "static" => matches!(
                            mode,
                            EngineContractMode::Static | EngineContractMode::Both
                        ),
                        "effect" | "slow" => matches!(
                            mode,
                            EngineContractMode::Effect | EngineContractMode::Both
                        ),
                        "ci" | "contracts" => true,
                        _ => false,
                    }))
        })
        .map(|(_, contract, _)| contract.id.0)
        .collect::<BTreeSet<_>>();

    if selected_ids.is_empty() {
        return Ok(("no contracts matched the requested selection".to_string(), EXIT_NOT_FOUND));
    }

    let _available_tools = collect_available_tools();
    let missing_tool_ids = BTreeMap::<String, Vec<String>>::new();

    let mut reports = Vec::new();
    let mut planned_cases = 0usize;
    let fail_fast = args.fail_fast && !args.no_fail_fast;
    for source in contract_domain_sources() {
        let loaded = (source.load)(&root)?;
        let selected = loaded
            .into_iter()
            .filter(|contract| selected_ids.contains(contract.id.0.as_str()))
            .collect::<Vec<_>>();
        if selected.is_empty() {
            continue;
        }
        let options = RunOptions {
            lane: contracts::ContractLane::Local,
            mode: match args.mode {
                ContractRunModeArg::Effect => EngineMode::Effect,
                ContractRunModeArg::Static => EngineMode::Static,
                ContractRunModeArg::All => EngineMode::Effect,
            },
            run_id: args.run_id.clone(),
            required_only: false,
            ci: false,
            color_enabled: match args.color {
                ContractsColorArg::Auto => !args.no_ansi,
                ContractsColorArg::Always => !args.no_ansi,
                ContractsColorArg::Never => false,
            },
            allow_subprocess: allowed_effects,
            allow_network: allowed_effects,
            allow_k8s: false,
            allow_fs_write: true,
            allow_docker_daemon: false,
            deny_skip_required: false,
            skip_missing_tools: true,
            timeout_seconds: 300,
            fail_fast,
            contract_filter: None,
            test_filter: None,
            only_contracts: Vec::new(),
            only_tests: Vec::new(),
            skip_contracts: Vec::new(),
            tags: Vec::new(),
            list_only: false,
            artifacts_root: args.artifacts_root.clone(),
        };
        let mut report = engine::run_selected(source.name, selected, &root, &options)?;
        planned_cases += report.cases.len();
        for case in &mut report.cases {
            if let Some(missing) = missing_tool_ids.get(&case.contract_id) {
                case.status = CaseStatus::Skip;
                case.note = Some(format!("missing tool: {}", missing.join(", ")));
            } else if matches!(args.mode, ContractRunModeArg::Static)
                && case.kind != TestKind::Pure
            {
                case.status = CaseStatus::Skip;
                case.note = Some("disabled effect policy".to_string());
            } else if !allowed_effects && case.kind != TestKind::Pure {
                case.status = CaseStatus::Skip;
                case.note = Some("disabled effect policy".to_string());
            }
        }
        reports.push(report);
        if fail_fast
            && reports.iter().any(|report| report.fail_count() > 0 || report.error_count() > 0)
        {
            break;
        }
    }

    let payload = engine::to_json_all(&reports);
    let rendered = match effective_format(global, args.format) {
        GlobalFormatArg::Human => nextest_style::render(
            &reports,
            match args.mode {
                ContractRunModeArg::Static => "static",
                ContractRunModeArg::Effect => "effect",
                ContractRunModeArg::All => "all",
            },
            &args.jobs,
            fail_fast,
            RenderOptions {
                color: match args.color {
                    ContractsColorArg::Auto => !args.no_ansi,
                    ContractsColorArg::Always => !args.no_ansi,
                    ContractsColorArg::Never => false,
                },
                quiet: args.quiet,
                verbose: args.verbose,
            },
        ),
        GlobalFormatArg::Json => serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("encode contract run summary failed: {err}"))?,
        GlobalFormatArg::Both => format!(
            "{}\n{}",
            nextest_style::render(
                &reports,
                match args.mode {
                    ContractRunModeArg::Static => "static",
                    ContractRunModeArg::Effect => "effect",
                    ContractRunModeArg::All => "all",
                },
                &args.jobs,
                fail_fast,
                RenderOptions {
                    color: false,
                    quiet: args.quiet,
                    verbose: args.verbose,
                },
            ),
            serde_json::to_string_pretty(&payload)
                .map_err(|err| format!("encode contract run summary failed: {err}"))?
        ),
    };
    let _ = planned_cases;
    write_output(args.out, &rendered)?;
    let exit = reports.iter().map(|report| report.exit_code()).max().unwrap_or(EXIT_SUCCESS);
    Ok((rendered, exit))
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
