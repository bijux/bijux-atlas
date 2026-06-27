// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use clap::{Subcommand, ValueEnum};

use super::FormatArg;

#[derive(Subcommand, Debug)]
pub enum RegistryCommand {
    Status {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        missing: Option<RegistryMissingArg>,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Doctor {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        fix_suggestions: bool,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum RegistryMissingArg {
    Owner,
    Reports,
    SuiteMembership,
    Command,
    BrokenCommand,
}
