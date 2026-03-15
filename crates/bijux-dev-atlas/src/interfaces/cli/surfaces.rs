// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use clap::{Args, Subcommand};

use super::FormatArg;

#[derive(Subcommand, Debug)]
pub enum ArtifactsCommand {
    Clean(ArtifactsCommonArgs),
    Gc(ArtifactsGcArgs),
    Report {
        #[command(subcommand)]
        command: ArtifactsReportCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum ReportsCommand {
    List(ReportsListArgs),
    Index(ReportsListArgs),
    Progress(ReportsListArgs),
    Validate(ReportsValidateArgs),
}

#[derive(Args, Debug, Clone)]
pub struct ReportsListArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReportsValidateArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub dir: PathBuf,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ArtifactsCommonArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub allow_write: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ArtifactsGcArgs {
    #[command(flatten)]
    pub common: ArtifactsCommonArgs,
    #[arg(long, default_value_t = 5)]
    pub keep_last: usize,
}

#[derive(Subcommand, Debug)]
pub enum ArtifactsReportCommand {
    Inventory(ArtifactsCommonArgs),
    Manifest(ArtifactsReportScanArgs),
    Index(ArtifactsReportScanArgs),
    Read(ArtifactsReportReadArgs),
    Diff(ArtifactsReportDiffArgs),
    Validate(ArtifactsReportScanArgs),
}

#[derive(Args, Debug, Clone)]
pub struct ArtifactsReportScanArgs {
    #[command(flatten)]
    pub common: ArtifactsCommonArgs,
    #[arg(long)]
    pub reports_root: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ArtifactsReportDiffArgs {
    #[command(flatten)]
    pub common: ArtifactsCommonArgs,
    #[arg(long)]
    pub baseline_root: PathBuf,
    #[arg(long)]
    pub candidate_root: PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct ArtifactsReportReadArgs {
    #[command(flatten)]
    pub common: ArtifactsCommonArgs,
    #[arg(long)]
    pub report_path: Option<PathBuf>,
    #[arg(long)]
    pub reports_root: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum DocsCommand {
    Check(DocsCommonArgs),
    VerifyContracts(DocsCommonArgs),
    Doctor(DocsCommonArgs),
    DeployPlan(DocsCommonArgs),
    PagesSmoke(DocsPagesSmokeArgs),
    UxSmoke(DocsCommonArgs),
    Where(DocsCommonArgs),
    SiteDir(DocsCommonArgs),
    Validate(DocsCommonArgs),
    Build(DocsCommonArgs),
    Serve(DocsServeArgs),
    Clean(DocsCommonArgs),
    Lint(DocsCommonArgs),
    IncludesCheck(DocsCommonArgs),
    Links(DocsCommonArgs),
    NavIntegrity(DocsCommonArgs),
    ExternalLinks(DocsExternalLinksArgs),
    Inventory(DocsCommonArgs),
    Graph(DocsCommonArgs),
    Top(DocsTopArgs),
    Dead(DocsCommonArgs),
    Duplicates(DocsCommonArgs),
    PrunePlan(DocsCommonArgs),
    DedupeReport(DocsCommonArgs),
    ShrinkReport(DocsCommonArgs),
    Grep(DocsGrepArgs),
    HealthDashboard(DocsCommonArgs),
    LifecycleSummaryTable(DocsTableRenderArgs),
    DrillSummaryTable(DocsTableRenderArgs),
    Reference {
        #[command(subcommand)]
        command: DocsReferenceCommand,
    },
    Generate {
        #[command(subcommand)]
        command: DocsGenerateCommand,
    },
    Redirects {
        #[command(subcommand)]
        command: DocsRedirectsCommand,
    },
    Merge {
        #[command(subcommand)]
        command: DocsMergeCommand,
    },
    Spine {
        #[command(subcommand)]
        command: DocsSpineCommand,
    },
    Toc {
        #[command(subcommand)]
        command: DocsTocCommand,
    },
    VerifyGenerated(DocsCommonArgs),
}

#[derive(Subcommand, Debug)]
pub enum DocsSpineCommand {
    Validate(DocsCommonArgs),
    Report(DocsCommonArgs),
}

#[derive(Subcommand, Debug)]
pub enum DocsTocCommand {
    Verify(DocsCommonArgs),
}

#[derive(Subcommand, Debug)]
pub enum DocsReferenceCommand {
    Generate(DocsCommonArgs),
    Check(DocsCommonArgs),
}

#[derive(Subcommand, Debug)]
pub enum DocsGenerateCommand {
    Examples(DocsCommonArgs),
    CommandLists(DocsCommonArgs),
    SchemaSnippets(DocsCommonArgs),
    OpenapiSnippets(DocsCommonArgs),
    OpsSnippets(DocsCommonArgs),
    RealDataPages(DocsCommonArgs),
}

#[derive(Subcommand, Debug)]
pub enum DocsRedirectsCommand {
    Sync(DocsCommonArgs),
}

#[derive(Subcommand, Debug)]
pub enum DocsMergeCommand {
    Validate(DocsCommonArgs),
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
pub struct DocsTableRenderArgs {
    #[command(flatten)]
    pub common: DocsCommonArgs,
    #[arg(long)]
    pub input: PathBuf,
    #[arg(long)]
    pub output: PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct DocsExternalLinksArgs {
    #[command(flatten)]
    pub common: DocsCommonArgs,
    #[arg(long, default_value = "configs/sources/repository/docs/external-link-allowlist.json")]
    pub allowlist: PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct DocsPagesSmokeArgs {
    #[command(flatten)]
    pub common: DocsCommonArgs,
    #[arg(long)]
    pub url: Option<String>,
    #[arg(long, default_value = "Docs Build Info")]
    pub marker: String,
}

#[derive(Args, Debug, Clone)]
pub struct DocsTopArgs {
    #[command(flatten)]
    pub common: DocsCommonArgs,
    #[arg(long, default_value_t = 50)]
    pub limit: usize,
}

#[derive(Subcommand, Debug)]
pub enum ConfigsCommand {
    Print(ConfigsCommonArgs),
    List(ConfigsCommonArgs),
    Graph(ConfigsCommonArgs),
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

#[derive(Subcommand, Debug)]
pub enum MakesCommand {
    VerifyModule(MakesVerifyArgs),
    Wrappers {
        #[command(subcommand)]
        command: MakesWrappersCommand,
    },
    Surface(MakesCommonArgs),
    List(MakesCommonArgs),
    Explain(MakesExplainArgs),
    TargetList(MakesCommonArgs),
    LintPolicyReport(MakesCommonArgs),
}

#[derive(Subcommand, Debug)]
pub enum MakesWrappersCommand {
    Verify(MakesCommonArgs),
}

#[derive(Args, Debug, Clone)]
pub struct MakesCommonArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub allow_subprocess: bool,
    #[arg(long, default_value_t = false)]
    pub allow_write: bool,
}

#[derive(Args, Debug, Clone)]
pub struct MakesVerifyArgs {
    #[command(flatten)]
    pub common: MakesCommonArgs,
    pub module: String,
}

#[derive(Args, Debug, Clone)]
pub struct MakesExplainArgs {
    #[command(flatten)]
    pub common: MakesCommonArgs,
    pub target: String,
}

#[derive(Args, Debug, Clone)]
pub struct ConfigsExplainArgs {
    #[command(flatten)]
    pub common: ConfigsCommonArgs,
    pub file: String,
}

#[derive(Subcommand, Debug)]
pub enum DockerCommand {
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
