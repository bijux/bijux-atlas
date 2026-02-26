// SPDX-License-Identifier: Apache-2.0
//! `cli` defines argument parsing and command-surface types.
//!
//! Boundary: `cli` parses/normalizes user input and dispatches to command handlers; business logic
//! belongs in `commands`/`core`.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod checks;
mod dispatch;
mod ops;
mod surfaces;

pub use checks::*;
pub use ops::*;
pub use surfaces::*;

pub(crate) fn run() -> i32 {
    let cli = Cli::parse();
    dispatch::run_cli(cli)
}

#[derive(Parser, Debug)]
#[command(name = "bijux-dev-atlas", version, disable_help_subcommand = true)]
#[command(about = "Bijux Atlas development control-plane")]
pub struct Cli {
    #[arg(long, default_value_t = false)]
    pub quiet: bool,
    #[arg(long, default_value_t = false)]
    pub json: bool,
    #[arg(long, default_value_t = false)]
    pub verbose: bool,
    #[arg(long, default_value_t = false)]
    pub debug: bool,
    #[arg(long = "fail-fast", default_value_t = false)]
    pub fail_fast: bool,
    #[arg(long, default_value_t = false)]
    pub print_policies: bool,
    #[arg(long = "print-boundaries", default_value_t = false)]
    pub print_boundaries: bool,
    #[arg(long = "bijux-plugin-metadata", default_value_t = false)]
    pub bijux_plugin_metadata: bool,
    #[arg(long = "umbrella-version")]
    pub umbrella_version: Option<String>,
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(hide = true)]
    Version {
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    #[command(hide = true)]
    Help {
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Ops {
        #[command(subcommand)]
        command: OpsCommand,
    },
    Docs {
        #[command(subcommand)]
        command: DocsCommand,
    },
    Configs {
        #[command(subcommand)]
        command: ConfigsCommand,
    },
    #[command(hide = true)]
    Docker {
        #[command(subcommand)]
        command: DockerCommand,
    },
    #[command(hide = true)]
    Build {
        #[command(subcommand)]
        command: BuildCommand,
    },
    Policies {
        #[command(subcommand)]
        command: PoliciesCommand,
    },
    #[command(hide = true)]
    Workflows {
        #[command(subcommand)]
        command: WorkflowsCommand,
    },
    #[command(hide = true)]
    Gates {
        #[command(subcommand)]
        command: GatesCommand,
    },
    #[command(hide = true)]
    Capabilities {
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Check {
        #[command(subcommand)]
        command: CheckCommand,
    },
}
