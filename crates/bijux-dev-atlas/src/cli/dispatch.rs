// SPDX-License-Identifier: Apache-2.0

use crate::cli::{
    CheckCommand, CheckRegistryCommand, Cli, Command, ConfigsCommand, DocsCommand, FormatArg,
    OpsCommand, PoliciesCommand,
};
use crate::{
    plugin_metadata_json, run_build_command, run_capabilities_command, run_check_doctor,
    run_check_explain, run_check_list, run_check_registry_doctor, run_check_run,
    run_configs_command, run_demo_command, run_docker_command, run_docs_command, run_gates_command,
    run_help_inventory_command, run_ops_command, run_policies_command,
    run_print_boundaries_command, run_version_command, run_workflows_command,
};
use crate::{run_print_policies, CheckListOptions, CheckRunOptions};
use std::io::{self, Write};

fn force_json_output(command: &mut Command) {
    match command {
        Command::Version { format, .. } => *format = FormatArg::Json,
        Command::Help { format, .. } => *format = FormatArg::Json,
        Command::Ops { command } => force_json_ops(command),
        Command::Docs { command } => force_json_docs(command),
        Command::Demo { command } => force_json_demo(command),
        Command::Configs { command } => force_json_configs(command),
        Command::Policies { command } => force_json_policies(command),
        Command::Check { command } => force_json_check(command),
        Command::Docker { .. }
        | Command::Build { .. }
        | Command::Workflows { .. }
        | Command::Gates { .. }
        | Command::Capabilities { .. } => {}
    }
}

fn force_json_demo(command: &mut crate::cli::DemoCommand) {
    match command {
        crate::cli::DemoCommand::Quickstart(args) => args.format = FormatArg::Json,
    }
}

fn force_json_ops(command: &mut OpsCommand) {
    match command {
        OpsCommand::List(common)
        | OpsCommand::Doctor(common)
        | OpsCommand::Validate(common)
        | OpsCommand::Inventory(common)
        | OpsCommand::Docs(common)
        | OpsCommand::Conformance(common)
        | OpsCommand::Report(common)
        | OpsCommand::ListProfiles(common)
        | OpsCommand::ListTools(common)
        | OpsCommand::VerifyTools(common)
        | OpsCommand::ListActions(common)
        | OpsCommand::Plan(common)
        | OpsCommand::Up(common)
        | OpsCommand::Down(common)
        | OpsCommand::Clean(common)
        | OpsCommand::Cleanup(common)
        | OpsCommand::K8sPlan(common)
        | OpsCommand::K8sDryRun(common)
        | OpsCommand::K8sConformance(common)
        | OpsCommand::LoadPlan { common, .. }
        | OpsCommand::LoadRun { common, .. }
        | OpsCommand::LoadReport { common, .. } => common.format = FormatArg::Json,
        OpsCommand::Explain { common, .. } => common.format = FormatArg::Json,
        OpsCommand::Render(args) => args.common.format = FormatArg::Json,
        OpsCommand::Install(args) => args.common.format = FormatArg::Json,
        OpsCommand::Status(args) => args.common.format = FormatArg::Json,
        OpsCommand::ExplainProfile { common, .. } => common.format = FormatArg::Json,
        OpsCommand::Reset(args) => args.common.format = FormatArg::Json,
        OpsCommand::K8sApply(args) => args.common.format = FormatArg::Json,
        OpsCommand::K8sWait(args) => args.common.format = FormatArg::Json,
        OpsCommand::K8sLogs(args) => args.common.format = FormatArg::Json,
        OpsCommand::K8sPortForward(args) => args.common.format = FormatArg::Json,
        OpsCommand::Pins { command } => match command {
            crate::cli::OpsPinsCommand::Check(common)
            | crate::cli::OpsPinsCommand::Update { common, .. } => common.format = FormatArg::Json,
        },
        OpsCommand::Generate { command } => match command {
            crate::cli::OpsGenerateCommand::PinsIndex { common, .. } => {
                common.format = FormatArg::Json
            }
        },
        OpsCommand::Tools { command } => match command {
            crate::cli::OpsToolsCommand::List(common)
            | crate::cli::OpsToolsCommand::Verify(common)
            | crate::cli::OpsToolsCommand::Doctor(common) => common.format = FormatArg::Json,
        },
        OpsCommand::Suite { command } => match command {
            crate::cli::OpsSuiteCommand::List(common)
            | crate::cli::OpsSuiteCommand::Run { common, .. } => common.format = FormatArg::Json,
        },
        OpsCommand::Stack { command } => match command {
            crate::cli::OpsStackCommand::Plan(common)
            | crate::cli::OpsStackCommand::Up(common)
            | crate::cli::OpsStackCommand::Down(common) => common.format = FormatArg::Json,
            crate::cli::OpsStackCommand::Status(args) => args.common.format = FormatArg::Json,
            crate::cli::OpsStackCommand::Reset(args) => args.common.format = FormatArg::Json,
        },
        OpsCommand::K8s { command } => match command {
            crate::cli::OpsK8sCommand::Render(args) => args.common.format = FormatArg::Json,
            crate::cli::OpsK8sCommand::Plan(common)
            | crate::cli::OpsK8sCommand::DryRun(common)
            | crate::cli::OpsK8sCommand::Conformance(common)
            | crate::cli::OpsK8sCommand::Test(common) => common.format = FormatArg::Json,
            crate::cli::OpsK8sCommand::Apply(args) => args.common.format = FormatArg::Json,
            crate::cli::OpsK8sCommand::Wait(args) => args.common.format = FormatArg::Json,
            crate::cli::OpsK8sCommand::Logs(args) => args.common.format = FormatArg::Json,
            crate::cli::OpsK8sCommand::PortForward(args) => args.common.format = FormatArg::Json,
            crate::cli::OpsK8sCommand::Status(args) => args.common.format = FormatArg::Json,
        },
        OpsCommand::Load { command } => match command {
            crate::cli::OpsLoadCommand::Plan { common, .. }
            | crate::cli::OpsLoadCommand::Run { common, .. }
            | crate::cli::OpsLoadCommand::Report { common, .. } => common.format = FormatArg::Json,
        },
        OpsCommand::E2e { command } => match command {
            crate::cli::OpsE2eCommand::Run(common) => common.format = FormatArg::Json,
        },
        OpsCommand::Obs { command } => match command {
            crate::cli::OpsObsCommand::Drill { command } => match command {
                crate::cli::OpsObsDrillCommand::Run(common) => common.format = FormatArg::Json,
            },
            crate::cli::OpsObsCommand::Verify(common) => common.format = FormatArg::Json,
        },
    }
}

fn force_json_docs(command: &mut DocsCommand) {
    match command {
        DocsCommand::Check(common)
        | DocsCommand::VerifyContracts(common)
        | DocsCommand::Doctor(common)
        | DocsCommand::Validate(common)
        | DocsCommand::Build(common)
        | DocsCommand::Clean(common)
        | DocsCommand::Lint(common)
        | DocsCommand::Links(common)
        | DocsCommand::Inventory(common) => common.format = FormatArg::Json,
        DocsCommand::Serve(args) => args.common.format = FormatArg::Json,
        DocsCommand::Grep(args) => args.common.format = FormatArg::Json,
        DocsCommand::Registry { command } => match command {
            crate::cli::DocsRegistryCommand::Build(common)
            | crate::cli::DocsRegistryCommand::Validate(common) => common.format = FormatArg::Json,
        },
        DocsCommand::Reference { command } => match command {
            crate::cli::DocsReferenceCommand::Generate(common)
            | crate::cli::DocsReferenceCommand::Check(common) => common.format = FormatArg::Json,
        },
    }
}

fn force_json_configs(command: &mut ConfigsCommand) {
    match command {
        ConfigsCommand::Print(common)
        | ConfigsCommand::List(common)
        | ConfigsCommand::Doctor(common)
        | ConfigsCommand::Validate(common)
        | ConfigsCommand::Lint(common)
        | ConfigsCommand::Inventory(common)
        | ConfigsCommand::Compile(common)
        | ConfigsCommand::Diff(common) => common.format = FormatArg::Json,
        ConfigsCommand::Fmt { common, .. } => common.format = FormatArg::Json,
    }
}

fn force_json_policies(command: &mut PoliciesCommand) {
    match command {
        PoliciesCommand::List { format, .. }
        | PoliciesCommand::Explain { format, .. }
        | PoliciesCommand::Report { format, .. }
        | PoliciesCommand::Print { format, .. }
        | PoliciesCommand::Validate { format, .. } => *format = FormatArg::Json,
    }
}

fn force_json_check(command: &mut CheckCommand) {
    match command {
        CheckCommand::Registry { command } => match command {
            CheckRegistryCommand::Doctor { format, .. } => *format = FormatArg::Json,
        },
        CheckCommand::List { format, json, .. } => {
            *format = FormatArg::Json;
            *json = true;
        }
        CheckCommand::Explain { format, .. }
        | CheckCommand::Doctor { format, .. }
        | CheckCommand::Run { format, .. } => *format = FormatArg::Json,
    }
}

fn apply_fail_fast(command: &mut Command) {
    match command {
        Command::Check {
            command: CheckCommand::Run { fail_fast, .. },
        } => *fail_fast = true,
        Command::Docs { command } => match command {
            DocsCommand::Check(common)
            | DocsCommand::Doctor(common)
            | DocsCommand::Validate(common)
            | DocsCommand::Lint(common)
            | DocsCommand::Links(common)
            | DocsCommand::VerifyContracts(common) => common.strict = true,
            DocsCommand::Build(_)
            | DocsCommand::Serve(_)
            | DocsCommand::Clean(_)
            | DocsCommand::Inventory(_)
            | DocsCommand::Grep(_) => {}
            DocsCommand::Registry { command } => match command {
                crate::cli::DocsRegistryCommand::Build(_)
                | crate::cli::DocsRegistryCommand::Validate(_) => {}
            },
            DocsCommand::Reference { command } => match command {
                crate::cli::DocsReferenceCommand::Generate(_)
                | crate::cli::DocsReferenceCommand::Check(_) => {}
            },
        },
        Command::Configs { command } => match command {
            ConfigsCommand::Doctor(common)
            | ConfigsCommand::Validate(common)
            | ConfigsCommand::Lint(common)
            | ConfigsCommand::Inventory(common)
            | ConfigsCommand::Diff(common) => common.strict = true,
            ConfigsCommand::Fmt { check, .. } => *check = true,
            ConfigsCommand::Print(_) | ConfigsCommand::List(_) | ConfigsCommand::Compile(_) => {}
        },
        _ => {}
    }
}

fn propagate_repo_root(command: &mut Command, repo_root: Option<std::path::PathBuf>) {
    let Some(root) = repo_root else {
        return;
    };
    match command {
        Command::Ops { command } => match command {
            OpsCommand::List(common)
            | OpsCommand::Doctor(common)
            | OpsCommand::Validate(common)
            | OpsCommand::Inventory(common)
            | OpsCommand::Docs(common)
            | OpsCommand::Conformance(common)
            | OpsCommand::Report(common)
            | OpsCommand::ListProfiles(common)
            | OpsCommand::ListTools(common)
            | OpsCommand::VerifyTools(common)
            | OpsCommand::ListActions(common)
            | OpsCommand::Plan(common)
            | OpsCommand::Up(common)
            | OpsCommand::Down(common)
            | OpsCommand::Clean(common)
            | OpsCommand::Cleanup(common)
            | OpsCommand::K8sPlan(common)
            | OpsCommand::K8sDryRun(common)
            | OpsCommand::K8sConformance(common) => common.repo_root = Some(root.clone()),
            OpsCommand::LoadPlan { common, .. }
            | OpsCommand::LoadRun { common, .. }
            | OpsCommand::LoadReport { common, .. } => common.repo_root = Some(root.clone()),
            OpsCommand::Explain { common, .. } => common.repo_root = Some(root.clone()),
            OpsCommand::Render(args) => args.common.repo_root = Some(root.clone()),
            OpsCommand::Install(args) => args.common.repo_root = Some(root.clone()),
            OpsCommand::Status(args) => args.common.repo_root = Some(root.clone()),
            OpsCommand::ExplainProfile { common, .. } => common.repo_root = Some(root.clone()),
            OpsCommand::Reset(args) => args.common.repo_root = Some(root.clone()),
            OpsCommand::K8sApply(args) => args.common.repo_root = Some(root.clone()),
            OpsCommand::K8sWait(args) => args.common.repo_root = Some(root.clone()),
            OpsCommand::K8sLogs(args) => args.common.repo_root = Some(root.clone()),
            OpsCommand::K8sPortForward(args) => args.common.repo_root = Some(root.clone()),
            OpsCommand::Pins { command } => match command {
                crate::cli::OpsPinsCommand::Check(common)
                | crate::cli::OpsPinsCommand::Update { common, .. } => {
                    common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::Generate { command } => match command {
                crate::cli::OpsGenerateCommand::PinsIndex { common, .. } => {
                    common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::Tools { command } => match command {
                crate::cli::OpsToolsCommand::List(common)
                | crate::cli::OpsToolsCommand::Verify(common)
                | crate::cli::OpsToolsCommand::Doctor(common) => {
                    common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::Suite { command } => match command {
                crate::cli::OpsSuiteCommand::List(common)
                | crate::cli::OpsSuiteCommand::Run { common, .. } => {
                    common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::Stack { command } => match command {
                crate::cli::OpsStackCommand::Plan(common)
                | crate::cli::OpsStackCommand::Up(common)
                | crate::cli::OpsStackCommand::Down(common) => {
                    common.repo_root = Some(root.clone())
                }
                crate::cli::OpsStackCommand::Status(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::OpsStackCommand::Reset(args) => {
                    args.common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::K8s { command } => match command {
                crate::cli::OpsK8sCommand::Render(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::OpsK8sCommand::Plan(common)
                | crate::cli::OpsK8sCommand::DryRun(common)
                | crate::cli::OpsK8sCommand::Conformance(common)
                | crate::cli::OpsK8sCommand::Test(common) => common.repo_root = Some(root.clone()),
                crate::cli::OpsK8sCommand::Apply(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::OpsK8sCommand::Wait(args) => args.common.repo_root = Some(root.clone()),
                crate::cli::OpsK8sCommand::Logs(args) => args.common.repo_root = Some(root.clone()),
                crate::cli::OpsK8sCommand::PortForward(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::OpsK8sCommand::Status(args) => {
                    args.common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::Load { command } => match command {
                crate::cli::OpsLoadCommand::Plan { common, .. }
                | crate::cli::OpsLoadCommand::Run { common, .. }
                | crate::cli::OpsLoadCommand::Report { common, .. } => {
                    common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::E2e { command } => match command {
                crate::cli::OpsE2eCommand::Run(common) => common.repo_root = Some(root.clone()),
            },
            OpsCommand::Obs { command } => match command {
                crate::cli::OpsObsCommand::Drill { command } => match command {
                    crate::cli::OpsObsDrillCommand::Run(common) => {
                        common.repo_root = Some(root.clone())
                    }
                },
                crate::cli::OpsObsCommand::Verify(common) => common.repo_root = Some(root.clone()),
            },
        },
        Command::Docs { command } => match command {
            DocsCommand::Check(common)
            | DocsCommand::VerifyContracts(common)
            | DocsCommand::Doctor(common)
            | DocsCommand::Validate(common)
            | DocsCommand::Build(common)
            | DocsCommand::Clean(common)
            | DocsCommand::Lint(common)
            | DocsCommand::Links(common)
            | DocsCommand::Inventory(common) => common.repo_root = Some(root.clone()),
            DocsCommand::Serve(args) => args.common.repo_root = Some(root.clone()),
            DocsCommand::Grep(args) => args.common.repo_root = Some(root.clone()),
            DocsCommand::Registry { command } => match command {
                crate::cli::DocsRegistryCommand::Build(common)
                | crate::cli::DocsRegistryCommand::Validate(common) => {
                    common.repo_root = Some(root.clone())
                }
            },
            DocsCommand::Reference { command } => match command {
                crate::cli::DocsReferenceCommand::Generate(common)
                | crate::cli::DocsReferenceCommand::Check(common) => {
                    common.repo_root = Some(root.clone())
                }
            },
        },
        Command::Configs { command } => match command {
            ConfigsCommand::Print(common)
            | ConfigsCommand::List(common)
            | ConfigsCommand::Doctor(common)
            | ConfigsCommand::Validate(common)
            | ConfigsCommand::Lint(common)
            | ConfigsCommand::Inventory(common)
            | ConfigsCommand::Compile(common)
            | ConfigsCommand::Diff(common) => common.repo_root = Some(root.clone()),
            ConfigsCommand::Fmt { common, .. } => common.repo_root = Some(root.clone()),
        },
        Command::Policies { command } => match command {
            PoliciesCommand::List { repo_root, .. }
            | PoliciesCommand::Explain { repo_root, .. }
            | PoliciesCommand::Report { repo_root, .. }
            | PoliciesCommand::Print { repo_root, .. }
            | PoliciesCommand::Validate { repo_root, .. } => *repo_root = Some(root.clone()),
        },
        Command::Check { command } => match command {
            CheckCommand::Registry { command } => match command {
                CheckRegistryCommand::Doctor { repo_root, .. } => *repo_root = Some(root.clone()),
            },
            CheckCommand::List { repo_root, .. }
            | CheckCommand::Explain { repo_root, .. }
            | CheckCommand::Doctor { repo_root, .. }
            | CheckCommand::Run { repo_root, .. } => *repo_root = Some(root.clone()),
        },
        Command::Demo { command } => match command {
            crate::cli::DemoCommand::Quickstart(args) => args.repo_root = Some(root.clone()),
        },
        Command::Version { .. }
        | Command::Help { .. }
        | Command::Docker { .. }
        | Command::Build { .. }
        | Command::Workflows { .. }
        | Command::Gates { .. }
        | Command::Capabilities { .. } => {}
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
        Command::Demo { command } => run_demo_command(cli.quiet, command),
        Command::Configs { command } => run_configs_command(cli.quiet, command),
        Command::Docker { command } => run_docker_command(cli.quiet, command),
        Command::Build { command } => run_build_command(cli.quiet, command),
        Command::Policies { command } => run_policies_command(cli.quiet, command),
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
