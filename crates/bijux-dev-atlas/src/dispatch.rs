use crate::cli::{CheckCommand, CheckRegistryCommand, Cli, Command};
use crate::{
    plugin_metadata_json, run_capabilities_command, run_check_doctor, run_check_explain,
    run_check_list, run_check_registry_doctor, run_check_run, run_configs_command,
    run_docker_command, run_docs_command, run_gates_command, run_help_inventory_command,
    run_ops_command, run_policies_command, run_version_command, run_workflows_command,
};
use crate::{run_print_policies, CheckListOptions, CheckRunOptions};

pub(crate) fn run_cli(cli: Cli) -> i32 {
    if cli.bijux_plugin_metadata {
        println!("{}", plugin_metadata_json());
        return 0;
    }
    if cli.print_policies {
        return match run_print_policies(cli.repo_root.clone()) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    println!("{rendered}");
                }
                code
            }
            Err(err) => {
                eprintln!("bijux-dev-atlas --print-policies failed: {err}");
                1
            }
        };
    }

    let Some(command) = cli.command else {
        eprintln!("bijux-dev-atlas requires a subcommand unless --print-policies is provided");
        return 2;
    };

    let exit = match command {
        Command::Version { format, out } => match run_version_command(format, out) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    println!("{rendered}");
                }
                code
            }
            Err(err) => {
                eprintln!("bijux-dev-atlas version failed: {err}");
                1
            }
        },
        Command::Help { format, out } => match run_help_inventory_command(format, out) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    println!("{rendered}");
                }
                code
            }
            Err(err) => {
                eprintln!("bijux-dev-atlas help failed: {err}");
                1
            }
        },
        Command::Docs { command } => run_docs_command(cli.quiet, command),
        Command::Configs { command } => run_configs_command(cli.quiet, command),
        Command::Docker { command } => run_docker_command(cli.quiet, command),
        Command::Policies { command } => run_policies_command(cli.quiet, command),
        Command::Workflows { command } => run_workflows_command(cli.quiet, command),
        Command::Gates { command } => run_gates_command(cli.quiet, command),
        Command::Capabilities { format, out } => match run_capabilities_command(format, out) {
            Ok((rendered, code)) => {
                if !cli.quiet && !rendered.is_empty() {
                    println!("{rendered}");
                }
                code
            }
            Err(err) => {
                eprintln!("bijux-dev-atlas capabilities failed: {err}");
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
                    out,
                } => run_check_list(CheckListOptions {
                    repo_root,
                    suite,
                    domain,
                    tag,
                    id,
                    include_internal,
                    include_slow,
                    format,
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
                        println!("{rendered}");
                    }
                    code
                }
                Err(err) => {
                    eprintln!("bijux-dev-atlas check failed: {err}");
                    1
                }
            }
        }
        Command::Ops { command } => run_ops_command(cli.quiet, cli.debug, command),
    };

    if cli.verbose {
        eprintln!("bijux-dev-atlas exit={exit}");
    }
    exit
}
