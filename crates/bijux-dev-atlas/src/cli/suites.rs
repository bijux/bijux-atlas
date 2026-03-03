// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use clap::{Subcommand, ValueEnum};

use super::FormatArg;

#[derive(Subcommand, Debug)]
pub enum ContractCommand {
    Run {
        contract_id: String,
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long)]
        artifacts_root: Option<PathBuf>,
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long, default_value_t = false)]
        fail_fast: bool,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
pub enum SuitesCommand {
    Run {
        #[arg(long)]
        suite: String,
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long)]
        artifacts_root: Option<PathBuf>,
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long, default_value = "auto")]
        jobs: String,
        #[arg(long, default_value_t = false)]
        fail_fast: bool,
        #[arg(long, default_value_t = false)]
        no_fail_fast: bool,
        #[arg(long, value_enum, default_value_t = SuiteModeArg::All)]
        mode: SuiteModeArg,
        #[arg(long)]
        group: Option<String>,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    List {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Describe {
        #[arg(long)]
        suite: String,
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum SuiteModeArg {
    Pure,
    Effect,
    All,
}
