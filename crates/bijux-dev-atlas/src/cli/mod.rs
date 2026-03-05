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
    Observe {
        #[command(subcommand)]
        command: ObserveCommand,
    },
    Api {
        #[command(subcommand)]
        command: ApiCommand,
    },
    Load {
        #[command(subcommand)]
        command: LoadCommand,
    },
    Invariants {
        #[command(subcommand)]
        command: InvariantsCommand,
    },
    #[command(hide = true)]
    Drift {
        #[command(subcommand)]
        command: DriftCommand,
    },
    #[command(hide = true)]
    Reproduce {
        #[command(subcommand)]
        command: ReproduceCommand,
    },
    Security {
        #[command(subcommand)]
        command: SecurityCommand,
    },
    Runtime {
        #[command(subcommand)]
        command: RuntimeCommand,
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
    Plan(ReleasePlanArgs),
    CompatibilityCheck(ReleaseCompatibilityCheckArgs),
    UpgradePlan(ReleaseTransitionPlanArgs),
    RollbackPlan(ReleaseTransitionPlanArgs),
    Validate(ReleaseValidateArgs),
    Tag(ReleaseVersionCheckArgs),
    Notes(ReleaseChangelogGenerateArgs),
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
    Checksums {
        #[command(subcommand)]
        command: ReleaseChecksumsCommand,
    },
    Bundle {
        #[command(subcommand)]
        command: ReleaseBundleCommand,
    },
    ReadinessReport(ReleaseBundleBuildArgs),
    LaunchChecklist(ReleaseBundleBuildArgs),
    Sign(ReleaseSignArgs),
    Verify(ReleaseVerifyArgs),
    Diff(ReleaseDiffArgs),
    Packet(ReleasePacketArgs),
    Crates {
        #[command(subcommand)]
        command: ReleaseCratesCommand,
    },
    ApiSurface {
        #[command(subcommand)]
        command: ReleaseApiSurfaceCommand,
    },
    Semver {
        #[command(subcommand)]
        command: ReleaseSemverCommand,
    },
    Msrv {
        #[command(subcommand)]
        command: ReleaseMsrvCommand,
    },
    Images {
        #[command(subcommand)]
        command: ReleaseImagesCommand,
    },
    Ops {
        #[command(subcommand)]
        command: ReleaseOpsCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum InvariantsCommand {
    Run(InvariantsCommonArgs),
    List(InvariantsCommonArgs),
    Explain(InvariantsExplainArgs),
    Coverage(InvariantsCommonArgs),
    Docs(InvariantsCommonArgs),
}

#[derive(Subcommand, Debug)]
pub enum ObserveCommand {
    Metrics {
        #[command(subcommand)]
        command: ObserveMetricsCommand,
    },
    Dashboards {
        #[command(subcommand)]
        command: ObserveDashboardsCommand,
    },
    Logs {
        #[command(subcommand)]
        command: ObserveLogsCommand,
    },
    Traces {
        #[command(subcommand)]
        command: ObserveTracesCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum ObserveMetricsCommand {
    List(ObserveMetricsCommonArgs),
    Explain(ObserveMetricsExplainArgs),
    Docs(ObserveMetricsCommonArgs),
}

#[derive(Subcommand, Debug)]
pub enum ObserveDashboardsCommand {
    List(ObserveDashboardsCommonArgs),
    Verify(ObserveDashboardsCommonArgs),
    Explain(ObserveDashboardsCommonArgs),
}

#[derive(Subcommand, Debug)]
pub enum ApiCommand {
    List(ApiCommonArgs),
    Explain(ApiExplainArgs),
    Diff(ApiDiffArgs),
    Verify(ApiCommonArgs),
    Validate(ApiCommonArgs),
    Contract(ApiCommonArgs),
}

#[derive(Args, Debug, Clone)]
pub struct ObserveMetricsCommonArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ObserveMetricsExplainArgs {
    #[arg()]
    pub id_or_name: String,
    #[command(flatten)]
    pub common: ObserveMetricsCommonArgs,
}

#[derive(Args, Debug, Clone)]
pub struct ObserveDashboardsCommonArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ApiCommonArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ApiExplainArgs {
    #[arg()]
    pub endpoint: Option<String>,
    #[command(flatten)]
    pub common: ApiCommonArgs,
}

#[derive(Args, Debug, Clone)]
pub struct ApiDiffArgs {
    #[command(flatten)]
    pub common: ApiCommonArgs,
    #[arg(long)]
    pub baseline: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum ObserveTracesCommand {
    Explain(ObserveTracesCommonArgs),
    Verify(ObserveTracesCommonArgs),
    Coverage(ObserveTracesCommonArgs),
    Topology(ObserveTracesCommonArgs),
}

#[derive(Subcommand, Debug)]
pub enum ObserveLogsCommand {
    Explain(ObserveLogsCommonArgs),
}

#[derive(Args, Debug, Clone)]
pub struct ObserveLogsCommonArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ObserveTracesCommonArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum LoadCommand {
    Run(LoadCommonArgs),
    Compare(LoadCompareArgs),
    Baseline(LoadCommonArgs),
    Explain(LoadCommonArgs),
}

#[derive(Args, Debug, Clone)]
pub struct LoadCommonArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long, default_value = "mixed_workload")]
    pub scenario: String,
    #[arg(long, default_value_t = 300)]
    pub duration_secs: u32,
}

#[derive(Args, Debug, Clone)]
pub struct LoadCompareArgs {
    #[command(flatten)]
    pub common: LoadCommonArgs,
    #[arg(long)]
    pub baseline: Option<PathBuf>,
    #[arg(long)]
    pub current: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum DriftCommand {
    Detect(DriftDetectArgs),
    Explain(DriftExplainArgs),
    Report(DriftDetectArgs),
    Coverage(DriftDetectArgs),
    Baseline(DriftBaselineArgs),
    Compare(DriftCompareArgs),
}

#[derive(Args, Debug, Clone)]
pub struct DriftExplainArgs {
    #[arg()]
    pub drift_type: String,
    #[command(flatten)]
    pub common: InvariantsCommonArgs,
}

#[derive(Args, Debug, Clone)]
pub struct DriftDetectArgs {
    #[command(flatten)]
    pub common: InvariantsCommonArgs,
    #[arg(long)]
    pub ignore_file: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct DriftBaselineArgs {
    #[command(flatten)]
    pub detect: DriftDetectArgs,
    #[arg(long)]
    pub snapshot_out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct DriftCompareArgs {
    #[command(flatten)]
    pub detect: DriftDetectArgs,
    #[arg(long)]
    pub baseline: PathBuf,
    #[arg(long)]
    pub current: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
pub enum ReproduceCommand {
    Run(ReproduceCommonArgs),
    Verify(ReproduceCommonArgs),
    Explain(ReproduceExplainArgs),
    Status(ReproduceCommonArgs),
    AuditReport(ReproduceCommonArgs),
    Metrics(ReproduceCommonArgs),
    LineageValidate(ReproduceCommonArgs),
    SummaryTable(ReproduceCommonArgs),
}

#[derive(Args, Debug, Clone)]
pub struct ReproduceCommonArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReproduceExplainArgs {
    #[arg()]
    pub scenario: Option<String>,
    #[command(flatten)]
    pub common: ReproduceCommonArgs,
}

#[derive(Args, Debug, Clone)]
pub struct InvariantsCommonArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct InvariantsExplainArgs {
    #[arg()]
    pub id: String,
    #[command(flatten)]
    pub common: InvariantsCommonArgs,
}

#[derive(Subcommand, Debug)]
pub enum ReleaseCratesCommand {
    List(ReleaseCratesListArgs),
    ValidateMetadata(ReleaseCratesValidateArgs),
    ValidatePublishFlags(ReleaseCratesValidateArgs),
    DryRun(ReleaseCratesDryRunArgs),
    PublishPlan(ReleaseCratesListArgs),
}

#[derive(Subcommand, Debug)]
pub enum ReleaseApiSurfaceCommand {
    Snapshot(ReleaseApiSurfaceSnapshotArgs),
}

#[derive(Subcommand, Debug)]
pub enum ReleaseSemverCommand {
    Check(ReleaseSemverCheckArgs),
}

#[derive(Subcommand, Debug)]
pub enum ReleaseMsrvCommand {
    Verify(ReleaseMsrvVerifyArgs),
}

#[derive(Subcommand, Debug)]
pub enum ReleaseImagesCommand {
    ValidateLabels(ReleaseImagesValidateArgs),
    ValidateTags(ReleaseImagesValidateArgs),
    ValidateBaseDigests(ReleaseImagesValidateArgs),
    SbomVerify(ReleaseImagesValidateArgs),
    ProvenanceVerify(ReleaseImagesValidateArgs),
    ScanVerify(ReleaseImagesValidateArgs),
    SmokeVerify(ReleaseImagesValidateArgs),
    SizeReport(ReleaseImagesValidateArgs),
    RuntimeHardeningVerify(ReleaseImagesValidateArgs),
    RuntimeCommandVerify(ReleaseImagesValidateArgs),
    ManifestGenerate(ReleaseImagesManifestArgs),
    ManifestVerify(ReleaseImagesManifestArgs),
    ReleaseNotesCheck(ReleaseImagesNotesArgs),
    ChangelogExtract(ReleaseImagesChangelogArgs),
    IntegrationVerify(ReleaseImagesIntegrationArgs),
    BuildReproducibilityCheck(ReleaseImagesValidateArgs),
    LockedDependenciesVerify(ReleaseImagesValidateArgs),
    LockDriftVerify(ReleaseImagesValidateArgs),
    ReadinessSummary(ReleaseImagesValidateArgs),
}

#[derive(Subcommand, Debug)]
pub enum ReleaseOpsCommand {
    Package(ReleaseOpsPackageArgs),
    ValidatePackage(ReleaseOpsPackageArgs),
    Push(ReleaseOpsPushArgs),
    DigestVerify(ReleaseOpsPackageArgs),
    PullTest(ReleaseOpsPullTestArgs),
    BundleBuild(ReleaseOpsBundleArgs),
    BundleVerify(ReleaseOpsBundleArgs),
    ValuesCoverage(ReleaseOpsPackageArgs),
    ProfilesVerify(ReleaseOpsPackageArgs),
    LineageGenerate(ReleaseOpsPackageArgs),
    ProvenanceVerify(ReleaseOpsPackageArgs),
    ReadinessSummary(ReleaseOpsPackageArgs),
    ScenarioEvidenceVerify(ReleaseOpsPackageArgs),
    PublishPlan(ReleaseOpsPackageArgs),
}

#[derive(Subcommand, Debug)]
pub enum ReleaseChecksumsCommand {
    Generate(ReleaseCheckArgs),
    Verify(ReleaseCheckArgs),
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseCompatibilityCheckArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub from_version: Option<String>,
    #[arg(long)]
    pub to_version: Option<String>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseTransitionPlanArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub from_version: Option<String>,
    #[arg(long)]
    pub to_version: Option<String>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReleasePlanArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseValidateArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
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
    Check {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Rules {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    Report {
        #[arg(long)]
        repo_root: Option<PathBuf>,
        #[arg(long, value_enum, default_value_t = FormatArg::Text)]
        format: FormatArg,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    DoctrineReport {
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
    Adr {
        #[command(subcommand)]
        command: GovernanceAdrCommand,
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
pub enum GovernanceAdrCommand {
    Index {
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
    ConfigValidate(SecurityValidateArgs),
    Diagnostics(SecurityValidateArgs),
    PolicyInspect(SecurityPolicyInspectArgs),
    Audit(SecurityValidateArgs),
    VulnerabilityReport(SecurityValidateArgs),
    DependencyAudit(SecurityValidateArgs),
    IncidentReport(SecurityIncidentReportArgs),
    Authentication {
        #[command(subcommand)]
        command: SecurityAuthenticationCommand,
    },
    Authorization {
        #[command(subcommand)]
        command: SecurityAuthorizationCommand,
    },
    Compliance {
        #[command(subcommand)]
        command: SecurityComplianceCommand,
    },
    Threats {
        #[command(subcommand)]
        command: SecurityThreatCommand,
    },
    ScanArtifacts(SecurityScanArtifactsArgs),
}

#[derive(Subcommand, Debug)]
pub enum SecurityThreatCommand {
    List(SecurityValidateArgs),
    Explain(SecurityThreatExplainArgs),
    Verify(SecurityValidateArgs),
}

#[derive(Subcommand, Debug)]
pub enum SecurityAuthenticationCommand {
    ApiKeys(SecurityValidateArgs),
    TokenInspect(SecurityTokenInspectArgs),
    Diagnostics(SecurityValidateArgs),
    PolicyValidate(SecurityValidateArgs),
}

#[derive(Subcommand, Debug)]
pub enum SecurityAuthorizationCommand {
    Roles(SecurityValidateArgs),
    Permissions(SecurityValidateArgs),
    Diagnostics(SecurityValidateArgs),
    Assign(SecurityRoleAssignArgs),
    Validate(SecurityValidateArgs),
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
    ReplicaList(SystemClusterArgs),
    ReplicaHealth(SystemClusterArgs),
    ReplicaFailover(SystemClusterReplicaFailoverArgs),
    ReplicaDiagnostics(SystemClusterArgs),
    Failover(SystemClusterFailureActionArgs),
    RecoveryRun(SystemClusterArgs),
    ChaosTest(SystemClusterFailureActionArgs),
    ResilienceDiagnostics(SystemClusterArgs),
}

#[derive(Subcommand, Debug)]
pub enum AuditCommand {
    Run(AuditBundleArgs),
    Report(AuditBundleArgs),
    Explain(AuditBundleArgs),
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
    #[arg(
        long,
        default_value = "configs/ops/runtime/cluster-config.example.json"
    )]
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
pub struct SystemClusterReplicaFailoverArgs {
    #[command(flatten)]
    pub common: SystemClusterArgs,
    #[arg(long)]
    pub dataset_id: String,
    #[arg(long)]
    pub shard_id: String,
    #[arg(long)]
    pub promote_node_id: String,
}

#[derive(Args, Debug, Clone)]
pub struct SystemClusterFailureActionArgs {
    #[command(flatten)]
    pub common: SystemClusterArgs,
    #[arg(long)]
    pub target_id: Option<String>,
    #[arg(long, default_value = "node_crash")]
    pub fault_kind: String,
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

#[derive(Subcommand, Debug)]
pub enum RuntimeCommand {
    SelfCheck(RuntimeCommandArgs),
    PrintConfigSchema(RuntimeCommandArgs),
    ExplainConfigSchema(RuntimeCommandArgs),
}

#[derive(Args, Debug, Clone)]
pub struct RuntimeCommandArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub canonical: bool,
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
pub struct SecurityPolicyInspectArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long)]
    pub policy_id: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub struct SecurityTokenInspectArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub token: String,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct SecurityRoleAssignArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub principal: String,
    #[arg(long)]
    pub role_id: String,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct SecurityIncidentReportArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub incident_id: String,
    #[arg(long)]
    pub severity: String,
    #[arg(long)]
    pub summary: String,
    #[arg(long)]
    pub status: String,
    #[arg(long)]
    pub runbook: Option<String>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct SecurityThreatExplainArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg()]
    pub threat_id: Option<String>,
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
pub struct ReleaseCratesListArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseCratesValidateArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseCratesDryRunArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub enforce_size_budget: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseApiSurfaceSnapshotArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub crate_name: Option<String>,
    #[arg(long, default_value_t = false)]
    pub all: bool,
    #[arg(long, default_value_t = false)]
    pub write_golden: bool,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseSemverCheckArgs {
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
pub struct ReleaseMsrvVerifyArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseImagesValidateArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseImagesManifestArgs {
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
pub struct ReleaseImagesNotesArgs {
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
pub struct ReleaseImagesChangelogArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long)]
    pub from_ref: Option<String>,
    #[arg(long)]
    pub to_ref: Option<String>,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub allow_write: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseImagesIntegrationArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub allow_write: bool,
    #[arg(long, default_value_t = false)]
    pub by_digest: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseOpsPackageArgs {
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = FormatArg::Text)]
    pub format: FormatArg,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub allow_write: bool,
    #[arg(long, default_value_t = false)]
    pub allow_subprocess: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseOpsPushArgs {
    #[command(flatten)]
    pub common: ReleaseOpsPackageArgs,
    #[arg(long, default_value_t = false)]
    pub allow_network: bool,
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseOpsPullTestArgs {
    #[command(flatten)]
    pub common: ReleaseOpsPackageArgs,
    #[arg(long, default_value_t = false)]
    pub allow_network: bool,
}

#[derive(Args, Debug, Clone)]
pub struct ReleaseOpsBundleArgs {
    #[command(flatten)]
    pub common: ReleaseOpsPackageArgs,
    #[arg(long)]
    pub version: Option<String>,
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
