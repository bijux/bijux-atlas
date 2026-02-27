// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use clap::{Args, Subcommand};

use super::FormatArg;

#[derive(Subcommand, Debug)]
pub enum DemoCommand {
    Quickstart(DemoQuickstartArgs),
}

#[derive(Args, Debug, Clone)]
pub struct DemoQuickstartArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Json)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum DocsCommand {
    Check(DocsCommonArgs),
    VerifyContracts(DocsCommonArgs),
    Doctor(DocsCommonArgs),
    Validate(DocsCommonArgs),
    Build(DocsCommonArgs),
    Serve(DocsServeArgs),
    Clean(DocsCommonArgs),
    Lint(DocsCommonArgs),
    Links(DocsCommonArgs),
    Inventory(DocsCommonArgs),
    ShrinkReport(DocsCommonArgs),
    Grep(DocsGrepArgs),
    Reference {
        #[command(subcommand)]
        command: DocsReferenceCommand,
    },
    Registry {
        #[command(subcommand)]
        command: DocsRegistryCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum DocsRegistryCommand {
    Build(DocsCommonArgs),
    Validate(DocsCommonArgs),
}

#[derive(Subcommand, Debug)]
pub enum DocsReferenceCommand {
    Generate(DocsCommonArgs),
    Check(DocsCommonArgs),
}

#[derive(Args, Debug, Clone)]
pub struct DocsCommonArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub artifacts_root: Option<PathBuf>,
    #[arg(long)]
    pub run_id: Option<String>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub allow_subprocess: bool,
    #[arg(long, default_value_t = false)]
    pub allow_write: bool,
    #[arg(long, default_value_t = false)]
    pub allow_network: bool,
    #[arg(long, default_value_t = false)]
    pub strict: bool,
    #[arg(long, default_value_t = false)]
    pub include_drafts: bool,
}

#[derive(Args, Debug, Clone)]
pub struct DocsServeArgs {
    #[command(flatten)]
    pub common: DocsCommonArgs,
    #[arg(long, default_value_t = false)]
    pub json: bool,
    #[arg(long, default_value_t = 8000)]
    pub port: u16,
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,
}

#[derive(Args, Debug, Clone)]
pub struct DocsGrepArgs {
    #[command(flatten)]
    pub common: DocsCommonArgs,
    pub pattern: String,
}

#[derive(Subcommand, Debug)]
pub enum ConfigsCommand {
    Print(ConfigsCommonArgs),
    List(ConfigsCommonArgs),
    Verify(ConfigsCommonArgs),
    Doctor(ConfigsCommonArgs),
    Validate(ConfigsCommonArgs),
    Lint(ConfigsCommonArgs),
    Fmt {
        #[arg(long = "check", default_value_t = false)]
        check: bool,
        #[command(flatten)]
        common: ConfigsCommonArgs,
    },
    Inventory(ConfigsCommonArgs),
    Compile(ConfigsCommonArgs),
    Diff(ConfigsCommonArgs),
}

#[derive(Subcommand, Debug)]
pub enum DockerCommand {
    Contracts(DockerCommonArgs),
    Gates(DockerCommonArgs),
    Doctor(DockerCommonArgs),
    Validate(DockerCommonArgs),
    Build(DockerCommonArgs),
    Check(DockerCommonArgs),
    Smoke(DockerCommonArgs),
    Scan(DockerCommonArgs),
    Sbom(DockerCommonArgs),
    Lock(DockerCommonArgs),
    Policy {
        #[command(subcommand)]
        command: DockerPolicyCommand,
    },
    Push(DockerReleaseArgs),
    Release(DockerReleaseArgs),
}

#[derive(Subcommand, Debug)]
pub enum DockerPolicyCommand {
    Check(DockerCommonArgs),
}

#[derive(Subcommand, Debug)]
pub enum BuildCommand {
    Bin(BuildCommonArgs),
    Plan(BuildCommonArgs),
    Verify(BuildCommonArgs),
    Meta(BuildCommonArgs),
    Dist(BuildCommonArgs),
    Clean(BuildCleanArgs),
    Doctor(BuildCommonArgs),
    InstallLocal(BuildCommonArgs),
}

#[derive(Args, Debug, Clone)]
pub struct BuildCommonArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long)]
    pub run_id: Option<String>,
    #[arg(long, default_value_t = false)]
    pub allow_write: bool,
    #[arg(long, default_value_t = false)]
    pub allow_subprocess: bool,
}

#[derive(Args, Debug, Clone)]
pub struct BuildCleanArgs {
    #[command(flatten)]
    pub common: BuildCommonArgs,
    #[arg(long, default_value_t = false)]
    pub include_bin: bool,
}

#[derive(Args, Debug, Clone)]
pub struct DockerCommonArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub artifacts_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long)]
    pub run_id: Option<String>,
    #[arg(long, default_value_t = false)]
    pub allow_subprocess: bool,
    #[arg(long, default_value_t = false)]
    pub allow_write: bool,
    #[arg(long, default_value_t = false)]
    pub allow_network: bool,
}

#[derive(Args, Debug, Clone)]
pub struct DockerReleaseArgs {
    #[command(flatten)]
    pub common: DockerCommonArgs,
    #[arg(long, default_value_t = false)]
    pub i_know_what_im_doing: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ConfigsCommonArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub artifacts_root: Option<PathBuf>,
    #[arg(long)]
    pub run_id: Option<String>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub allow_write: bool,
    #[arg(long, default_value_t = false)]
    pub allow_subprocess: bool,
    #[arg(long, default_value_t = false)]
    pub allow_network: bool,
    #[arg(long, default_value_t = false)]
    pub strict: bool,
}
#[derive(Subcommand, Debug)]
pub enum ContractsCommand {
    Docker(ContractsDockerArgs),
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractsModeArg {
    Static,
    Effect,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractsFormatArg {
    Pretty,
    Json,
}

#[derive(Args, Debug, Clone)]
pub struct ContractsDockerArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub artifacts_root: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
    #[arg(long, value_enum, default_value_t = ContractsFormatArg::Pretty)]
    pub format: ContractsFormatArg,
    #[arg(long, value_enum, default_value_t = ContractsModeArg::Static)]
    pub mode: ContractsModeArg,
    #[arg(long, default_value_t = false)]
    pub fail_fast: bool,
    #[arg(long)]
    pub filter: Option<String>,
    #[arg(long)]
    pub filter_test: Option<String>,
    #[arg(long, default_value_t = false)]
    pub list: bool,
    #[arg(long, default_value_t = false)]
    pub list_tests: bool,
    #[arg(long)]
    pub explain: Option<String>,
    #[arg(long, default_value_t = false)]
    pub allow_subprocess: bool,
    #[arg(long, default_value_t = false)]
    pub allow_network: bool,
    #[arg(long, default_value_t = false)]
    pub skip_missing_tools: bool,
    #[arg(long, default_value_t = 300)]
    pub timeout_seconds: u64,
}
