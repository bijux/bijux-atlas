use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "bijux-dev-atlas", version, disable_help_subcommand = true)]
#[command(about = "Bijux Atlas development control-plane")]
pub struct Cli {
    #[arg(long, default_value_t = false)]
    pub quiet: bool,
    #[arg(long, default_value_t = false)]
    pub verbose: bool,
    #[arg(long, default_value_t = false)]
    pub debug: bool,
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
    Version {
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
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
    Docker {
        #[command(subcommand)]
        command: DockerCommand,
    },
    Policies {
        #[command(subcommand)]
        command: PoliciesCommand,
    },
    Workflows {
        #[command(subcommand)]
        command: WorkflowsCommand,
    },
    Gates {
        #[command(subcommand)]
        command: GatesCommand,
    },
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

#[derive(Subcommand, Debug)]
pub enum CheckCommand {
    Registry {
        #[command(subcommand)]
        command: CheckRegistryCommand,
    },
    List {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_name = "ci-fast|ci|local|deep|<suite_id>")]
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
        #[arg(long, value_name = "ci-fast|ci|local|deep|<suite_id>")]
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
            value_name = "ci-fast|ci|local|deep|<suite_id>",
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

#[derive(Subcommand, Debug)]
pub enum OpsCommand {
    Doctor(OpsCommonArgs),
    Validate(OpsCommonArgs),
    Inventory(OpsCommonArgs),
    Docs(OpsCommonArgs),
    Conformance(OpsCommonArgs),
    Report(OpsCommonArgs),
    Render(OpsRenderArgs),
    Install(OpsInstallArgs),
    Status(OpsStatusArgs),
    ListProfiles(OpsCommonArgs),
    ExplainProfile {
        name: String,
        #[command(flatten)]
        common: OpsCommonArgs,
    },
    ListTools(OpsCommonArgs),
    VerifyTools(OpsCommonArgs),
    ListActions(OpsCommonArgs),
    Up(OpsCommonArgs),
    Down(OpsCommonArgs),
    Clean(OpsCommonArgs),
    Reset(OpsResetArgs),
    Pins {
        #[command(subcommand)]
        command: OpsPinsCommand,
    },
    Generate {
        #[command(subcommand)]
        command: OpsGenerateCommand,
    },
}

#[derive(Args, Debug, Clone)]
pub struct OpsRenderArgs {
    #[command(flatten)]
    pub common: OpsCommonArgs,
    #[arg(long, value_enum, default_value_t = OpsRenderTarget::Helm)]
    pub target: OpsRenderTarget,
    #[arg(long, default_value_t = false)]
    pub check: bool,
    #[arg(long, default_value_t = false)]
    pub write: bool,
    #[arg(long, default_value_t = false)]
    pub stdout: bool,
    #[arg(long, default_value_t = false)]
    pub diff: bool,
    #[arg(long)]
    pub helm_binary: Option<String>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum OpsRenderTarget {
    Helm,
    Kustomize,
    Kind,
}

#[derive(Subcommand, Debug)]
pub enum OpsPinsCommand {
    Check(OpsCommonArgs),
    Update {
        #[arg(long, default_value_t = false)]
        i_know_what_im_doing: bool,
        #[command(flatten)]
        common: OpsCommonArgs,
    },
}

#[derive(Subcommand, Debug)]
pub enum OpsGenerateCommand {
    PinsIndex {
        #[arg(long, default_value_t = false)]
        check: bool,
        #[command(flatten)]
        common: OpsCommonArgs,
    },
}

#[derive(Args, Debug, Clone)]
pub struct OpsCommonArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub ops_root: Option<PathBuf>,
    #[arg(long)]
    pub profile: Option<String>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long)]
    pub run_id: Option<String>,
    #[arg(long, default_value_t = false)]
    pub strict: bool,
    #[arg(long, default_value_t = false)]
    pub allow_subprocess: bool,
    #[arg(long, default_value_t = false)]
    pub allow_write: bool,
}

#[derive(Args, Debug, Clone)]
pub struct OpsInstallArgs {
    #[command(flatten)]
    pub common: OpsCommonArgs,
    #[arg(long, default_value_t = false)]
    pub kind: bool,
    #[arg(long, default_value_t = false)]
    pub apply: bool,
    #[arg(long, default_value_t = false)]
    pub plan: bool,
    #[arg(long, default_value = "none")]
    pub dry_run: String,
    #[arg(long, default_value_t = false)]
    pub force: bool,
}

#[derive(Args, Debug, Clone)]
pub struct OpsStatusArgs {
    #[command(flatten)]
    pub common: OpsCommonArgs,
    #[arg(long, value_enum, default_value_t = OpsStatusTarget::Local)]
    pub target: OpsStatusTarget,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum OpsStatusTarget {
    Local,
    K8s,
    Pods,
    Endpoints,
}

#[derive(Args, Debug, Clone)]
pub struct OpsResetArgs {
    #[command(flatten)]
    pub common: OpsCommonArgs,
    #[arg(long = "reset-run-id")]
    pub reset_id: String,
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
    Grep(DocsGrepArgs),
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
    Build(DockerCommonArgs),
    Check(DockerCommonArgs),
    Push(DockerReleaseArgs),
    Release(DockerReleaseArgs),
}

#[derive(Args, Debug, Clone)]
pub struct DockerCommonArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
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
