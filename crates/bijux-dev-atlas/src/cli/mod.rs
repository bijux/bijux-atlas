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
mod registry;
mod suites;
mod surfaces;
mod tests;

pub use checks::*;
pub use ops::*;
pub use registry::*;
pub use suites::*;
pub use surfaces::*;
pub use tests::*;

pub(crate) fn run() -> i32 {
    let cli = Cli::parse();
    dispatch::run_cli(cli)
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, Eq, PartialEq)]
pub enum GlobalFormatArg {
    Human,
    Json,
    Both,
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
    #[arg(long = "output-format", global = true, value_enum)]
    pub output_format: Option<GlobalFormatArg>,
    #[arg(long, global = true, default_value_t = false)]
    pub no_deprecation_warn: bool,
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
    Reports {
        #[command(subcommand)]
        command: ReportsCommand,
    },
    #[command(hide = true)]
    Make {
        #[command(subcommand)]
        command: MakeCommand,
    },
    #[command(hide = true)]
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
    Governance {
        #[command(subcommand)]
        command: GovernanceCommand,
    },
    System {
        #[command(subcommand)]
        command: SystemCommand,
    },
    Audit {
        #[command(subcommand)]
        command: AuditCommand,
    },
    Security {
        #[command(subcommand)]
        command: SecurityCommand,
    },
    Datasets {
        #[command(subcommand)]
        command: DatasetsCommand,
    },
    Ingest {
        #[command(subcommand)]
        command: IngestCommand,
    },
    Perf {
        #[command(subcommand)]
        command: PerfCommand,
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
    Checks {
        #[command(subcommand)]
        command: ChecksCommand,
    },
    Contract {
        #[command(subcommand)]
        command: ContractCommand,
    },
    Registry {
        #[command(subcommand)]
        command: RegistryCommand,
    },
    Suites {
        #[command(subcommand)]
        command: SuitesCommand,
    },
    Tests {
        #[command(subcommand)]
        command: TestsCommand,
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
        id: String,
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Run {
        id: String,
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
    Rebuild {
        #[command(subcommand)]
        command: ReleaseRebuildCommand,
    },
    Reproducibility {
        #[command(subcommand)]
        command: ReleaseReproducibilityCommand,
    },
    Version {
        #[command(subcommand)]
        command: ReleaseVersionCommand,
    },
    Changelog {
        #[command(subcommand)]
        command: ReleaseChangelogCommand,
    },
    Manifest {
        #[command(subcommand)]
        command: ReleaseManifestCommand,
    },
    Bundle {
        #[command(subcommand)]
        command: ReleaseBundleCommand,
    },
    Sign(ReleaseSignArgs),
    Verify(ReleaseVerifyArgs),
    Diff(ReleaseDiffArgs),
    Packet(ReleasePacketArgs),
}

#[derive(Subcommand, Debug)]
pub enum ReleaseManifestCommand {
    Generate(ReleaseManifestGenerateArgs),
    Validate(ReleaseManifestValidateArgs),
}

#[derive(Subcommand, Debug)]
pub enum ReleaseRebuildCommand {
    Verify(ReleaseRebuildVerifyArgs),
}

#[derive(Subcommand, Debug)]
pub enum ReleaseReproducibilityCommand {
    Report(ReleaseReproducibilityReportArgs),
}

#[derive(Subcommand, Debug)]
pub enum ReleaseVersionCommand {
    Check(ReleaseVersionCheckArgs),
}

#[derive(Subcommand, Debug)]
pub enum ReleaseChangelogCommand {
    Generate(ReleaseChangelogGenerateArgs),
    Validate(ReleaseChangelogValidateArgs),
}

#[derive(Subcommand, Debug)]
pub enum ReleaseBundleCommand {
    Build(ReleaseBundleBuildArgs),
    Verify(ReleaseBundleVerifyArgs),
    Hash(ReleaseBundleHashArgs),
}

#[derive(Subcommand, Debug)]
pub enum PerfCommand {
    Validate(PerfValidateArgs),
    Run(PerfRunArgs),
    Diff(PerfDiffArgs),
    ColdStart(PerfValidateArgs),
    Kind(PerfKindArgs),
    Benches {
        #[command(subcommand)]
        command: PerfBenchesCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum DatasetsCommand {
    Validate(DatasetsValidateArgs),
}

#[derive(Subcommand, Debug)]
pub enum IngestCommand {
    DryRun(IngestDryRunArgs),
    Run(IngestDryRunArgs),
}

#[derive(Subcommand, Debug)]
pub enum PerfBenchesCommand {
    List(PerfValidateArgs),
}

#[derive(Subcommand, Debug)]
pub enum GovernanceCommand {
    Version {
        #[arg(long)]
        repo_root: Option<PathBuf>,
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
    Explain {
        id: String,
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Validate {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    #[command(visible_alias = "exception")]
    Exceptions {
        #[command(subcommand)]
        command: GovernanceExceptionsCommand,
    },
    Deprecations {
        #[command(subcommand)]
        command: GovernanceDeprecationsCommand,
    },
    Breaking {
        #[command(subcommand)]
        command: GovernanceBreakingCommand,
    },
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
pub enum GovernanceExceptionsCommand {
    List {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Validate {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
pub enum GovernanceDeprecationsCommand {
    Validate {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
pub enum GovernanceBreakingCommand {
    Validate {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Subcommand, Debug)]
pub enum SecurityCommand {
    Validate(SecurityValidateArgs),
    Compliance {
        #[command(subcommand)]
        command: SecurityComplianceCommand,
    },
    ScanArtifacts(SecurityScanArtifactsArgs),
}

#[derive(Subcommand, Debug)]
pub enum SystemCommand {
    Simulate {
        #[command(subcommand)]
        command: SystemSimulateCommand,
    },
    Debug {
        #[command(subcommand)]
        command: SystemDebugCommand,
    },
    Cluster {
        #[command(subcommand)]
        command: SystemClusterCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum SystemSimulateCommand {
    Install(SystemSimulateArgs),
    Upgrade(SystemSimulateArgs),
    Rollback(SystemSimulateArgs),
    OfflineMode(SystemSimulateArgs),
    Suite(SystemSimulateArgs),
}

#[derive(Subcommand, Debug)]
pub enum SystemDebugCommand {
    Diagnostics(SystemDebugArgs),
    MetricsSnapshot(SystemDebugArgs),
    HealthChecks(SystemDebugArgs),
    RuntimeState(SystemDebugArgs),
    TraceSampling(SystemDebugArgs),
}

#[derive(Subcommand, Debug)]
pub enum SystemClusterCommand {
    Topology(SystemClusterArgs),
    NodeList(SystemClusterArgs),
    Status(SystemClusterArgs),
    Diagnostics(SystemClusterArgs),
    Membership(SystemClusterArgs),
    NodeHealth(SystemClusterArgs),
    NodeDrain(SystemClusterNodeActionArgs),
    NodeMaintenance(SystemClusterNodeActionArgs),
    NodeDiagnostics(SystemClusterNodeActionArgs),
    ShardRouting(SystemClusterArgs),
    ShardList(SystemClusterArgs),
    ShardDistribution(SystemClusterArgs),
    ShardDiagnostics(SystemClusterArgs),
    ShardRebalance(SystemClusterShardActionArgs),
}

#[derive(Subcommand, Debug)]
pub enum AuditCommand {
    Bundle {
        #[command(subcommand)]
        command: AuditBundleCommand,
    },
    Compliance {
        #[command(subcommand)]
        command: AuditComplianceCommand,
    },
    Readiness {
        #[command(subcommand)]
        command: AuditReadinessCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum AuditBundleCommand {
    Generate(AuditBundleArgs),
    Validate(AuditBundleArgs),
}

#[derive(Subcommand, Debug)]
pub enum AuditComplianceCommand {
    Report(AuditBundleArgs),
}

#[derive(Subcommand, Debug)]
pub enum AuditReadinessCommand {
    Validate(AuditBundleArgs),
}

#[derive(Args, Debug, Clone)]
pub struct AuditBundleArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct SystemSimulateArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct SystemDebugArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long, default_value = "http://127.0.0.1:8080")]
    pub base_url: String,
    #[arg(long, default_value_t = 5)]
    pub timeout_seconds: u64,
}

#[derive(Args, Debug, Clone)]
pub struct SystemClusterArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long, default_value = "configs/ops/runtime/cluster-config.example.json")]
    pub cluster_config: PathBuf,
    #[arg(long, default_value = "configs/ops/runtime/node-config.example.json")]
    pub node_config: PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct SystemClusterNodeActionArgs {
    #[command(flatten)]
    pub common: SystemClusterArgs,
    #[arg(long)]
    pub node_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct SystemClusterShardActionArgs {
    #[command(flatten)]
    pub common: SystemClusterArgs,
    #[arg(long)]
    pub shard_id: Option<String>,
    #[arg(long)]
    pub target_node_id: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct SecurityValidateArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum SecurityComplianceCommand {
    Validate(SecurityValidateArgs),
}

#[derive(Args, Debug, Clone)]
pub struct SecurityScanArtifactsArgs {
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

#[derive(Args, Debug, Clone)]
pub struct ReleaseRebuildVerifyArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseReproducibilityReportArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseVersionCheckArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long)]
    pub tag: Option<String>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseChangelogGenerateArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseChangelogValidateArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long)]
    pub tag: Option<String>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseManifestGenerateArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseManifestValidateArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseBundleBuildArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseBundleVerifyArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseBundleHashArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseSignArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, default_value = "release/evidence")]
    pub evidence: PathBuf,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseVerifyArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub evidence: PathBuf,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseDiffArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg()]
    pub evidence_a: PathBuf,
    #[arg()]
    pub evidence_b: PathBuf,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReleasePacketArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, default_value = "release/evidence")]
    pub evidence: PathBuf,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct PerfValidateArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct PerfRunArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, default_value = "gene-lookup")]
    pub scenario: String,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct PerfDiffArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg()]
    pub report_a: PathBuf,
    #[arg()]
    pub report_b: PathBuf,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct PerfKindArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, default_value = "perf")]
    pub profile: String,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct DatasetsValidateArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct IngestDryRunArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub dataset: String,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}
