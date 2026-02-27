// SPDX-License-Identifier: Apache-2.0

use std::path::PathBuf;

use clap::{Args, Subcommand, ValueEnum};

use super::FormatArg;

#[derive(Subcommand, Debug, Clone)]
pub enum OpsCommand {
    List(OpsCommonArgs),
    Explain {
        action: String,
        #[command(flatten)]
        common: OpsCommonArgs,
    },
    Stack {
        #[command(subcommand)]
        command: OpsStackCommand,
    },
    K8s {
        #[command(subcommand)]
        command: OpsK8sCommand,
    },
    Load {
        #[command(subcommand)]
        command: OpsLoadCommand,
    },
    Datasets {
        #[command(subcommand)]
        command: OpsDatasetsCommand,
    },
    E2e {
        #[command(subcommand)]
        command: OpsE2eCommand,
    },
    Obs {
        #[command(subcommand)]
        command: OpsObsCommand,
    },
    Schema {
        #[command(subcommand)]
        command: OpsSchemaCommand,
    },
    InventoryDomain {
        #[command(name = "inventory", subcommand)]
        command: OpsInventoryCommand,
    },
    ReportDomain {
        #[command(name = "report", subcommand)]
        command: OpsReportCommand,
    },
    Evidence {
        #[command(subcommand)]
        command: OpsEvidenceCommand,
    },
    Tools {
        #[command(subcommand)]
        command: OpsToolsCommand,
    },
    Suite {
        #[command(subcommand)]
        command: OpsSuiteCommand,
    },
    Doctor(OpsCommonArgs),
    Validate(OpsCommonArgs),
    Graph(OpsCommonArgs),
    Inventory(OpsCommonArgs),
    Docs(OpsCommonArgs),
    DocsVerify(OpsCommonArgs),
    Conformance(OpsCommonArgs),
    Report(OpsCommonArgs),
    Readiness(OpsCommonArgs),
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
    Plan(OpsCommonArgs),
    Up(OpsCommonArgs),
    Down(OpsCommonArgs),
    Clean(OpsCommonArgs),
    Cleanup(OpsCommonArgs),
    Reset(OpsResetArgs),
    Pins {
        #[command(subcommand)]
        command: OpsPinsCommand,
    },
    Generate {
        #[command(subcommand)]
        command: OpsGenerateCommand,
    },
    #[command(hide = true)]
    K8sPlan(OpsCommonArgs),
    #[command(hide = true)]
    K8sApply(OpsK8sApplyArgs),
    #[command(hide = true)]
    K8sDryRun(OpsCommonArgs),
    #[command(hide = true)]
    K8sConformance(OpsCommonArgs),
    #[command(hide = true)]
    K8sWait(OpsK8sWaitArgs),
    #[command(hide = true)]
    K8sLogs(OpsK8sLogsArgs),
    #[command(hide = true)]
    K8sPortForward(OpsK8sPortForwardArgs),
    #[command(hide = true)]
    LoadPlan {
        suite: String,
        #[command(flatten)]
        common: OpsCommonArgs,
    },
    #[command(hide = true)]
    LoadRun {
        suite: String,
        #[command(flatten)]
        common: OpsCommonArgs,
    },
    #[command(hide = true)]
    LoadReport {
        suite: String,
        #[command(flatten)]
        common: OpsCommonArgs,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum OpsStackCommand {
    Plan(OpsCommonArgs),
    Up(OpsCommonArgs),
    Down(OpsCommonArgs),
    Status(OpsStatusArgs),
    Logs(OpsCommonArgs),
    Ports(OpsCommonArgs),
    Versions(OpsCommonArgs),
    Doctor(OpsCommonArgs),
    Reset(OpsResetArgs),
}

#[derive(Subcommand, Debug, Clone)]
pub enum OpsK8sCommand {
    Render(OpsRenderArgs),
    Install(OpsInstallArgs),
    Uninstall(OpsCommonArgs),
    Diff(OpsCommonArgs),
    Rollout(OpsCommonArgs),
    Plan(OpsCommonArgs),
    Apply(OpsK8sApplyArgs),
    DryRun(OpsCommonArgs),
    Conformance(OpsCommonArgs),
    Wait(OpsK8sWaitArgs),
    Logs(OpsK8sLogsArgs),
    PortForward(OpsK8sPortForwardArgs),
    Test(OpsCommonArgs),
    Status(OpsStatusArgs),
}

#[derive(Args, Debug, Clone)]
pub struct OpsK8sApplyArgs {
    #[command(flatten)]
    pub common: OpsCommonArgs,
    #[arg(long, default_value_t = false)]
    pub apply: bool,
}

#[derive(Args, Debug, Clone)]
pub struct OpsK8sWaitArgs {
    #[command(flatten)]
    pub common: OpsCommonArgs,
    #[arg(long, default_value_t = 120)]
    pub timeout_seconds: u64,
}

#[derive(Args, Debug, Clone)]
pub struct OpsK8sLogsArgs {
    #[command(flatten)]
    pub common: OpsCommonArgs,
    #[arg(long)]
    pub pod: Option<String>,
    #[arg(long, default_value_t = 200)]
    pub tail: usize,
}

#[derive(Args, Debug, Clone)]
pub struct OpsK8sPortForwardArgs {
    #[command(flatten)]
    pub common: OpsCommonArgs,
    #[arg(long, default_value = "service/bijux-atlas")]
    pub resource: String,
    #[arg(long, default_value_t = 8080)]
    pub local_port: u16,
    #[arg(long, default_value_t = 8080)]
    pub remote_port: u16,
}

#[derive(Subcommand, Debug, Clone)]
pub enum OpsLoadCommand {
    Plan {
        suite: String,
        #[command(flatten)]
        common: OpsCommonArgs,
    },
    Run {
        suite: String,
        #[command(flatten)]
        common: OpsCommonArgs,
    },
    Report {
        suite: String,
        #[command(flatten)]
        common: OpsCommonArgs,
    },
    Baseline {
        #[command(subcommand)]
        command: OpsLoadBaselineCommand,
    },
    Evaluate(OpsCommonArgs),
    ListSuites(OpsCommonArgs),
}

#[derive(Subcommand, Debug, Clone)]
pub enum OpsLoadBaselineCommand {
    Update(OpsCommonArgs),
}

#[derive(Subcommand, Debug, Clone)]
pub enum OpsDatasetsCommand {
    List(OpsCommonArgs),
    Ingest(OpsCommonArgs),
    Publish(OpsCommonArgs),
    Promote(OpsCommonArgs),
    Rollback(OpsCommonArgs),
    Qc(OpsCommonArgs),
}

#[derive(Subcommand, Debug, Clone)]
pub enum OpsE2eCommand {
    Run(OpsCommonArgs),
    Smoke(OpsCommonArgs),
    Realdata(OpsCommonArgs),
    ListSuites(OpsCommonArgs),
}

#[derive(Subcommand, Debug, Clone)]
pub enum OpsObsCommand {
    Up(OpsCommonArgs),
    Down(OpsCommonArgs),
    Validate(OpsCommonArgs),
    Snapshot(OpsCommonArgs),
    Dashboards(OpsCommonArgs),
    Drill {
        #[command(subcommand)]
        command: OpsObsDrillCommand,
    },
    Verify(OpsCommonArgs),
}

#[derive(Subcommand, Debug, Clone)]
pub enum OpsObsDrillCommand {
    Run(OpsCommonArgs),
}

#[derive(Subcommand, Debug, Clone)]
pub enum OpsSchemaCommand {
    Validate(OpsCommonArgs),
    Diff(OpsCommonArgs),
    Coverage(OpsCommonArgs),
    RegenIndex(OpsCommonArgs),
}

#[derive(Subcommand, Debug, Clone)]
pub enum OpsInventoryCommand {
    Validate(OpsCommonArgs),
    Graph(OpsCommonArgs),
    Diff(OpsCommonArgs),
    Coverage(OpsCommonArgs),
    OrphanCheck(OpsCommonArgs),
}

#[derive(Subcommand, Debug, Clone)]
pub enum OpsReportCommand {
    Generate(OpsCommonArgs),
    Diff(OpsCommonArgs),
    Readiness(OpsCommonArgs),
    Bundle(OpsCommonArgs),
}

#[derive(Subcommand, Debug, Clone)]
pub enum OpsEvidenceCommand {
    Collect(OpsCommonArgs),
    Verify(OpsCommonArgs),
}

#[derive(Subcommand, Debug, Clone)]
pub enum OpsToolsCommand {
    List(OpsCommonArgs),
    Verify(OpsCommonArgs),
    Doctor(OpsCommonArgs),
}

#[derive(Subcommand, Debug, Clone)]
pub enum OpsSuiteCommand {
    List(OpsCommonArgs),
    Run {
        suite: String,
        #[command(flatten)]
        common: OpsCommonArgs,
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

#[derive(Subcommand, Debug, Clone)]
pub enum OpsPinsCommand {
    Check(OpsCommonArgs),
    Update {
        #[arg(long, default_value_t = false)]
        i_know_what_im_doing: bool,
        #[command(flatten)]
        common: OpsCommonArgs,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum OpsGenerateCommand {
    PinsIndex {
        #[arg(long, default_value_t = false)]
        check: bool,
        #[command(flatten)]
        common: OpsCommonArgs,
    },
    SurfaceList {
        #[arg(long, default_value_t = false)]
        check: bool,
        #[arg(long, default_value_t = false)]
        write_example: bool,
        #[command(flatten)]
        common: OpsCommonArgs,
    },
    Runbook {
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
    pub artifacts_root: Option<PathBuf>,
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
    pub fail_fast: bool,
    #[arg(long)]
    pub max_failures: Option<usize>,
    #[arg(long, default_value_t = false)]
    pub allow_subprocess: bool,
    #[arg(long, default_value_t = false)]
    pub allow_write: bool,
    #[arg(long, default_value_t = false)]
    pub allow_network: bool,
    #[arg(long, default_value_t = false)]
    pub force: bool,
    #[arg(long = "tool")]
    pub tool_overrides: Vec<String>,
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
