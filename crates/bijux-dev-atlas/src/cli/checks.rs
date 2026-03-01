// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use clap::{Subcommand, ValueEnum};

#[derive(Subcommand, Debug)]
pub enum CheckCommand {
    Registry {
        #[command(subcommand)]
        command: CheckRegistryCommand,
    },
    List {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_name = "required|ci-fast|ci|local|deep|<suite_id>")]
        suite: Option<String>,
        #[arg(long, value_enum)]
        domain: Option<DomainArg>,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long, value_name = "GLOB")]
        id: Option<String>,
        #[arg(long, default_value_t = false)]
        include_internal: bool,
        #[arg(long, default_value_t = false)]
        include_slow: bool,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long, default_value_t = false)]
        json: bool,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Explain {
        check_id: String,
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Doctor {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        include_internal: bool,
        #[arg(long, default_value_t = false)]
        include_slow: bool,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Run {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long)]
        artifacts_root: Option<PathBuf>,
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long, value_name = "required|ci-fast|ci|local|deep|<suite_id>")]
        suite: Option<String>,
        #[arg(long, value_enum)]
        domain: Option<DomainArg>,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long, value_name = "GLOB")]
        id: Option<String>,
        #[arg(long, default_value_t = false)]
        include_internal: bool,
        #[arg(long, default_value_t = false)]
        include_slow: bool,
        #[arg(long, default_value_t = false)]
        allow_subprocess: bool,
        #[arg(long, default_value_t = false)]
        allow_git: bool,
        #[arg(long = "allow-write", default_value_t = false)]
        allow_write: bool,
        #[arg(long, default_value_t = false)]
        allow_network: bool,
        #[arg(long, default_value_t = false)]
        fail_fast: bool,
        #[arg(long)]
        max_failures: Option<usize>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, default_value_t = 0)]
        durations: usize,
    },
    TreeBudgets {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    RepoDoctor {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    RootSurfaceExplain {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
pub enum CheckRegistryCommand {
    Doctor {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
pub enum WorkflowsCommand {
    Validate {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        include_internal: bool,
        #[arg(long, default_value_t = false)]
        include_slow: bool,
    },
    Doctor {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        include_internal: bool,
        #[arg(long, default_value_t = false)]
        include_slow: bool,
    },
    Surface {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        include_internal: bool,
        #[arg(long, default_value_t = false)]
        include_slow: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum PoliciesCommand {
    List {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Json)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Explain {
        policy_id: String,
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Json)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Report {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Json)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Print {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Json)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Validate {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Json)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
pub enum GatesCommand {
    List {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        include_internal: bool,
        #[arg(long, default_value_t = false)]
        include_slow: bool,
    },
    Run {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long)]
        artifacts_root: Option<PathBuf>,
        #[arg(long)]
        run_id: Option<String>,
        #[arg(
            long,
            value_name = "required|ci-fast|ci|local|deep|<suite_id>",
            default_value = "ci_fast"
        )]
        suite: String,
        #[arg(long, default_value_t = false)]
        include_internal: bool,
        #[arg(long, default_value_t = false)]
        include_slow: bool,
        #[arg(long, default_value_t = false)]
        allow_subprocess: bool,
        #[arg(long, default_value_t = false)]
        allow_git: bool,
        #[arg(long = "allow-write", default_value_t = false)]
        allow_write: bool,
        #[arg(long, default_value_t = false)]
        allow_network: bool,
        #[arg(long, default_value_t = false)]
        fail_fast: bool,
        #[arg(long)]
        max_failures: Option<usize>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, default_value_t = 0)]
        durations: usize,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum DomainArg {
    Root,
    Workflows,
    Configs,
    Docker,
    Crates,
    Ops,
    Repo,
    Docs,
    Make,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum FormatArg {
    Text,
    Json,
    Jsonl,
}
