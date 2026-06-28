// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use clap::{Subcommand, ValueEnum};

use crate::cli::FormatArg;

#[derive(Subcommand, Debug)]
pub enum TestsCommand {
    List {
        #[arg(long)]
        repo_root: Option<PathBuf>,
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
        #[arg(long, value_enum, default_value_t = TestsModeArg::Fast)]
        mode: TestsModeArg,
        #[arg(long, default_value_t = false)]
        fail_fast: bool,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Doctor {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long)]
        artifacts_root: Option<PathBuf>,
        #[arg(long)]
        run_id: Option<String>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(ValueEnum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum TestsModeArg {
    Fast,
    All,
}
