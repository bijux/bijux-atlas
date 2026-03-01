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
    ExternalLinks(DocsExternalLinksArgs),
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

#[derive(Args, Debug, Clone)]
pub struct DocsExternalLinksArgs {
    #[command(flatten)]
    pub common: DocsCommonArgs,
    #[arg(long, default_value = "configs/docs/external-link-allowlist.json")]
    pub allowlist: PathBuf,
}

#[derive(Subcommand, Debug)]
pub enum ConfigsCommand {
    Print(ConfigsCommonArgs),
    List(ConfigsCommonArgs),
    Explain(ConfigsExplainArgs),
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

#[derive(Args, Debug, Clone)]
pub struct ConfigsExplainArgs {
    #[command(flatten)]
    pub common: ConfigsCommonArgs,
    pub file: String,
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
    All(ContractsCommonArgs),
    Root(ContractsCommonArgs),
    Configs(ContractsCommonArgs),
    Docs(ContractsCommonArgs),
    Docker(ContractsDockerArgs),
    Make(ContractsMakeArgs),
    Ops(ContractsOpsArgs),
    SelfCheck(ContractsCommonArgs),
    Snapshot(ContractsSnapshotArgs),
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractsModeArg {
    Static,
    Effect,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractsFormatArg {
    #[value(alias = "pretty", alias = "text")]
    Human,
    Table,
    Json,
    Junit,
    Github,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractsProfileArg {
    Local,
    Ci,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractsLaneArg {
    Local,
    Pr,
    Merge,
    Release,
}

#[derive(Args, Debug, Clone)]
pub struct ContractsCommonArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub artifacts_root: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
    #[arg(long, value_enum, default_value_t = ContractsFormatArg::Human)]
    pub format: ContractsFormatArg,
    #[arg(long, value_enum, default_value_t = ContractsModeArg::Static)]
    pub mode: ContractsModeArg,
    #[arg(long, value_enum, default_value_t = ContractsProfileArg::Local)]
    pub profile: ContractsProfileArg,
    #[arg(long, value_enum, default_value_t = ContractsLaneArg::Local)]
    pub lane: ContractsLaneArg,
    #[arg(long, default_value_t = false)]
    pub required: bool,
    #[arg(long, default_value_t = false)]
    pub ci: bool,
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub deny_skip_required: bool,
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    pub deny_effects: bool,
    #[arg(long, default_value_t = false)]
    pub fail_fast: bool,
    #[arg(long = "filter-contract", alias = "filter")]
    #[arg(alias = "id")]
    pub filter_contract: Option<String>,
    #[arg(long = "group")]
    pub groups: Vec<String>,
    #[arg(long)]
    pub filter_test: Option<String>,
    #[arg(long = "only")]
    pub only_contracts: Vec<String>,
    #[arg(long = "only-test")]
    pub only_tests: Vec<String>,
    #[arg(long = "skip")]
    pub skip_contracts: Vec<String>,
    #[arg(long = "tag")]
    pub tags: Vec<String>,
    #[arg(long, default_value_t = false)]
    pub list: bool,
    #[arg(long, default_value_t = false)]
    pub list_tests: bool,
    #[arg(long, default_value_t = false)]
    pub changed_only: bool,
    #[arg(long)]
    pub explain: Option<String>,
    #[arg(long = "explain-test")]
    pub explain_test: Option<String>,
    #[arg(long = "json-out")]
    pub json_out: Option<PathBuf>,
    #[arg(long = "junit-out")]
    pub junit_out: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub allow_subprocess: bool,
    #[arg(long, default_value_t = false)]
    pub allow_network: bool,
    #[arg(long, default_value_t = false)]
    pub allow_k8s: bool,
    #[arg(long, default_value_t = false)]
    pub allow_fs_write: bool,
    #[arg(long, default_value_t = false)]
    pub allow_docker_daemon: bool,
    #[arg(long, default_value_t = false)]
    pub skip_missing_tools: bool,
    #[arg(long, default_value_t = 300)]
    pub timeout_seconds: u64,
}

#[derive(Args, Debug, Clone)]
pub struct ContractsDockerArgs {
    #[command(flatten)]
    pub common: ContractsCommonArgs,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractsOpsDomainArg {
    Root,
    Datasets,
    E2e,
    Env,
    Inventory,
    K8s,
    Load,
    Observe,
    Report,
    Schema,
    Stack,
}

#[derive(Args, Debug, Clone)]
pub struct ContractsOpsArgs {
    #[command(flatten)]
    pub common: ContractsCommonArgs,
    #[arg(long, value_enum)]
    pub domain: Option<ContractsOpsDomainArg>,
}

#[derive(Args, Debug, Clone)]
pub struct ContractsMakeArgs {
    #[command(flatten)]
    pub common: ContractsCommonArgs,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContractsSnapshotDomainArg {
    All,
    Root,
    Configs,
    Docs,
    Docker,
    Make,
    Ops,
}

#[derive(Args, Debug, Clone)]
pub struct ContractsSnapshotArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = ContractsSnapshotDomainArg::Ops)]
    pub domain: ContractsSnapshotDomainArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}
