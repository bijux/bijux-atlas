// SPDX-License-Identifier: Apache-2.0
//! `cli` defines argument parsing and command-surface types.
//!
//! Boundary: `cli` parses/normalizes user input and dispatches to command handlers; business logic
//! belongs in `commands`/`core`.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

mod checks;
mod dispatch;
mod dispatch_mutations;
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
    #[command(
        after_help = "Ops Pillars And Docs Entrypoints:\n  inventory -> docs/operations/reference/ops-surface.md\n  schema -> docs/operations/reference/schema-index.md\n  datasets -> docs/operations/datasets.md\n  e2e -> docs/operations/e2e/index.md\n  env -> docs/operations/config.md\n  stack -> docs/operations/local-stack.md\n  k8s -> docs/operations/k8s/index.md\n  load -> docs/operations/load/index.md\n  observe -> docs/operations/observability/index.md\n  report -> docs/operations/unified-report.md"
    )]
    Ops {
        #[command(subcommand)]
        command: OpsCommand,
    },
    Docs {
        #[command(subcommand)]
        command: DocsCommand,
    },
    #[command(hide = true)]
    Artifacts {
        #[command(subcommand)]
        command: ArtifactsCommand,
    },
    #[command(hide = true)]
    Make {
        #[command(subcommand)]
        command: MakeCommand,
    },
    Contracts {
        #[command(subcommand)]
        command: ContractsCommand,
    },
    Demo {
        #[command(subcommand)]
        command: DemoCommand,
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
    Ci {
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
    Validate {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, default_value = "kind")]
        profile: String,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    #[command(hide = true)]
    Release {
        #[command(subcommand)]
        command: ReleaseCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum ReleaseCommand {
    Check(ReleaseCheckArgs),
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseCheckArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, default_value = "kind")]
    pub profile: String,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}
