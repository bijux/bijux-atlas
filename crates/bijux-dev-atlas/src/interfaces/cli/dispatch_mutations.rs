// SPDX-License-Identifier: Apache-2.0

use crate::cli::{
    ApiCommand, ArtifactsCommand, AuditCommand, Command, ConfigsCommand, DatasetsCommand,
    DocsCommand, DriftCommand, FormatArg, IngestCommand, InvariantsCommand, LoadCommand,
    MakeCommand, MigrationsCommand, ObserveCommand, OpsCommand, PerfCommand, PoliciesCommand,
    RegistryCommand, ReleaseApiSurfaceCommand, ReleaseChecksumsCommand, ReleaseCommand,
    ReleaseCratesCommand, ReleaseImagesCommand, ReleaseMsrvCommand, ReleaseOpsCommand,
    ReleaseSemverCommand, ReportsCommand, ReproduceCommand, SecurityCommand, TestsCommand,
};

pub(super) fn force_json_output(command: &mut Command) {
    match command {
        Command::Version { format, .. } => *format = FormatArg::Json,
        Command::Help { format, .. } => *format = FormatArg::Json,
        Command::List { format, .. } => *format = FormatArg::Json,
        Command::Describe { format, .. } => *format = FormatArg::Json,
        Command::Run { format, .. } => *format = FormatArg::Json,
        Command::Suites { command } => match command {
            crate::cli::SuitesCommand::Run { format, .. }
            | crate::cli::SuitesCommand::Report { format, .. }
            | crate::cli::SuitesCommand::Diff { format, .. } => {
                *format = crate::cli::SuiteOutputFormatArg::Json
            }
            crate::cli::SuitesCommand::List { format, .. }
            | crate::cli::SuitesCommand::Describe { format, .. }
            | crate::cli::SuitesCommand::Last { format, .. }
            | crate::cli::SuitesCommand::History { format, .. }
            | crate::cli::SuitesCommand::Lint { format, .. } => *format = FormatArg::Json,
        },
        Command::Tests { command } => force_json_tests(command),
        Command::Registry { command } => match command {
            RegistryCommand::Status { format, .. } | RegistryCommand::Doctor { format, .. } => {
                *format = FormatArg::Json
            }
        },
        Command::Check { command } | Command::Checks { command } => match command {
            crate::cli::CheckCommand::List { format, .. }
            | crate::cli::CheckCommand::Explain { format, .. }
            | crate::cli::CheckCommand::Run { format, .. }
            | crate::cli::CheckCommand::Doctor { format, .. } => *format = FormatArg::Json,
        },
        Command::Ops { command } => force_json_ops(command),
        Command::Docs { command } => force_json_docs(command),
        Command::Make { command } => force_json_make(command),
        Command::Artifacts { command } => force_json_artifacts(command),
        Command::Reports { command } => match command {
            ReportsCommand::List(args) => args.format = FormatArg::Json,
            ReportsCommand::Index(args) => args.format = FormatArg::Json,
            ReportsCommand::Progress(args) => args.format = FormatArg::Json,
            ReportsCommand::Validate(args) => args.format = FormatArg::Json,
        },
        Command::Demo { command } => force_json_demo(command),
        Command::Configs { command } => force_json_configs(command),
        Command::Governance { command } => force_json_governance(command),
        Command::Security { command } => force_json_security(command),
        Command::Runtime { command } => force_json_runtime(command),
        Command::Tutorials { command } => force_json_tutorials(command),
        Command::Migrations { command } => force_json_migrations(command),
        Command::System { command } => force_json_system(command),
        Command::Audit { command } => force_json_audit(command),
        Command::Observe { command } => force_json_observe(command),
        Command::Api { command } => force_json_api(command),
        Command::Load { command } => force_json_load(command),
        Command::Invariants { command } => force_json_invariants(command),
        Command::Drift { command } => force_json_drift(command),
        Command::Reproduce { command } => force_json_reproduce(command),
        Command::Datasets { command } => force_json_datasets(command),
        Command::Ingest { command } => force_json_ingest(command),
        Command::Perf { command } => force_json_perf(command),
        Command::Policies { command } => force_json_policies(command),
        Command::Docker { command } => force_json_docker(command),
        Command::Validate { format, .. } => *format = FormatArg::Json,
        Command::Release { command } => match command {
            ReleaseCommand::Plan(args) => args.format = FormatArg::Json,
            ReleaseCommand::CompatibilityCheck(args) => args.format = FormatArg::Json,
            ReleaseCommand::UpgradePlan(args) => args.format = FormatArg::Json,
            ReleaseCommand::RollbackPlan(args) => args.format = FormatArg::Json,
            ReleaseCommand::Validate(args) => args.format = FormatArg::Json,
            ReleaseCommand::Tag(args) => args.format = FormatArg::Json,
            ReleaseCommand::Notes(args) => args.format = FormatArg::Json,
            ReleaseCommand::Check(args) => args.format = FormatArg::Json,
            ReleaseCommand::Rebuild { command } => match command {
                crate::cli::ReleaseRebuildCommand::Verify(args) => args.format = FormatArg::Json,
            },
            ReleaseCommand::Reproducibility { command } => match command {
                crate::cli::ReleaseReproducibilityCommand::Report(args) => {
                    args.format = FormatArg::Json
                }
            },
            ReleaseCommand::Version { command } => match command {
                crate::cli::ReleaseVersionCommand::Check(args) => args.format = FormatArg::Json,
            },
            ReleaseCommand::Changelog { command } => match command {
                crate::cli::ReleaseChangelogCommand::Generate(args) => {
                    args.format = FormatArg::Json
                }
                crate::cli::ReleaseChangelogCommand::Validate(args) => {
                    args.format = FormatArg::Json
                }
            },
            ReleaseCommand::Manifest { command } => match command {
                crate::cli::ReleaseManifestCommand::Generate(args) => args.format = FormatArg::Json,
                crate::cli::ReleaseManifestCommand::Validate(args) => args.format = FormatArg::Json,
            },
            ReleaseCommand::Checksums { command } => match command {
                ReleaseChecksumsCommand::Generate(args) | ReleaseChecksumsCommand::Verify(args) => {
                    args.format = FormatArg::Json
                }
            },
            ReleaseCommand::Bundle { command } => match command {
                crate::cli::ReleaseBundleCommand::Build(args) => args.format = FormatArg::Json,
                crate::cli::ReleaseBundleCommand::Verify(args) => args.format = FormatArg::Json,
                crate::cli::ReleaseBundleCommand::Hash(args) => args.format = FormatArg::Json,
            },
            ReleaseCommand::ReadinessReport(args) | ReleaseCommand::LaunchChecklist(args) => {
                args.format = FormatArg::Json
            }
            ReleaseCommand::Sign(args) => args.format = FormatArg::Json,
            ReleaseCommand::Verify(args) => args.format = FormatArg::Json,
            ReleaseCommand::Diff(args) => args.format = FormatArg::Json,
            ReleaseCommand::Packet(args) => args.format = FormatArg::Json,
            ReleaseCommand::Crates { command } => match command {
                ReleaseCratesCommand::List(args) => args.format = FormatArg::Json,
                ReleaseCratesCommand::ValidateMetadata(args) => args.format = FormatArg::Json,
                ReleaseCratesCommand::ValidatePublishFlags(args) => args.format = FormatArg::Json,
                ReleaseCratesCommand::DryRun(args) => args.format = FormatArg::Json,
                ReleaseCratesCommand::PublishPlan(args) => args.format = FormatArg::Json,
            },
            ReleaseCommand::ApiSurface { command } => match command {
                ReleaseApiSurfaceCommand::Snapshot(args) => args.format = FormatArg::Json,
            },
            ReleaseCommand::Semver { command } => match command {
                ReleaseSemverCommand::Check(args) => args.format = FormatArg::Json,
            },
            ReleaseCommand::Msrv { command } => match command {
                ReleaseMsrvCommand::Verify(args) => args.format = FormatArg::Json,
            },
            ReleaseCommand::Images { command } => match command {
                ReleaseImagesCommand::ValidateLabels(args)
                | ReleaseImagesCommand::ValidateTags(args)
                | ReleaseImagesCommand::ValidateBaseDigests(args)
                | ReleaseImagesCommand::SbomVerify(args)
                | ReleaseImagesCommand::ProvenanceVerify(args)
                | ReleaseImagesCommand::ScanVerify(args)
                | ReleaseImagesCommand::SmokeVerify(args)
                | ReleaseImagesCommand::SizeReport(args)
                | ReleaseImagesCommand::RuntimeHardeningVerify(args)
                | ReleaseImagesCommand::RuntimeCommandVerify(args) => args.format = FormatArg::Json,
                ReleaseImagesCommand::ManifestGenerate(args)
                | ReleaseImagesCommand::ManifestVerify(args) => args.format = FormatArg::Json,
                ReleaseImagesCommand::ReleaseNotesCheck(args) => args.format = FormatArg::Json,
                ReleaseImagesCommand::ChangelogExtract(args) => args.format = FormatArg::Json,
                ReleaseImagesCommand::IntegrationVerify(args) => args.format = FormatArg::Json,
                ReleaseImagesCommand::BuildReproducibilityCheck(args)
                | ReleaseImagesCommand::LockedDependenciesVerify(args)
                | ReleaseImagesCommand::LockDriftVerify(args)
                | ReleaseImagesCommand::ReadinessSummary(args) => args.format = FormatArg::Json,
            },
            ReleaseCommand::Ops { command } => match command {
                ReleaseOpsCommand::Package(args)
                | ReleaseOpsCommand::ValidatePackage(args)
                | ReleaseOpsCommand::CompatibilityMatrix(args)
                | ReleaseOpsCommand::DigestVerify(args)
                | ReleaseOpsCommand::ValuesCoverage(args)
                | ReleaseOpsCommand::ProfilesVerify(args)
                | ReleaseOpsCommand::LineageGenerate(args)
                | ReleaseOpsCommand::ProvenanceVerify(args)
                | ReleaseOpsCommand::ReadinessSummary(args)
                | ReleaseOpsCommand::ScenarioEvidenceVerify(args)
                | ReleaseOpsCommand::PublishPlan(args) => args.format = FormatArg::Json,
                ReleaseOpsCommand::Push(args) => args.common.format = FormatArg::Json,
                ReleaseOpsCommand::PullTest(args) => args.common.format = FormatArg::Json,
                ReleaseOpsCommand::BundleBuild(args) | ReleaseOpsCommand::BundleVerify(args) => {
                    args.common.format = FormatArg::Json
                }
            },
        },
        Command::Build { .. }
        | Command::Ci { .. }
        | Command::Workflows { .. }
        | Command::Gates { .. }
        | Command::Capabilities { .. } => {}
    }
}

fn force_json_invariants(command: &mut InvariantsCommand) {
    match command {
        InvariantsCommand::Run(args)
        | InvariantsCommand::List(args)
        | InvariantsCommand::Coverage(args)
        | InvariantsCommand::Docs(args) => args.format = FormatArg::Json,
        InvariantsCommand::Explain(args) => args.common.format = FormatArg::Json,
    }
}

fn force_json_drift(command: &mut DriftCommand) {
    match command {
        DriftCommand::Detect(args) | DriftCommand::Report(args) => {
            args.common.format = FormatArg::Json
        }
        DriftCommand::Coverage(args) => args.common.format = FormatArg::Json,
        DriftCommand::Explain(args) => args.common.format = FormatArg::Json,
        DriftCommand::Baseline(args) => args.detect.common.format = FormatArg::Json,
        DriftCommand::Compare(args) => args.detect.common.format = FormatArg::Json,
    }
}

fn force_json_reproduce(command: &mut ReproduceCommand) {
    match command {
        ReproduceCommand::Run(args)
        | ReproduceCommand::Verify(args)
        | ReproduceCommand::Status(args)
        | ReproduceCommand::AuditReport(args)
        | ReproduceCommand::Metrics(args)
        | ReproduceCommand::LineageValidate(args)
        | ReproduceCommand::SummaryTable(args) => args.format = FormatArg::Json,
        ReproduceCommand::Explain(args) => args.common.format = FormatArg::Json,
    }
}

fn force_json_artifacts(command: &mut ArtifactsCommand) {
    match command {
        ArtifactsCommand::Clean(common) => common.format = FormatArg::Json,
        ArtifactsCommand::Gc(args) => args.common.format = FormatArg::Json,
        ArtifactsCommand::Report { command } => match command {
            crate::cli::ArtifactsReportCommand::Inventory(common) => {
                common.format = FormatArg::Json
            }
            crate::cli::ArtifactsReportCommand::Manifest(args)
            | crate::cli::ArtifactsReportCommand::Index(args)
            | crate::cli::ArtifactsReportCommand::Validate(args) => {
                args.common.format = FormatArg::Json
            }
            crate::cli::ArtifactsReportCommand::Read(args) => args.common.format = FormatArg::Json,
            crate::cli::ArtifactsReportCommand::Diff(args) => args.common.format = FormatArg::Json,
        },
    }
}

fn force_json_demo(command: &mut crate::cli::DemoCommand) {
    match command {
        crate::cli::DemoCommand::Quickstart(args) => args.format = FormatArg::Json,
    }
}

fn force_json_ops(command: &mut OpsCommand) {
    match command {
        OpsCommand::Logs { command }
        | OpsCommand::Describe { command }
        | OpsCommand::Events { command } => match command {
            crate::cli::OpsCollectCommand::Collect(args) => args.common.format = FormatArg::Json,
        },
        OpsCommand::Resources { command } => match command {
            crate::cli::OpsResourcesCommand::Snapshot(args) => args.common.format = FormatArg::Json,
        },
        OpsCommand::Kind { command } => match command {
            crate::cli::OpsKindCommand::Up(common)
            | crate::cli::OpsKindCommand::Down(common)
            | crate::cli::OpsKindCommand::Status(common) => common.format = FormatArg::Json,
            crate::cli::OpsKindCommand::PreloadImage(args) => args.common.format = FormatArg::Json,
            crate::cli::OpsKindCommand::Install(args) => args.common.format = FormatArg::Json,
            crate::cli::OpsKindCommand::Upgrade(args) => {
                args.release.common.format = FormatArg::Json
            }
            crate::cli::OpsKindCommand::Rollback(args) => {
                args.release.common.format = FormatArg::Json
            }
            crate::cli::OpsKindCommand::Smoke(args) => args.common.format = FormatArg::Json,
        },
        OpsCommand::Helm { command } => match command {
            crate::cli::OpsHelmCommand::Install(args) => {
                args.release.common.format = FormatArg::Json
            }
            crate::cli::OpsHelmCommand::Uninstall(args) => args.common.format = FormatArg::Json,
            crate::cli::OpsHelmCommand::Upgrade(args) => {
                args.release.common.format = FormatArg::Json
            }
            crate::cli::OpsHelmCommand::Rollback(args) => {
                args.release.common.format = FormatArg::Json
            }
        },
        OpsCommand::List(common)
        | OpsCommand::Doctor(common)
        | OpsCommand::Validate(common)
        | OpsCommand::Graph(common)
        | OpsCommand::Inventory(common)
        | OpsCommand::Docs(common)
        | OpsCommand::DocsVerify(common)
        | OpsCommand::Conformance(common)
        | OpsCommand::Report(common)
        | OpsCommand::HelmEnv(crate::cli::OpsHelmEnvArgs { common, .. })
        | OpsCommand::Readiness(common)
        | OpsCommand::ListProfiles(common)
        | OpsCommand::ListTools(common)
        | OpsCommand::VerifyTools(common)
        | OpsCommand::ListActions(common)
        | OpsCommand::Plan(common)
        | OpsCommand::ReleasePlan(common)
        | OpsCommand::InstallPlan(common)
        | OpsCommand::Up(common)
        | OpsCommand::Down(common)
        | OpsCommand::Clean(common)
        | OpsCommand::Cleanup(common)
        | OpsCommand::K8sPlan(common)
        | OpsCommand::K8sEnvSurface(common)
        | OpsCommand::K8sValidateProfiles(common)
        | OpsCommand::Profiles {
            command:
                crate::cli::OpsProfilesCommand::Validate(crate::cli::OpsProfilesValidateArgs {
                    common,
                    ..
                }),
        }
        | OpsCommand::Profiles {
            command:
                crate::cli::OpsProfilesCommand::SchemaValidate(crate::cli::OpsProfileValidationArgs {
                    common,
                    ..
                })
                | crate::cli::OpsProfilesCommand::Kubeconform(crate::cli::OpsProfileValidationArgs {
                    common,
                    ..
                })
                | crate::cli::OpsProfilesCommand::RolloutSafetyValidate(
                    crate::cli::OpsProfileValidationArgs { common, .. },
                )
                | crate::cli::OpsProfilesCommand::PolicyValidate(crate::cli::OpsProfileValidationArgs {
                    common,
                    ..
                })
                | crate::cli::OpsProfilesCommand::ResourceValidate(crate::cli::OpsProfileValidationArgs {
                    common,
                    ..
                })
                | crate::cli::OpsProfilesCommand::SecuritycontextValidate(
                    crate::cli::OpsProfileValidationArgs { common, .. },
                )
                | crate::cli::OpsProfilesCommand::ServiceMonitorValidate(
                    crate::cli::OpsProfileValidationArgs { common, .. },
                )
                | crate::cli::OpsProfilesCommand::HpaValidate(crate::cli::OpsProfileValidationArgs {
                    common,
                    ..
                }),
        }
        | OpsCommand::Profile {
            command: crate::cli::OpsProfileCommand::List(common),
        }
        | OpsCommand::K8sDryRun(common)
        | OpsCommand::K8sPorts(common)
        | OpsCommand::K8sConformance(common)
        | OpsCommand::LoadPlan { common, .. }
        | OpsCommand::LoadRun { common, .. }
        | OpsCommand::LoadReport { common, .. } => common.format = FormatArg::Json,
        OpsCommand::Explain { common, .. }
        | OpsCommand::Profile {
            command: crate::cli::OpsProfileCommand::Explain { common, .. },
        } => common.format = FormatArg::Json,
        OpsCommand::Render(args) => args.common.format = FormatArg::Json,
        OpsCommand::Package(args) => args.common.format = FormatArg::Json,
        OpsCommand::Install(args) => args.common.format = FormatArg::Json,
        OpsCommand::Smoke(args) => args.common.format = FormatArg::Json,
        OpsCommand::Status(args) => args.common.format = FormatArg::Json,
        OpsCommand::ExplainProfile { common, .. } => common.format = FormatArg::Json,
        OpsCommand::Reset(args) => args.common.format = FormatArg::Json,
        OpsCommand::K8sApply(args) => args.common.format = FormatArg::Json,
        OpsCommand::K8sWait(args) => args.common.format = FormatArg::Json,
        OpsCommand::K8sLogs(args) => args.common.format = FormatArg::Json,
        OpsCommand::K8sPortForward(args) => args.common.format = FormatArg::Json,
        OpsCommand::Pins { command } => match command {
            crate::cli::OpsPinsCommand::Check(common)
            | crate::cli::OpsPinsCommand::Update { common, .. } => common.format = FormatArg::Json,
        },
        OpsCommand::Generate { command } => match command {
            crate::cli::OpsGenerateCommand::PinsIndex { common, .. }
            | crate::cli::OpsGenerateCommand::SurfaceList { common, .. }
            | crate::cli::OpsGenerateCommand::Runbook { common, .. }
            | crate::cli::OpsGenerateCommand::ChartDependencySbom { common, .. }
            | crate::cli::OpsGenerateCommand::ResilienceReport { common, .. } => {
                common.format = FormatArg::Json
            }
        },
        OpsCommand::Evidence { command } => match command {
            crate::cli::OpsEvidenceCommand::Collect(common) => common.format = FormatArg::Json,
            crate::cli::OpsEvidenceCommand::Summarize(args) => args.common.format = FormatArg::Json,
            crate::cli::OpsEvidenceCommand::Verify(args) => args.common.format = FormatArg::Json,
            crate::cli::OpsEvidenceCommand::Diff(args) => args.common.format = FormatArg::Json,
        },
        OpsCommand::Diagnose { command } => match command {
            crate::cli::OpsDiagnoseCommand::Bundle(args) => args.common.format = FormatArg::Json,
            crate::cli::OpsDiagnoseCommand::Explain(args) => args.common.format = FormatArg::Json,
            crate::cli::OpsDiagnoseCommand::Redact(args) => args.common.format = FormatArg::Json,
        },
        OpsCommand::Drills { command } => match command {
            crate::cli::OpsDrillsCommand::Run(args) => args.common.format = FormatArg::Json,
        },
        OpsCommand::Datasets { command } => match command {
            crate::cli::OpsDatasetsCommand::List(common)
            | crate::cli::OpsDatasetsCommand::Ingest(common)
            | crate::cli::OpsDatasetsCommand::Publish(common)
            | crate::cli::OpsDatasetsCommand::Promote(common)
            | crate::cli::OpsDatasetsCommand::Rollback(common)
            | crate::cli::OpsDatasetsCommand::Qc(common) => common.format = FormatArg::Json,
        },
        OpsCommand::Schema { command } => match command {
            crate::cli::OpsSchemaCommand::Validate(common)
            | crate::cli::OpsSchemaCommand::Diff(common)
            | crate::cli::OpsSchemaCommand::Coverage(common)
            | crate::cli::OpsSchemaCommand::RegenIndex(common) => common.format = FormatArg::Json,
        },
        OpsCommand::InventoryDomain { command } => match command {
            crate::cli::OpsInventoryCommand::Validate(common)
            | crate::cli::OpsInventoryCommand::Graph(common)
            | crate::cli::OpsInventoryCommand::Diff(common)
            | crate::cli::OpsInventoryCommand::Coverage(common)
            | crate::cli::OpsInventoryCommand::OrphanCheck(common) => {
                common.format = FormatArg::Json
            }
        },
        OpsCommand::ReportDomain { command } => match command {
            crate::cli::OpsReportCommand::Generate(common)
            | crate::cli::OpsReportCommand::Diff(common)
            | crate::cli::OpsReportCommand::Readiness(common)
            | crate::cli::OpsReportCommand::Bundle(common) => common.format = FormatArg::Json,
        },
        OpsCommand::Tools { command } => match command {
            crate::cli::OpsToolsCommand::List(common)
            | crate::cli::OpsToolsCommand::Verify(common)
            | crate::cli::OpsToolsCommand::Doctor(common) => common.format = FormatArg::Json,
        },
        OpsCommand::Suite { command } => match command {
            crate::cli::OpsSuiteCommand::List(common)
            | crate::cli::OpsSuiteCommand::Run { common, .. } => common.format = FormatArg::Json,
        },
        OpsCommand::Stack { command } => match command {
            crate::cli::OpsStackCommand::Plan(common)
            | crate::cli::OpsStackCommand::Up(common)
            | crate::cli::OpsStackCommand::Down(common)
            | crate::cli::OpsStackCommand::Logs(common)
            | crate::cli::OpsStackCommand::Ports(common)
            | crate::cli::OpsStackCommand::Versions(common)
            | crate::cli::OpsStackCommand::Doctor(common) => common.format = FormatArg::Json,
            crate::cli::OpsStackCommand::Status(args) => args.common.format = FormatArg::Json,
            crate::cli::OpsStackCommand::Reset(args) => args.common.format = FormatArg::Json,
        },
        OpsCommand::K8s { command } => match command {
            crate::cli::OpsK8sCommand::Render(args) => args.common.format = FormatArg::Json,
            crate::cli::OpsK8sCommand::Validate(common)
            | crate::cli::OpsK8sCommand::EnvSurface(common)
            | crate::cli::OpsK8sCommand::ValidateProfiles(common)
            | crate::cli::OpsK8sCommand::Plan(common)
            | crate::cli::OpsK8sCommand::Uninstall(common)
            | crate::cli::OpsK8sCommand::Ports(common)
            | crate::cli::OpsK8sCommand::Diff(common)
            | crate::cli::OpsK8sCommand::Rollout(common)
            | crate::cli::OpsK8sCommand::DryRun(common)
            | crate::cli::OpsK8sCommand::Conformance(common)
            | crate::cli::OpsK8sCommand::Test(common) => common.format = FormatArg::Json,
            crate::cli::OpsK8sCommand::Smoke(args) => args.common.format = FormatArg::Json,
            crate::cli::OpsK8sCommand::Install(args) => args.common.format = FormatArg::Json,
            crate::cli::OpsK8sCommand::Apply(args) => args.common.format = FormatArg::Json,
            crate::cli::OpsK8sCommand::Wait(args) => args.common.format = FormatArg::Json,
            crate::cli::OpsK8sCommand::Logs(args) => args.common.format = FormatArg::Json,
            crate::cli::OpsK8sCommand::PortForward(args) => args.common.format = FormatArg::Json,
            crate::cli::OpsK8sCommand::Status(args) => args.common.format = FormatArg::Json,
        },
        OpsCommand::Load { command } => match command {
            crate::cli::OpsLoadCommand::Plan { common, .. }
            | crate::cli::OpsLoadCommand::Run { common, .. }
            | crate::cli::OpsLoadCommand::Report { common, .. }
            | crate::cli::OpsLoadCommand::Evaluate(common)
            | crate::cli::OpsLoadCommand::ListSuites(common) => common.format = FormatArg::Json,
            crate::cli::OpsLoadCommand::Baseline { command } => match command {
                crate::cli::OpsLoadBaselineCommand::Update(common) => {
                    common.format = FormatArg::Json
                }
            },
        },
        OpsCommand::E2e { command } => match command {
            crate::cli::OpsE2eCommand::Run(common)
            | crate::cli::OpsE2eCommand::Smoke(common)
            | crate::cli::OpsE2eCommand::Realdata(common)
            | crate::cli::OpsE2eCommand::ListSuites(common) => common.format = FormatArg::Json,
        },
        OpsCommand::Scenario { command } => match command {
            crate::cli::OpsScenarioCommand::Run(args) => args.common.format = FormatArg::Json,
            crate::cli::OpsScenarioCommand::List(common) => common.format = FormatArg::Json,
        },
        OpsCommand::Obs { command } => match command {
            crate::cli::OpsObsCommand::Up(common)
            | crate::cli::OpsObsCommand::Down(common)
            | crate::cli::OpsObsCommand::Validate(common)
            | crate::cli::OpsObsCommand::Snapshot(common)
            | crate::cli::OpsObsCommand::Dashboards(common)
            | crate::cli::OpsObsCommand::Verify(common) => common.format = FormatArg::Json,
            crate::cli::OpsObsCommand::Slo { command } => match command {
                crate::cli::OpsObsSloCommand::List(common)
                | crate::cli::OpsObsSloCommand::Verify(common) => common.format = FormatArg::Json,
            },
            crate::cli::OpsObsCommand::Alerts { command } => match command {
                crate::cli::OpsObsAlertsCommand::Verify(common) => common.format = FormatArg::Json,
            },
            crate::cli::OpsObsCommand::Runbooks { command } => match command {
                crate::cli::OpsObsRunbooksCommand::Verify(common) => {
                    common.format = FormatArg::Json
                }
            },
            crate::cli::OpsObsCommand::Drill { command } => match command {
                crate::cli::OpsObsDrillCommand::Run(common) => common.format = FormatArg::Json,
            },
        },
    }
}

fn force_json_docs(command: &mut DocsCommand) {
    match command {
        DocsCommand::Check(common)
        | DocsCommand::VerifyContracts(common)
        | DocsCommand::Doctor(common)
        | DocsCommand::DeployPlan(common)
        | DocsCommand::UxSmoke(common)
        | DocsCommand::Where(common)
        | DocsCommand::SiteDir(common)
        | DocsCommand::Validate(common)
        | DocsCommand::Build(common)
        | DocsCommand::Clean(common)
        | DocsCommand::Lint(common)
        | DocsCommand::IncludesCheck(common)
        | DocsCommand::Links(common)
        | DocsCommand::NavIntegrity(common)
        | DocsCommand::ExternalLinks(crate::cli::DocsExternalLinksArgs { common, .. })
        | DocsCommand::Inventory(common)
        | DocsCommand::Graph(common)
        | DocsCommand::Dead(common)
        | DocsCommand::Duplicates(common)
        | DocsCommand::PrunePlan(common)
        | DocsCommand::DedupeReport(common)
        | DocsCommand::ShrinkReport(common)
        | DocsCommand::HealthDashboard(common)
        | DocsCommand::VerifyGenerated(common) => common.format = FormatArg::Json,
        DocsCommand::Top(args) => args.common.format = FormatArg::Json,
        DocsCommand::PagesSmoke(args) => args.common.format = FormatArg::Json,
        DocsCommand::Serve(args) => args.common.format = FormatArg::Json,
        DocsCommand::Grep(args) => args.common.format = FormatArg::Json,
        DocsCommand::LifecycleSummaryTable(args) | DocsCommand::DrillSummaryTable(args) => {
            args.common.format = FormatArg::Json
        }
        DocsCommand::Redirects { command } => match command {
            crate::cli::DocsRedirectsCommand::Sync(common) => common.format = FormatArg::Json,
        },
        DocsCommand::Merge { command } => match command {
            crate::cli::DocsMergeCommand::Validate(common) => common.format = FormatArg::Json,
        },
        DocsCommand::Spine { command } => match command {
            crate::cli::DocsSpineCommand::Validate(common)
            | crate::cli::DocsSpineCommand::Report(common) => common.format = FormatArg::Json,
        },
        DocsCommand::Toc { command } => match command {
            crate::cli::DocsTocCommand::Verify(common) => common.format = FormatArg::Json,
        },
        DocsCommand::Reference { command } => match command {
            crate::cli::DocsReferenceCommand::Generate(common)
            | crate::cli::DocsReferenceCommand::Check(common) => common.format = FormatArg::Json,
        },
        DocsCommand::Generate { command } => match command {
            crate::cli::DocsGenerateCommand::Examples(common)
            | crate::cli::DocsGenerateCommand::CommandLists(common)
            | crate::cli::DocsGenerateCommand::SchemaSnippets(common)
            | crate::cli::DocsGenerateCommand::OpenapiSnippets(common)
            | crate::cli::DocsGenerateCommand::OpsSnippets(common)
            | crate::cli::DocsGenerateCommand::RealDataPages(common) => {
                common.format = FormatArg::Json
            }
        },
    }
}

fn force_json_make(command: &mut MakeCommand) {
    match command {
        MakeCommand::VerifyModule(args) => args.common.format = FormatArg::Json,
        MakeCommand::Wrappers { command } => match command {
            crate::cli::MakeWrappersCommand::Verify(common) => common.format = FormatArg::Json,
        },
        MakeCommand::Surface(common)
        | MakeCommand::List(common)
        | MakeCommand::Explain(crate::cli::MakeExplainArgs { common, .. })
        | MakeCommand::TargetList(common)
        | MakeCommand::LintPolicyReport(common) => common.format = FormatArg::Json,
    }
}

fn force_json_configs(command: &mut ConfigsCommand) {
    match command {
        ConfigsCommand::Print(common)
        | ConfigsCommand::List(common)
        | ConfigsCommand::Graph(common)
        | ConfigsCommand::Explain(crate::cli::ConfigsExplainArgs { common, .. })
        | ConfigsCommand::Verify(common)
        | ConfigsCommand::Doctor(common)
        | ConfigsCommand::Validate(common)
        | ConfigsCommand::Lint(common)
        | ConfigsCommand::Inventory(common)
        | ConfigsCommand::Compile(common)
        | ConfigsCommand::Diff(common) => common.format = FormatArg::Json,
        ConfigsCommand::Fmt { common, .. } => common.format = FormatArg::Json,
    }
}

fn force_json_governance(command: &mut crate::cli::GovernanceCommand) {
    match command {
        crate::cli::GovernanceCommand::Version { format, .. }
        | crate::cli::GovernanceCommand::List { format, .. }
        | crate::cli::GovernanceCommand::Explain { format, .. }
        | crate::cli::GovernanceCommand::Validate { format, .. }
        | crate::cli::GovernanceCommand::Check { format, .. }
        | crate::cli::GovernanceCommand::Rules { format, .. }
        | crate::cli::GovernanceCommand::Report { format, .. }
        | crate::cli::GovernanceCommand::DoctrineReport { format, .. }
        | crate::cli::GovernanceCommand::Doctor { format, .. } => *format = FormatArg::Json,
        crate::cli::GovernanceCommand::Exceptions { command } => match command {
            crate::cli::GovernanceExceptionsCommand::List { format, .. }
            | crate::cli::GovernanceExceptionsCommand::Validate { format, .. } => {
                *format = FormatArg::Json
            }
        },
        crate::cli::GovernanceCommand::Deprecations { command } => match command {
            crate::cli::GovernanceDeprecationsCommand::Validate { format, .. } => {
                *format = FormatArg::Json
            }
        },
        crate::cli::GovernanceCommand::Breaking { command } => match command {
            crate::cli::GovernanceBreakingCommand::Validate { format, .. } => {
                *format = FormatArg::Json
            }
        },
        crate::cli::GovernanceCommand::Adr { command } => match command {
            crate::cli::GovernanceAdrCommand::Index { format, .. } => *format = FormatArg::Json,
        },
    }
}

fn force_json_policies(command: &mut PoliciesCommand) {
    match command {
        PoliciesCommand::List { format, .. }
        | PoliciesCommand::Explain { format, .. }
        | PoliciesCommand::Report { format, .. }
        | PoliciesCommand::Print { format, .. }
        | PoliciesCommand::Validate { format, .. } => *format = FormatArg::Json,
    }
}

fn force_json_security(command: &mut SecurityCommand) {
    match command {
        SecurityCommand::Validate(args)
        | SecurityCommand::ConfigValidate(args)
        | SecurityCommand::Diagnostics(args)
        | SecurityCommand::Audit(args)
        | SecurityCommand::VulnerabilityReport(args)
        | SecurityCommand::DependencyAudit(args) => args.format = FormatArg::Json,
        SecurityCommand::PolicyInspect(args) => args.format = FormatArg::Json,
        SecurityCommand::IncidentReport(args) => args.format = FormatArg::Json,
        SecurityCommand::Authentication { command } => match command {
            crate::cli::SecurityAuthenticationCommand::ApiKeys(args)
            | crate::cli::SecurityAuthenticationCommand::Diagnostics(args)
            | crate::cli::SecurityAuthenticationCommand::PolicyValidate(args) => {
                args.format = FormatArg::Json
            }
            crate::cli::SecurityAuthenticationCommand::TokenInspect(args) => {
                args.format = FormatArg::Json
            }
        },
        SecurityCommand::Authorization { command } => match command {
            crate::cli::SecurityAuthorizationCommand::Roles(args)
            | crate::cli::SecurityAuthorizationCommand::Permissions(args)
            | crate::cli::SecurityAuthorizationCommand::Diagnostics(args)
            | crate::cli::SecurityAuthorizationCommand::Validate(args) => {
                args.format = FormatArg::Json
            }
            crate::cli::SecurityAuthorizationCommand::Assign(args) => args.format = FormatArg::Json,
        },
        SecurityCommand::Compliance { command } => match command {
            crate::cli::SecurityComplianceCommand::Validate(args) => args.format = FormatArg::Json,
        },
        SecurityCommand::Threats { command } => match command {
            crate::cli::SecurityThreatCommand::List(args)
            | crate::cli::SecurityThreatCommand::Verify(args) => args.format = FormatArg::Json,
            crate::cli::SecurityThreatCommand::Explain(args) => args.format = FormatArg::Json,
        },
        SecurityCommand::ScanArtifacts(args) => args.format = FormatArg::Json,
    }
}

fn force_json_runtime(command: &mut crate::cli::RuntimeCommand) {
    match command {
        crate::cli::RuntimeCommand::SelfCheck(args)
        | crate::cli::RuntimeCommand::PrintConfigSchema(args)
        | crate::cli::RuntimeCommand::ExplainConfigSchema(args) => args.format = FormatArg::Json,
    }
}

fn force_json_tutorials(command: &mut crate::cli::TutorialsCommand) {
    match command {
        crate::cli::TutorialsCommand::List(args)
        | crate::cli::TutorialsCommand::Explain(args)
        | crate::cli::TutorialsCommand::Verify(args)
        | crate::cli::TutorialsCommand::ReproducibilityCheck(args)
        | crate::cli::TutorialsCommand::Generate(args) => args.format = FormatArg::Json,
        crate::cli::TutorialsCommand::Run { command } => match command {
            crate::cli::TutorialsRunCommand::Workflow(args) => args.common.format = FormatArg::Json,
            crate::cli::TutorialsRunCommand::DatasetE2e(args) => {
                args.common.format = FormatArg::Json
            }
        },
        crate::cli::TutorialsCommand::Build { command } => match command {
            crate::cli::TutorialsBuildCommand::Docs(args) => args.common.format = FormatArg::Json,
        },
        crate::cli::TutorialsCommand::Dataset { command } => match command {
            crate::cli::TutorialsDatasetCommand::Package(args) => {
                args.common.format = FormatArg::Json
            }
            crate::cli::TutorialsDatasetCommand::Ingest(args)
            | crate::cli::TutorialsDatasetCommand::IntegrityCheck(args) => {
                args.format = FormatArg::Json
            }
        },
        crate::cli::TutorialsCommand::Workspace { command } => match command {
            crate::cli::TutorialsWorkspaceCommand::Cleanup(args) => {
                args.common.format = FormatArg::Json
            }
        },
        crate::cli::TutorialsCommand::Dashboards { command } => match command {
            crate::cli::TutorialsDashboardsCommand::Validate(args) => args.format = FormatArg::Json,
        },
        crate::cli::TutorialsCommand::Evidence { command } => match command {
            crate::cli::TutorialsEvidenceCommand::Validate(args) => args.format = FormatArg::Json,
        },
        crate::cli::TutorialsCommand::RealData { command } => match command {
            crate::cli::TutorialsRealDataCommand::List(args)
            | crate::cli::TutorialsRealDataCommand::Doctor(args) => args.format = FormatArg::Json,
            crate::cli::TutorialsRealDataCommand::Plan(args) => {
                args.common.format = FormatArg::Json
            }
            crate::cli::TutorialsRealDataCommand::Fetch(args)
            | crate::cli::TutorialsRealDataCommand::Ingest(args)
            | crate::cli::TutorialsRealDataCommand::QueryPack(args)
            | crate::cli::TutorialsRealDataCommand::ExportEvidence(args)
            | crate::cli::TutorialsRealDataCommand::CompareRegression(args)
            | crate::cli::TutorialsRealDataCommand::VerifyIdempotency(args)
            | crate::cli::TutorialsRealDataCommand::CleanRun(args) => {
                args.common.format = FormatArg::Json
            }
            crate::cli::TutorialsRealDataCommand::RunAll(args) => {
                args.common.format = FormatArg::Json
            }
        },
    }
}

fn force_json_migrations(command: &mut MigrationsCommand) {
    match command {
        MigrationsCommand::Status { format, .. } => *format = FormatArg::Json,
    }
}

fn force_json_system(command: &mut crate::cli::SystemCommand) {
    match command {
        crate::cli::SystemCommand::Simulate { command } => match command {
            crate::cli::SystemSimulateCommand::Install(args)
            | crate::cli::SystemSimulateCommand::Upgrade(args)
            | crate::cli::SystemSimulateCommand::Rollback(args)
            | crate::cli::SystemSimulateCommand::OfflineMode(args)
            | crate::cli::SystemSimulateCommand::Suite(args) => args.format = FormatArg::Json,
        },
        crate::cli::SystemCommand::Debug { command } => match command {
            crate::cli::SystemDebugCommand::Diagnostics(args)
            | crate::cli::SystemDebugCommand::MetricsSnapshot(args)
            | crate::cli::SystemDebugCommand::HealthChecks(args)
            | crate::cli::SystemDebugCommand::RuntimeState(args)
            | crate::cli::SystemDebugCommand::TraceSampling(args) => args.format = FormatArg::Json,
        },
        crate::cli::SystemCommand::Cluster { command } => match command {
            crate::cli::SystemClusterCommand::Topology(args)
            | crate::cli::SystemClusterCommand::NodeList(args)
            | crate::cli::SystemClusterCommand::Status(args)
            | crate::cli::SystemClusterCommand::Diagnostics(args)
            | crate::cli::SystemClusterCommand::Membership(args)
            | crate::cli::SystemClusterCommand::NodeHealth(args)
            | crate::cli::SystemClusterCommand::ShardRouting(args)
            | crate::cli::SystemClusterCommand::ShardList(args)
            | crate::cli::SystemClusterCommand::ShardDistribution(args)
            | crate::cli::SystemClusterCommand::ShardDiagnostics(args)
            | crate::cli::SystemClusterCommand::ReplicaList(args)
            | crate::cli::SystemClusterCommand::ReplicaHealth(args)
            | crate::cli::SystemClusterCommand::ReplicaDiagnostics(args)
            | crate::cli::SystemClusterCommand::RecoveryRun(args)
            | crate::cli::SystemClusterCommand::ResilienceDiagnostics(args) => {
                args.format = FormatArg::Json
            }
            crate::cli::SystemClusterCommand::NodeDrain(args)
            | crate::cli::SystemClusterCommand::NodeMaintenance(args)
            | crate::cli::SystemClusterCommand::NodeDiagnostics(args) => {
                args.common.format = FormatArg::Json
            }
            crate::cli::SystemClusterCommand::ShardRebalance(args) => {
                args.common.format = FormatArg::Json
            }
            crate::cli::SystemClusterCommand::ReplicaFailover(args) => {
                args.common.format = FormatArg::Json
            }
            crate::cli::SystemClusterCommand::Failover(args)
            | crate::cli::SystemClusterCommand::ChaosTest(args) => {
                args.common.format = FormatArg::Json
            }
        },
    }
}

fn force_json_audit(command: &mut AuditCommand) {
    match command {
        AuditCommand::Run(args) | AuditCommand::Report(args) | AuditCommand::Explain(args) => {
            args.format = FormatArg::Json
        }
        AuditCommand::Bundle { command } => match command {
            crate::cli::AuditBundleCommand::Generate(args)
            | crate::cli::AuditBundleCommand::Validate(args) => args.format = FormatArg::Json,
        },
        AuditCommand::Compliance { command } => match command {
            crate::cli::AuditComplianceCommand::Report(args) => args.format = FormatArg::Json,
        },
        AuditCommand::Readiness { command } => match command {
            crate::cli::AuditReadinessCommand::Validate(args) => args.format = FormatArg::Json,
        },
    }
}

fn force_json_observe(command: &mut ObserveCommand) {
    match command {
        ObserveCommand::Metrics { command } => match command {
            crate::cli::ObserveMetricsCommand::List(args)
            | crate::cli::ObserveMetricsCommand::Docs(args) => args.format = FormatArg::Json,
            crate::cli::ObserveMetricsCommand::Explain(args) => {
                args.common.format = FormatArg::Json
            }
        },
        ObserveCommand::Dashboards { command } => match command {
            crate::cli::ObserveDashboardsCommand::List(args)
            | crate::cli::ObserveDashboardsCommand::Verify(args)
            | crate::cli::ObserveDashboardsCommand::Explain(args) => args.format = FormatArg::Json,
        },
        ObserveCommand::Logs { command } => match command {
            crate::cli::ObserveLogsCommand::Explain(args) => args.format = FormatArg::Json,
        },
        ObserveCommand::Traces { command } => match command {
            crate::cli::ObserveTracesCommand::Explain(args)
            | crate::cli::ObserveTracesCommand::Verify(args)
            | crate::cli::ObserveTracesCommand::Coverage(args)
            | crate::cli::ObserveTracesCommand::Topology(args) => args.format = FormatArg::Json,
        },
    }
}

fn force_json_api(command: &mut ApiCommand) {
    match command {
        ApiCommand::List(args)
        | ApiCommand::Verify(args)
        | ApiCommand::Validate(args)
        | ApiCommand::Contract(args) => args.format = FormatArg::Json,
        ApiCommand::Explain(args) => args.common.format = FormatArg::Json,
        ApiCommand::Diff(args) => args.common.format = FormatArg::Json,
    }
}

fn force_json_load(command: &mut LoadCommand) {
    match command {
        LoadCommand::Run(args) | LoadCommand::Baseline(args) | LoadCommand::Explain(args) => {
            args.format = FormatArg::Json
        }
        LoadCommand::Compare(args) => args.common.format = FormatArg::Json,
    }
}

fn force_json_perf(command: &mut PerfCommand) {
    match command {
        PerfCommand::Validate(args) => args.format = FormatArg::Json,
        PerfCommand::Run(args) => args.format = FormatArg::Json,
        PerfCommand::Diff(args) => args.format = FormatArg::Json,
        PerfCommand::ColdStart(args) => args.format = FormatArg::Json,
        PerfCommand::Kind(args) => args.format = FormatArg::Json,
        PerfCommand::Benches { command } => match command {
            crate::cli::PerfBenchesCommand::List(args) => args.format = FormatArg::Json,
        },
        PerfCommand::CliUx { command } => match command {
            crate::cli::PerfCliUxCommand::Bench(args) => args.format = FormatArg::Json,
            crate::cli::PerfCliUxCommand::Diff(args) => args.format = FormatArg::Json,
        },
    }
}

fn force_json_datasets(command: &mut DatasetsCommand) {
    match command {
        DatasetsCommand::Validate(args) => args.format = FormatArg::Json,
    }
}

fn force_json_ingest(command: &mut IngestCommand) {
    match command {
        IngestCommand::DryRun(args) => args.format = FormatArg::Json,
        IngestCommand::Run(args) => args.format = FormatArg::Json,
    }
}

pub(super) fn apply_fail_fast(command: &mut Command) {
    match command {
        Command::Tests {
            command: TestsCommand::Run { fail_fast, .. },
        } => *fail_fast = true,
        Command::Tests { .. } => {}
        Command::Suites {
            command:
                crate::cli::SuitesCommand::Run {
                    fail_fast,
                    no_fail_fast,
                    ..
                },
        } => {
            *fail_fast = true;
            *no_fail_fast = false;
        }
        Command::Docs { command } => match command {
            DocsCommand::Check(common)
            | DocsCommand::Doctor(common)
            | DocsCommand::DeployPlan(common)
            | DocsCommand::SiteDir(common)
            | DocsCommand::Validate(common)
            | DocsCommand::Lint(common)
            | DocsCommand::IncludesCheck(common)
            | DocsCommand::Links(common)
            | DocsCommand::NavIntegrity(common)
            | DocsCommand::ExternalLinks(crate::cli::DocsExternalLinksArgs { common, .. })
            | DocsCommand::Graph(common)
            | DocsCommand::Dead(common)
            | DocsCommand::Duplicates(common)
            | DocsCommand::PrunePlan(common)
            | DocsCommand::DedupeReport(common)
            | DocsCommand::VerifyContracts(common)
            | DocsCommand::UxSmoke(common) => common.strict = true,
            DocsCommand::Build(_)
            | DocsCommand::Serve(_)
            | DocsCommand::PagesSmoke(_)
            | DocsCommand::Clean(_)
            | DocsCommand::Inventory(_)
            | DocsCommand::Top(_)
            | DocsCommand::ShrinkReport(_)
            | DocsCommand::Grep(_)
            | DocsCommand::HealthDashboard(_)
            | DocsCommand::LifecycleSummaryTable(_)
            | DocsCommand::DrillSummaryTable(_)
            | DocsCommand::Where(_)
            | DocsCommand::VerifyGenerated(_) => {}
            DocsCommand::Redirects { command } => match command {
                crate::cli::DocsRedirectsCommand::Sync(_) => {}
            },
            DocsCommand::Merge { command } => match command {
                crate::cli::DocsMergeCommand::Validate(_) => {}
            },
            DocsCommand::Spine { command } => match command {
                crate::cli::DocsSpineCommand::Validate(_)
                | crate::cli::DocsSpineCommand::Report(_) => {}
            },
            DocsCommand::Toc { command } => match command {
                crate::cli::DocsTocCommand::Verify(_) => {}
            },
            DocsCommand::Reference { command } => match command {
                crate::cli::DocsReferenceCommand::Generate(_)
                | crate::cli::DocsReferenceCommand::Check(_) => {}
            },
            DocsCommand::Generate { command } => match command {
                crate::cli::DocsGenerateCommand::Examples(_)
                | crate::cli::DocsGenerateCommand::CommandLists(_)
                | crate::cli::DocsGenerateCommand::SchemaSnippets(_)
                | crate::cli::DocsGenerateCommand::OpenapiSnippets(_)
                | crate::cli::DocsGenerateCommand::OpsSnippets(_)
                | crate::cli::DocsGenerateCommand::RealDataPages(_) => {}
            },
        },
        Command::Configs { command } => match command {
            ConfigsCommand::Doctor(common)
            | ConfigsCommand::Verify(common)
            | ConfigsCommand::Validate(common)
            | ConfigsCommand::Lint(common)
            | ConfigsCommand::Inventory(common)
            | ConfigsCommand::Diff(common)
            | ConfigsCommand::Graph(common) => common.strict = true,
            ConfigsCommand::Fmt { check, .. } => *check = true,
            ConfigsCommand::Print(_)
            | ConfigsCommand::List(_)
            | ConfigsCommand::Explain(_)
            | ConfigsCommand::Compile(_) => {}
        },
        Command::Make { .. } => {}
        _ => {}
    }
}

pub(super) fn propagate_repo_root(command: &mut Command, repo_root: Option<std::path::PathBuf>) {
    let Some(root) = repo_root else {
        return;
    };
    match command {
        Command::Make { command } => match command {
            MakeCommand::VerifyModule(args) => args.common.repo_root = Some(root.clone()),
            MakeCommand::Wrappers { command } => match command {
                crate::cli::MakeWrappersCommand::Verify(common) => {
                    common.repo_root = Some(root.clone())
                }
            },
            MakeCommand::Surface(common)
            | MakeCommand::List(common)
            | MakeCommand::Explain(crate::cli::MakeExplainArgs { common, .. })
            | MakeCommand::TargetList(common)
            | MakeCommand::LintPolicyReport(common) => common.repo_root = Some(root.clone()),
        },
        Command::Artifacts { command } => match command {
            ArtifactsCommand::Clean(common) => common.repo_root = Some(root.clone()),
            ArtifactsCommand::Gc(args) => args.common.repo_root = Some(root.clone()),
            ArtifactsCommand::Report { command } => match command {
                crate::cli::ArtifactsReportCommand::Inventory(common) => {
                    common.repo_root = Some(root.clone())
                }
                crate::cli::ArtifactsReportCommand::Manifest(args)
                | crate::cli::ArtifactsReportCommand::Index(args)
                | crate::cli::ArtifactsReportCommand::Validate(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::ArtifactsReportCommand::Read(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::ArtifactsReportCommand::Diff(args) => {
                    args.common.repo_root = Some(root.clone())
                }
            },
        },
        Command::Reports { command } => match command {
            ReportsCommand::List(args) => args.repo_root = Some(root.clone()),
            ReportsCommand::Index(args) => args.repo_root = Some(root.clone()),
            ReportsCommand::Progress(args) => args.repo_root = Some(root.clone()),
            ReportsCommand::Validate(args) => args.repo_root = Some(root.clone()),
        },
        Command::Ops { command } => match command {
            OpsCommand::Kind { command } => match command {
                crate::cli::OpsKindCommand::Up(common)
                | crate::cli::OpsKindCommand::Down(common)
                | crate::cli::OpsKindCommand::Status(common) => {
                    common.repo_root = Some(root.clone())
                }
                crate::cli::OpsKindCommand::PreloadImage(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::OpsKindCommand::Install(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::OpsKindCommand::Upgrade(args) => {
                    args.release.common.repo_root = Some(root.clone())
                }
                crate::cli::OpsKindCommand::Rollback(args) => {
                    args.release.common.repo_root = Some(root.clone())
                }
                crate::cli::OpsKindCommand::Smoke(args) => {
                    args.common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::Logs { command }
            | OpsCommand::Describe { command }
            | OpsCommand::Events { command } => match command {
                crate::cli::OpsCollectCommand::Collect(args) => {
                    args.common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::Resources { command } => match command {
                crate::cli::OpsResourcesCommand::Snapshot(args) => {
                    args.common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::Helm { command } => match command {
                crate::cli::OpsHelmCommand::Install(args) => {
                    args.release.common.repo_root = Some(root.clone())
                }
                crate::cli::OpsHelmCommand::Uninstall(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::OpsHelmCommand::Upgrade(args) => {
                    args.release.common.repo_root = Some(root.clone())
                }
                crate::cli::OpsHelmCommand::Rollback(args) => {
                    args.release.common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::List(common)
            | OpsCommand::Doctor(common)
            | OpsCommand::Validate(common)
            | OpsCommand::Graph(common)
            | OpsCommand::Inventory(common)
            | OpsCommand::Docs(common)
            | OpsCommand::DocsVerify(common)
            | OpsCommand::Conformance(common)
            | OpsCommand::Report(common)
            | OpsCommand::Readiness(common)
            | OpsCommand::ListProfiles(common)
            | OpsCommand::ListTools(common)
            | OpsCommand::VerifyTools(common)
            | OpsCommand::ListActions(common)
            | OpsCommand::Plan(common)
            | OpsCommand::ReleasePlan(common)
            | OpsCommand::InstallPlan(common)
            | OpsCommand::Up(common)
            | OpsCommand::Down(common)
            | OpsCommand::Clean(common)
            | OpsCommand::Cleanup(common)
            | OpsCommand::K8sPlan(common)
            | OpsCommand::K8sEnvSurface(common)
            | OpsCommand::K8sValidateProfiles(common)
            | OpsCommand::Profiles {
                command:
                    crate::cli::OpsProfilesCommand::Validate(crate::cli::OpsProfilesValidateArgs {
                        common,
                        ..
                    }),
            }
            | OpsCommand::Profiles {
                command:
                    crate::cli::OpsProfilesCommand::SchemaValidate(
                        crate::cli::OpsProfileValidationArgs { common, .. },
                    )
                    | crate::cli::OpsProfilesCommand::Kubeconform(crate::cli::OpsProfileValidationArgs {
                        common,
                        ..
                    })
                    | crate::cli::OpsProfilesCommand::RolloutSafetyValidate(
                        crate::cli::OpsProfileValidationArgs { common, .. },
                    )
                    | crate::cli::OpsProfilesCommand::PolicyValidate(
                        crate::cli::OpsProfileValidationArgs { common, .. },
                    )
                    | crate::cli::OpsProfilesCommand::ResourceValidate(
                        crate::cli::OpsProfileValidationArgs { common, .. },
                    )
                    | crate::cli::OpsProfilesCommand::SecuritycontextValidate(
                        crate::cli::OpsProfileValidationArgs { common, .. },
                    )
                    | crate::cli::OpsProfilesCommand::ServiceMonitorValidate(
                        crate::cli::OpsProfileValidationArgs { common, .. },
                    )
                    | crate::cli::OpsProfilesCommand::HpaValidate(crate::cli::OpsProfileValidationArgs {
                        common,
                        ..
                    }),
            }
            | OpsCommand::Profile {
                command: crate::cli::OpsProfileCommand::List(common),
            }
            | OpsCommand::K8sDryRun(common)
            | OpsCommand::K8sPorts(common)
            | OpsCommand::K8sConformance(common) => common.repo_root = Some(root.clone()),
            OpsCommand::LoadPlan { common, .. }
            | OpsCommand::LoadRun { common, .. }
            | OpsCommand::LoadReport { common, .. } => common.repo_root = Some(root.clone()),
            OpsCommand::Explain { common, .. }
            | OpsCommand::Profile {
                command: crate::cli::OpsProfileCommand::Explain { common, .. },
            } => common.repo_root = Some(root.clone()),
            OpsCommand::Render(args) => args.common.repo_root = Some(root.clone()),
            OpsCommand::Package(args) => args.common.repo_root = Some(root.clone()),
            OpsCommand::Install(args) => args.common.repo_root = Some(root.clone()),
            OpsCommand::Smoke(args) => args.common.repo_root = Some(root.clone()),
            OpsCommand::Status(args) => args.common.repo_root = Some(root.clone()),
            OpsCommand::HelmEnv(args) => args.common.repo_root = Some(root.clone()),
            OpsCommand::ExplainProfile { common, .. } => common.repo_root = Some(root.clone()),
            OpsCommand::Reset(args) => args.common.repo_root = Some(root.clone()),
            OpsCommand::K8sApply(args) => args.common.repo_root = Some(root.clone()),
            OpsCommand::K8sWait(args) => args.common.repo_root = Some(root.clone()),
            OpsCommand::K8sLogs(args) => args.common.repo_root = Some(root.clone()),
            OpsCommand::K8sPortForward(args) => args.common.repo_root = Some(root.clone()),
            OpsCommand::Pins { command } => match command {
                crate::cli::OpsPinsCommand::Check(common)
                | crate::cli::OpsPinsCommand::Update { common, .. } => {
                    common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::Generate { command } => match command {
                crate::cli::OpsGenerateCommand::PinsIndex { common, .. }
                | crate::cli::OpsGenerateCommand::SurfaceList { common, .. }
                | crate::cli::OpsGenerateCommand::Runbook { common, .. }
                | crate::cli::OpsGenerateCommand::ChartDependencySbom { common, .. }
                | crate::cli::OpsGenerateCommand::ResilienceReport { common, .. } => {
                    common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::Evidence { command } => match command {
                crate::cli::OpsEvidenceCommand::Collect(common) => {
                    common.repo_root = Some(root.clone())
                }
                crate::cli::OpsEvidenceCommand::Summarize(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::OpsEvidenceCommand::Verify(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::OpsEvidenceCommand::Diff(args) => {
                    args.common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::Diagnose { command } => match command {
                crate::cli::OpsDiagnoseCommand::Bundle(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::OpsDiagnoseCommand::Explain(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::OpsDiagnoseCommand::Redact(args) => {
                    args.common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::Drills { command } => match command {
                crate::cli::OpsDrillsCommand::Run(args) => {
                    args.common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::Datasets { command } => match command {
                crate::cli::OpsDatasetsCommand::List(common)
                | crate::cli::OpsDatasetsCommand::Ingest(common)
                | crate::cli::OpsDatasetsCommand::Publish(common)
                | crate::cli::OpsDatasetsCommand::Promote(common)
                | crate::cli::OpsDatasetsCommand::Rollback(common)
                | crate::cli::OpsDatasetsCommand::Qc(common) => {
                    common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::Schema { command } => match command {
                crate::cli::OpsSchemaCommand::Validate(common)
                | crate::cli::OpsSchemaCommand::Diff(common)
                | crate::cli::OpsSchemaCommand::Coverage(common)
                | crate::cli::OpsSchemaCommand::RegenIndex(common) => {
                    common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::InventoryDomain { command } => match command {
                crate::cli::OpsInventoryCommand::Validate(common)
                | crate::cli::OpsInventoryCommand::Graph(common)
                | crate::cli::OpsInventoryCommand::Diff(common)
                | crate::cli::OpsInventoryCommand::Coverage(common)
                | crate::cli::OpsInventoryCommand::OrphanCheck(common) => {
                    common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::ReportDomain { command } => match command {
                crate::cli::OpsReportCommand::Generate(common)
                | crate::cli::OpsReportCommand::Diff(common)
                | crate::cli::OpsReportCommand::Readiness(common)
                | crate::cli::OpsReportCommand::Bundle(common) => {
                    common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::Tools { command } => match command {
                crate::cli::OpsToolsCommand::List(common)
                | crate::cli::OpsToolsCommand::Verify(common)
                | crate::cli::OpsToolsCommand::Doctor(common) => {
                    common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::Suite { command } => match command {
                crate::cli::OpsSuiteCommand::List(common)
                | crate::cli::OpsSuiteCommand::Run { common, .. } => {
                    common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::Stack { command } => match command {
                crate::cli::OpsStackCommand::Plan(common)
                | crate::cli::OpsStackCommand::Up(common)
                | crate::cli::OpsStackCommand::Down(common)
                | crate::cli::OpsStackCommand::Logs(common)
                | crate::cli::OpsStackCommand::Ports(common)
                | crate::cli::OpsStackCommand::Versions(common)
                | crate::cli::OpsStackCommand::Doctor(common) => {
                    common.repo_root = Some(root.clone())
                }
                crate::cli::OpsStackCommand::Status(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::OpsStackCommand::Reset(args) => {
                    args.common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::K8s { command } => match command {
                crate::cli::OpsK8sCommand::Render(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::OpsK8sCommand::Validate(common)
                | crate::cli::OpsK8sCommand::EnvSurface(common)
                | crate::cli::OpsK8sCommand::ValidateProfiles(common)
                | crate::cli::OpsK8sCommand::Plan(common)
                | crate::cli::OpsK8sCommand::Uninstall(common)
                | crate::cli::OpsK8sCommand::Ports(common)
                | crate::cli::OpsK8sCommand::Diff(common)
                | crate::cli::OpsK8sCommand::Rollout(common)
                | crate::cli::OpsK8sCommand::DryRun(common)
                | crate::cli::OpsK8sCommand::Conformance(common)
                | crate::cli::OpsK8sCommand::Test(common) => common.repo_root = Some(root.clone()),
                crate::cli::OpsK8sCommand::Smoke(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::OpsK8sCommand::Install(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::OpsK8sCommand::Apply(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::OpsK8sCommand::Wait(args) => args.common.repo_root = Some(root.clone()),
                crate::cli::OpsK8sCommand::Logs(args) => args.common.repo_root = Some(root.clone()),
                crate::cli::OpsK8sCommand::PortForward(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::OpsK8sCommand::Status(args) => {
                    args.common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::Load { command } => match command {
                crate::cli::OpsLoadCommand::Plan { common, .. }
                | crate::cli::OpsLoadCommand::Run { common, .. }
                | crate::cli::OpsLoadCommand::Report { common, .. }
                | crate::cli::OpsLoadCommand::Evaluate(common)
                | crate::cli::OpsLoadCommand::ListSuites(common) => {
                    common.repo_root = Some(root.clone())
                }
                crate::cli::OpsLoadCommand::Baseline { command } => match command {
                    crate::cli::OpsLoadBaselineCommand::Update(common) => {
                        common.repo_root = Some(root.clone())
                    }
                },
            },
            OpsCommand::E2e { command } => match command {
                crate::cli::OpsE2eCommand::Run(common)
                | crate::cli::OpsE2eCommand::Smoke(common)
                | crate::cli::OpsE2eCommand::Realdata(common)
                | crate::cli::OpsE2eCommand::ListSuites(common) => {
                    common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::Scenario { command } => match command {
                crate::cli::OpsScenarioCommand::Run(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::OpsScenarioCommand::List(common) => {
                    common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::Obs { command } => match command {
                crate::cli::OpsObsCommand::Up(common)
                | crate::cli::OpsObsCommand::Down(common)
                | crate::cli::OpsObsCommand::Validate(common)
                | crate::cli::OpsObsCommand::Snapshot(common)
                | crate::cli::OpsObsCommand::Dashboards(common)
                | crate::cli::OpsObsCommand::Verify(common) => {
                    common.repo_root = Some(root.clone())
                }
                crate::cli::OpsObsCommand::Slo { command } => match command {
                    crate::cli::OpsObsSloCommand::List(common)
                    | crate::cli::OpsObsSloCommand::Verify(common) => {
                        common.repo_root = Some(root.clone())
                    }
                },
                crate::cli::OpsObsCommand::Alerts { command } => match command {
                    crate::cli::OpsObsAlertsCommand::Verify(common) => {
                        common.repo_root = Some(root.clone())
                    }
                },
                crate::cli::OpsObsCommand::Runbooks { command } => match command {
                    crate::cli::OpsObsRunbooksCommand::Verify(common) => {
                        common.repo_root = Some(root.clone())
                    }
                },
                crate::cli::OpsObsCommand::Drill { command } => match command {
                    crate::cli::OpsObsDrillCommand::Run(common) => {
                        common.repo_root = Some(root.clone())
                    }
                },
            },
        },
        Command::Docs { command } => match command {
            DocsCommand::Check(common)
            | DocsCommand::VerifyContracts(common)
            | DocsCommand::Doctor(common)
            | DocsCommand::DeployPlan(common)
            | DocsCommand::UxSmoke(common)
            | DocsCommand::Where(common)
            | DocsCommand::SiteDir(common)
            | DocsCommand::Validate(common)
            | DocsCommand::Build(common)
            | DocsCommand::Clean(common)
            | DocsCommand::Lint(common)
            | DocsCommand::IncludesCheck(common)
            | DocsCommand::Links(common)
            | DocsCommand::NavIntegrity(common)
            | DocsCommand::ExternalLinks(crate::cli::DocsExternalLinksArgs { common, .. })
            | DocsCommand::Inventory(common)
            | DocsCommand::Graph(common)
            | DocsCommand::Dead(common)
            | DocsCommand::Duplicates(common)
            | DocsCommand::PrunePlan(common)
            | DocsCommand::DedupeReport(common)
            | DocsCommand::ShrinkReport(common)
            | DocsCommand::HealthDashboard(common)
            | DocsCommand::VerifyGenerated(common) => common.repo_root = Some(root.clone()),
            DocsCommand::Top(args) => args.common.repo_root = Some(root.clone()),
            DocsCommand::PagesSmoke(args) => args.common.repo_root = Some(root.clone()),
            DocsCommand::Serve(args) => args.common.repo_root = Some(root.clone()),
            DocsCommand::Grep(args) => args.common.repo_root = Some(root.clone()),
            DocsCommand::LifecycleSummaryTable(args) | DocsCommand::DrillSummaryTable(args) => {
                args.common.repo_root = Some(root.clone())
            }
            DocsCommand::Redirects { command } => match command {
                crate::cli::DocsRedirectsCommand::Sync(common) => {
                    common.repo_root = Some(root.clone())
                }
            },
            DocsCommand::Merge { command } => match command {
                crate::cli::DocsMergeCommand::Validate(common) => {
                    common.repo_root = Some(root.clone())
                }
            },
            DocsCommand::Spine { command } => match command {
                crate::cli::DocsSpineCommand::Validate(common)
                | crate::cli::DocsSpineCommand::Report(common) => {
                    common.repo_root = Some(root.clone())
                }
            },
            DocsCommand::Toc { command } => match command {
                crate::cli::DocsTocCommand::Verify(common) => common.repo_root = Some(root.clone()),
            },
            DocsCommand::Reference { command } => match command {
                crate::cli::DocsReferenceCommand::Generate(common)
                | crate::cli::DocsReferenceCommand::Check(common) => {
                    common.repo_root = Some(root.clone())
                }
            },
            DocsCommand::Generate { command } => match command {
                crate::cli::DocsGenerateCommand::Examples(common)
                | crate::cli::DocsGenerateCommand::CommandLists(common)
                | crate::cli::DocsGenerateCommand::SchemaSnippets(common)
                | crate::cli::DocsGenerateCommand::OpenapiSnippets(common)
                | crate::cli::DocsGenerateCommand::OpsSnippets(common)
                | crate::cli::DocsGenerateCommand::RealDataPages(common) => {
                    common.repo_root = Some(root.clone())
                }
            },
        },
        Command::Configs { command } => match command {
            ConfigsCommand::Print(common)
            | ConfigsCommand::List(common)
            | ConfigsCommand::Graph(common)
            | ConfigsCommand::Explain(crate::cli::ConfigsExplainArgs { common, .. })
            | ConfigsCommand::Verify(common)
            | ConfigsCommand::Doctor(common)
            | ConfigsCommand::Validate(common)
            | ConfigsCommand::Lint(common)
            | ConfigsCommand::Inventory(common)
            | ConfigsCommand::Compile(common)
            | ConfigsCommand::Diff(common) => common.repo_root = Some(root.clone()),
            ConfigsCommand::Fmt { common, .. } => common.repo_root = Some(root.clone()),
        },
        Command::Governance { command } => match command {
            crate::cli::GovernanceCommand::Version { repo_root, .. }
            | crate::cli::GovernanceCommand::List { repo_root, .. }
            | crate::cli::GovernanceCommand::Explain { repo_root, .. }
            | crate::cli::GovernanceCommand::Validate { repo_root, .. }
            | crate::cli::GovernanceCommand::Check { repo_root, .. }
            | crate::cli::GovernanceCommand::Rules { repo_root, .. }
            | crate::cli::GovernanceCommand::Report { repo_root, .. }
            | crate::cli::GovernanceCommand::DoctrineReport { repo_root, .. }
            | crate::cli::GovernanceCommand::Doctor { repo_root, .. } => {
                *repo_root = Some(root.clone())
            }
            crate::cli::GovernanceCommand::Exceptions { command } => match command {
                crate::cli::GovernanceExceptionsCommand::List { repo_root, .. }
                | crate::cli::GovernanceExceptionsCommand::Validate { repo_root, .. } => {
                    *repo_root = Some(root.clone())
                }
            },
            crate::cli::GovernanceCommand::Deprecations { command } => match command {
                crate::cli::GovernanceDeprecationsCommand::Validate { repo_root, .. } => {
                    *repo_root = Some(root.clone())
                }
            },
            crate::cli::GovernanceCommand::Breaking { command } => match command {
                crate::cli::GovernanceBreakingCommand::Validate { repo_root, .. } => {
                    *repo_root = Some(root.clone())
                }
            },
            crate::cli::GovernanceCommand::Adr { command } => match command {
                crate::cli::GovernanceAdrCommand::Index { repo_root, .. } => {
                    *repo_root = Some(root.clone())
                }
            },
        },
        Command::Security { command } => match command {
            SecurityCommand::Validate(args)
            | SecurityCommand::ConfigValidate(args)
            | SecurityCommand::Diagnostics(args)
            | SecurityCommand::Audit(args)
            | SecurityCommand::VulnerabilityReport(args)
            | SecurityCommand::DependencyAudit(args) => args.repo_root = Some(root.clone()),
            SecurityCommand::PolicyInspect(args) => args.repo_root = Some(root.clone()),
            SecurityCommand::IncidentReport(args) => args.repo_root = Some(root.clone()),
            SecurityCommand::Authentication { command } => match command {
                crate::cli::SecurityAuthenticationCommand::ApiKeys(args)
                | crate::cli::SecurityAuthenticationCommand::Diagnostics(args)
                | crate::cli::SecurityAuthenticationCommand::PolicyValidate(args) => {
                    args.repo_root = Some(root.clone())
                }
                crate::cli::SecurityAuthenticationCommand::TokenInspect(args) => {
                    args.repo_root = Some(root.clone())
                }
            },
            SecurityCommand::Authorization { command } => match command {
                crate::cli::SecurityAuthorizationCommand::Roles(args)
                | crate::cli::SecurityAuthorizationCommand::Permissions(args)
                | crate::cli::SecurityAuthorizationCommand::Diagnostics(args)
                | crate::cli::SecurityAuthorizationCommand::Validate(args) => {
                    args.repo_root = Some(root.clone())
                }
                crate::cli::SecurityAuthorizationCommand::Assign(args) => {
                    args.repo_root = Some(root.clone())
                }
            },
            SecurityCommand::Compliance { command } => match command {
                crate::cli::SecurityComplianceCommand::Validate(args) => {
                    args.repo_root = Some(root.clone())
                }
            },
            SecurityCommand::Threats { command } => match command {
                crate::cli::SecurityThreatCommand::List(args)
                | crate::cli::SecurityThreatCommand::Verify(args) => {
                    args.repo_root = Some(root.clone())
                }
                crate::cli::SecurityThreatCommand::Explain(args) => {
                    args.repo_root = Some(root.clone())
                }
            },
            SecurityCommand::ScanArtifacts(args) => args.repo_root = Some(root.clone()),
        },
        Command::Runtime { command } => match command {
            crate::cli::RuntimeCommand::SelfCheck(args)
            | crate::cli::RuntimeCommand::PrintConfigSchema(args)
            | crate::cli::RuntimeCommand::ExplainConfigSchema(args) => {
                args.repo_root = Some(root.clone())
            }
        },
        Command::Tutorials { command } => match command {
            crate::cli::TutorialsCommand::List(args)
            | crate::cli::TutorialsCommand::Explain(args)
            | crate::cli::TutorialsCommand::Verify(args)
            | crate::cli::TutorialsCommand::ReproducibilityCheck(args)
            | crate::cli::TutorialsCommand::Generate(args) => args.repo_root = Some(root.clone()),
            crate::cli::TutorialsCommand::Run { command } => match command {
                crate::cli::TutorialsRunCommand::Workflow(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::TutorialsRunCommand::DatasetE2e(args) => {
                    args.common.repo_root = Some(root.clone())
                }
            },
            crate::cli::TutorialsCommand::Build { command } => match command {
                crate::cli::TutorialsBuildCommand::Docs(args) => {
                    args.common.repo_root = Some(root.clone())
                }
            },
            crate::cli::TutorialsCommand::Dataset { command } => match command {
                crate::cli::TutorialsDatasetCommand::Package(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::TutorialsDatasetCommand::Ingest(args)
                | crate::cli::TutorialsDatasetCommand::IntegrityCheck(args) => {
                    args.repo_root = Some(root.clone())
                }
            },
            crate::cli::TutorialsCommand::Workspace { command } => match command {
                crate::cli::TutorialsWorkspaceCommand::Cleanup(args) => {
                    args.common.repo_root = Some(root.clone())
                }
            },
            crate::cli::TutorialsCommand::Dashboards { command } => match command {
                crate::cli::TutorialsDashboardsCommand::Validate(args) => {
                    args.repo_root = Some(root.clone())
                }
            },
            crate::cli::TutorialsCommand::Evidence { command } => match command {
                crate::cli::TutorialsEvidenceCommand::Validate(args) => {
                    args.repo_root = Some(root.clone())
                }
            },
            crate::cli::TutorialsCommand::RealData { command } => match command {
                crate::cli::TutorialsRealDataCommand::List(args)
                | crate::cli::TutorialsRealDataCommand::Doctor(args) => {
                    args.repo_root = Some(root.clone())
                }
                crate::cli::TutorialsRealDataCommand::Plan(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::TutorialsRealDataCommand::Fetch(args)
                | crate::cli::TutorialsRealDataCommand::Ingest(args)
                | crate::cli::TutorialsRealDataCommand::QueryPack(args)
                | crate::cli::TutorialsRealDataCommand::ExportEvidence(args)
                | crate::cli::TutorialsRealDataCommand::CompareRegression(args)
                | crate::cli::TutorialsRealDataCommand::VerifyIdempotency(args)
                | crate::cli::TutorialsRealDataCommand::CleanRun(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::TutorialsRealDataCommand::RunAll(args) => {
                    args.common.repo_root = Some(root.clone())
                }
            },
        },
        Command::Migrations { command } => match command {
            MigrationsCommand::Status { repo_root, .. } => *repo_root = Some(root.clone()),
        },
        Command::System { command } => match command {
            crate::cli::SystemCommand::Simulate { command } => match command {
                crate::cli::SystemSimulateCommand::Install(args)
                | crate::cli::SystemSimulateCommand::Upgrade(args)
                | crate::cli::SystemSimulateCommand::Rollback(args)
                | crate::cli::SystemSimulateCommand::OfflineMode(args)
                | crate::cli::SystemSimulateCommand::Suite(args) => {
                    args.repo_root = Some(root.clone())
                }
            },
            crate::cli::SystemCommand::Debug { command } => match command {
                crate::cli::SystemDebugCommand::Diagnostics(args)
                | crate::cli::SystemDebugCommand::MetricsSnapshot(args)
                | crate::cli::SystemDebugCommand::HealthChecks(args)
                | crate::cli::SystemDebugCommand::RuntimeState(args)
                | crate::cli::SystemDebugCommand::TraceSampling(args) => {
                    args.repo_root = Some(root.clone())
                }
            },
            crate::cli::SystemCommand::Cluster { command } => match command {
                crate::cli::SystemClusterCommand::Topology(args)
                | crate::cli::SystemClusterCommand::NodeList(args)
                | crate::cli::SystemClusterCommand::Status(args)
                | crate::cli::SystemClusterCommand::Diagnostics(args)
                | crate::cli::SystemClusterCommand::Membership(args)
                | crate::cli::SystemClusterCommand::NodeHealth(args)
                | crate::cli::SystemClusterCommand::ShardRouting(args)
                | crate::cli::SystemClusterCommand::ShardList(args)
                | crate::cli::SystemClusterCommand::ShardDistribution(args)
                | crate::cli::SystemClusterCommand::ShardDiagnostics(args)
                | crate::cli::SystemClusterCommand::ReplicaList(args)
                | crate::cli::SystemClusterCommand::ReplicaHealth(args)
                | crate::cli::SystemClusterCommand::ReplicaDiagnostics(args)
                | crate::cli::SystemClusterCommand::RecoveryRun(args)
                | crate::cli::SystemClusterCommand::ResilienceDiagnostics(args) => {
                    args.repo_root = Some(root.clone())
                }
                crate::cli::SystemClusterCommand::NodeDrain(args)
                | crate::cli::SystemClusterCommand::NodeMaintenance(args)
                | crate::cli::SystemClusterCommand::NodeDiagnostics(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::SystemClusterCommand::ShardRebalance(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::SystemClusterCommand::ReplicaFailover(args) => {
                    args.common.repo_root = Some(root.clone())
                }
                crate::cli::SystemClusterCommand::Failover(args)
                | crate::cli::SystemClusterCommand::ChaosTest(args) => {
                    args.common.repo_root = Some(root.clone())
                }
            },
        },
        Command::Audit { command } => match command {
            AuditCommand::Run(args) | AuditCommand::Report(args) | AuditCommand::Explain(args) => {
                args.repo_root = Some(root.clone())
            }
            AuditCommand::Bundle { command } => match command {
                crate::cli::AuditBundleCommand::Generate(args)
                | crate::cli::AuditBundleCommand::Validate(args) => {
                    args.repo_root = Some(root.clone())
                }
            },
            AuditCommand::Compliance { command } => match command {
                crate::cli::AuditComplianceCommand::Report(args) => {
                    args.repo_root = Some(root.clone())
                }
            },
            AuditCommand::Readiness { command } => match command {
                crate::cli::AuditReadinessCommand::Validate(args) => {
                    args.repo_root = Some(root.clone())
                }
            },
        },
        Command::Observe { command } => match command {
            ObserveCommand::Metrics { command } => match command {
                crate::cli::ObserveMetricsCommand::List(args)
                | crate::cli::ObserveMetricsCommand::Docs(args) => {
                    args.repo_root = Some(root.clone())
                }
                crate::cli::ObserveMetricsCommand::Explain(args) => {
                    args.common.repo_root = Some(root.clone())
                }
            },
            ObserveCommand::Dashboards { command } => match command {
                crate::cli::ObserveDashboardsCommand::List(args)
                | crate::cli::ObserveDashboardsCommand::Verify(args)
                | crate::cli::ObserveDashboardsCommand::Explain(args) => {
                    args.repo_root = Some(root.clone())
                }
            },
            ObserveCommand::Logs { command } => match command {
                crate::cli::ObserveLogsCommand::Explain(args) => {
                    args.repo_root = Some(root.clone())
                }
            },
            ObserveCommand::Traces { command } => match command {
                crate::cli::ObserveTracesCommand::Explain(args)
                | crate::cli::ObserveTracesCommand::Verify(args)
                | crate::cli::ObserveTracesCommand::Coverage(args)
                | crate::cli::ObserveTracesCommand::Topology(args) => {
                    args.repo_root = Some(root.clone())
                }
            },
        },
        Command::Api { command } => match command {
            ApiCommand::List(args)
            | ApiCommand::Verify(args)
            | ApiCommand::Validate(args)
            | ApiCommand::Contract(args) => args.repo_root = Some(root.clone()),
            ApiCommand::Explain(args) => args.common.repo_root = Some(root.clone()),
            ApiCommand::Diff(args) => args.common.repo_root = Some(root.clone()),
        },
        Command::Load { command } => match command {
            LoadCommand::Run(args) | LoadCommand::Baseline(args) | LoadCommand::Explain(args) => {
                args.repo_root = Some(root.clone())
            }
            LoadCommand::Compare(args) => args.common.repo_root = Some(root.clone()),
        },
        Command::Invariants { command } => match command {
            InvariantsCommand::Run(args)
            | InvariantsCommand::List(args)
            | InvariantsCommand::Coverage(args)
            | InvariantsCommand::Docs(args) => args.repo_root = Some(root.clone()),
            InvariantsCommand::Explain(args) => args.common.repo_root = Some(root.clone()),
        },
        Command::Drift { command } => match command {
            DriftCommand::Detect(args) | DriftCommand::Report(args) => {
                args.common.repo_root = Some(root.clone())
            }
            DriftCommand::Coverage(args) => args.common.repo_root = Some(root.clone()),
            DriftCommand::Explain(args) => args.common.repo_root = Some(root.clone()),
            DriftCommand::Baseline(args) => args.detect.common.repo_root = Some(root.clone()),
            DriftCommand::Compare(args) => args.detect.common.repo_root = Some(root.clone()),
        },
        Command::Reproduce { command } => match command {
            ReproduceCommand::Run(args)
            | ReproduceCommand::Verify(args)
            | ReproduceCommand::Status(args)
            | ReproduceCommand::AuditReport(args)
            | ReproduceCommand::Metrics(args)
            | ReproduceCommand::LineageValidate(args)
            | ReproduceCommand::SummaryTable(args) => args.repo_root = Some(root.clone()),
            ReproduceCommand::Explain(args) => args.common.repo_root = Some(root.clone()),
        },
        Command::Datasets { command } => match command {
            DatasetsCommand::Validate(args) => args.repo_root = Some(root.clone()),
        },
        Command::Ingest { command } => match command {
            IngestCommand::DryRun(args) => args.repo_root = Some(root.clone()),
            IngestCommand::Run(args) => args.repo_root = Some(root.clone()),
        },
        Command::Perf { command } => match command {
            PerfCommand::Validate(args) => args.repo_root = Some(root.clone()),
            PerfCommand::Run(args) => args.repo_root = Some(root.clone()),
            PerfCommand::Diff(args) => args.repo_root = Some(root.clone()),
            PerfCommand::ColdStart(args) => args.repo_root = Some(root.clone()),
            PerfCommand::Kind(args) => args.repo_root = Some(root.clone()),
            PerfCommand::Benches { command } => match command {
                crate::cli::PerfBenchesCommand::List(args) => args.repo_root = Some(root.clone()),
            },
            PerfCommand::CliUx { command } => match command {
                crate::cli::PerfCliUxCommand::Bench(args) => args.repo_root = Some(root.clone()),
                crate::cli::PerfCliUxCommand::Diff(args) => args.repo_root = Some(root.clone()),
            },
        },
        Command::Policies { command } => match command {
            PoliciesCommand::List { repo_root, .. }
            | PoliciesCommand::Explain { repo_root, .. }
            | PoliciesCommand::Report { repo_root, .. }
            | PoliciesCommand::Print { repo_root, .. }
            | PoliciesCommand::Validate { repo_root, .. } => *repo_root = Some(root.clone()),
        },
        Command::Suites { command } => match command {
            crate::cli::SuitesCommand::Run { repo_root, .. }
            | crate::cli::SuitesCommand::List { repo_root, .. }
            | crate::cli::SuitesCommand::Describe { repo_root, .. }
            | crate::cli::SuitesCommand::Last { repo_root, .. }
            | crate::cli::SuitesCommand::History { repo_root, .. }
            | crate::cli::SuitesCommand::Report { repo_root, .. }
            | crate::cli::SuitesCommand::Diff { repo_root, .. }
            | crate::cli::SuitesCommand::Lint { repo_root, .. } => *repo_root = Some(root.clone()),
        },
        Command::Tests { command } => match command {
            TestsCommand::List { repo_root, .. }
            | TestsCommand::Run { repo_root, .. }
            | TestsCommand::Doctor { repo_root, .. } => *repo_root = Some(root.clone()),
        },
        Command::List { repo_root, .. }
        | Command::Describe { repo_root, .. }
        | Command::Run { repo_root, .. } => *repo_root = Some(root.clone()),
        Command::Registry { command } => match command {
            RegistryCommand::Status { repo_root, .. }
            | RegistryCommand::Doctor { repo_root, .. } => *repo_root = Some(root.clone()),
        },
        Command::Check { command } | Command::Checks { command } => match command {
            crate::cli::CheckCommand::List { repo_root, .. }
            | crate::cli::CheckCommand::Explain { repo_root, .. }
            | crate::cli::CheckCommand::Run { repo_root, .. }
            | crate::cli::CheckCommand::Doctor { repo_root, .. } => {
                *repo_root = Some(root.clone())
            }
        },
        Command::Demo { command } => match command {
            crate::cli::DemoCommand::Quickstart(args) => args.repo_root = Some(root.clone()),
        },
        Command::Ci { command } | Command::Workflows { command } => match command {
            crate::cli::WorkflowsCommand::Lanes { command } => match command {
                crate::cli::CiLanesCommand::List { repo_root, .. }
                | crate::cli::CiLanesCommand::Explain { repo_root, .. }
                | crate::cli::CiLanesCommand::Validate { repo_root, .. } => {
                    *repo_root = Some(root.clone())
                }
            },
            crate::cli::WorkflowsCommand::Simulate { repo_root, .. } => {
                *repo_root = Some(root.clone())
            }
            crate::cli::WorkflowsCommand::EnvContract { command } => match command {
                crate::cli::CiEnvContractCommand::Validate { repo_root, .. } => {
                    *repo_root = Some(root.clone())
                }
            },
            crate::cli::WorkflowsCommand::Validate { repo_root, .. }
            | crate::cli::WorkflowsCommand::Doctor { repo_root, .. }
            | crate::cli::WorkflowsCommand::Surface { repo_root, .. }
            | crate::cli::WorkflowsCommand::Explain { repo_root, .. }
            | crate::cli::WorkflowsCommand::Report { repo_root, .. }
            | crate::cli::WorkflowsCommand::Verify { repo_root, .. } => {
                *repo_root = Some(root.clone())
            }
        },
        Command::Validate { repo_root, .. } => {
            if repo_root.is_none() {
                *repo_root = Some(root.clone());
            }
        }
        Command::Release { command } => match command {
            ReleaseCommand::Plan(args) => {
                if args.repo_root.is_none() {
                    args.repo_root = Some(root.clone());
                }
            }
            ReleaseCommand::CompatibilityCheck(args) => {
                if args.repo_root.is_none() {
                    args.repo_root = Some(root.clone());
                }
            }
            ReleaseCommand::UpgradePlan(args) => {
                if args.repo_root.is_none() {
                    args.repo_root = Some(root.clone());
                }
            }
            ReleaseCommand::RollbackPlan(args) => {
                if args.repo_root.is_none() {
                    args.repo_root = Some(root.clone());
                }
            }
            ReleaseCommand::Validate(args) => {
                if args.repo_root.is_none() {
                    args.repo_root = Some(root.clone());
                }
            }
            ReleaseCommand::Tag(args) => {
                if args.repo_root.is_none() {
                    args.repo_root = Some(root.clone());
                }
            }
            ReleaseCommand::Notes(args) => {
                if args.repo_root.is_none() {
                    args.repo_root = Some(root.clone());
                }
            }
            ReleaseCommand::Check(args) => {
                if args.repo_root.is_none() {
                    args.repo_root = Some(root.clone());
                }
            }
            ReleaseCommand::Rebuild { command } => match command {
                crate::cli::ReleaseRebuildCommand::Verify(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
            },
            ReleaseCommand::Reproducibility { command } => match command {
                crate::cli::ReleaseReproducibilityCommand::Report(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
            },
            ReleaseCommand::Version { command } => match command {
                crate::cli::ReleaseVersionCommand::Check(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
            },
            ReleaseCommand::Changelog { command } => match command {
                crate::cli::ReleaseChangelogCommand::Generate(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
                crate::cli::ReleaseChangelogCommand::Validate(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
            },
            ReleaseCommand::Manifest { command } => match command {
                crate::cli::ReleaseManifestCommand::Generate(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
                crate::cli::ReleaseManifestCommand::Validate(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
            },
            ReleaseCommand::Checksums { command } => match command {
                ReleaseChecksumsCommand::Generate(args) | ReleaseChecksumsCommand::Verify(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
            },
            ReleaseCommand::Bundle { command } => match command {
                crate::cli::ReleaseBundleCommand::Build(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
                crate::cli::ReleaseBundleCommand::Verify(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
                crate::cli::ReleaseBundleCommand::Hash(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
            },
            ReleaseCommand::ReadinessReport(args) | ReleaseCommand::LaunchChecklist(args) => {
                if args.repo_root.is_none() {
                    args.repo_root = Some(root.clone());
                }
            }
            ReleaseCommand::Sign(args) => {
                if args.repo_root.is_none() {
                    args.repo_root = Some(root.clone());
                }
            }
            ReleaseCommand::Verify(args) => {
                if args.repo_root.is_none() {
                    args.repo_root = Some(root.clone());
                }
            }
            ReleaseCommand::Diff(args) => {
                if args.repo_root.is_none() {
                    args.repo_root = Some(root.clone());
                }
            }
            ReleaseCommand::Packet(args) => {
                if args.repo_root.is_none() {
                    args.repo_root = Some(root.clone());
                }
            }
            ReleaseCommand::Crates { command } => match command {
                ReleaseCratesCommand::List(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
                ReleaseCratesCommand::ValidateMetadata(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
                ReleaseCratesCommand::ValidatePublishFlags(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
                ReleaseCratesCommand::DryRun(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
                ReleaseCratesCommand::PublishPlan(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
            },
            ReleaseCommand::ApiSurface { command } => match command {
                ReleaseApiSurfaceCommand::Snapshot(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
            },
            ReleaseCommand::Semver { command } => match command {
                ReleaseSemverCommand::Check(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
            },
            ReleaseCommand::Msrv { command } => match command {
                ReleaseMsrvCommand::Verify(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
            },
            ReleaseCommand::Images { command } => match command {
                ReleaseImagesCommand::ValidateLabels(args)
                | ReleaseImagesCommand::ValidateTags(args)
                | ReleaseImagesCommand::ValidateBaseDigests(args)
                | ReleaseImagesCommand::SbomVerify(args)
                | ReleaseImagesCommand::ProvenanceVerify(args)
                | ReleaseImagesCommand::ScanVerify(args)
                | ReleaseImagesCommand::SmokeVerify(args)
                | ReleaseImagesCommand::SizeReport(args)
                | ReleaseImagesCommand::RuntimeHardeningVerify(args)
                | ReleaseImagesCommand::RuntimeCommandVerify(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
                ReleaseImagesCommand::ManifestGenerate(args)
                | ReleaseImagesCommand::ManifestVerify(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
                ReleaseImagesCommand::ReleaseNotesCheck(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
                ReleaseImagesCommand::ChangelogExtract(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
                ReleaseImagesCommand::IntegrationVerify(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
                ReleaseImagesCommand::BuildReproducibilityCheck(args)
                | ReleaseImagesCommand::LockedDependenciesVerify(args)
                | ReleaseImagesCommand::LockDriftVerify(args)
                | ReleaseImagesCommand::ReadinessSummary(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
            },
            ReleaseCommand::Ops { command } => match command {
                ReleaseOpsCommand::Package(args)
                | ReleaseOpsCommand::ValidatePackage(args)
                | ReleaseOpsCommand::CompatibilityMatrix(args)
                | ReleaseOpsCommand::DigestVerify(args)
                | ReleaseOpsCommand::ValuesCoverage(args)
                | ReleaseOpsCommand::ProfilesVerify(args)
                | ReleaseOpsCommand::LineageGenerate(args)
                | ReleaseOpsCommand::ProvenanceVerify(args)
                | ReleaseOpsCommand::ReadinessSummary(args)
                | ReleaseOpsCommand::ScenarioEvidenceVerify(args)
                | ReleaseOpsCommand::PublishPlan(args) => {
                    if args.repo_root.is_none() {
                        args.repo_root = Some(root.clone());
                    }
                }
                ReleaseOpsCommand::Push(args) => {
                    if args.common.repo_root.is_none() {
                        args.common.repo_root = Some(root.clone());
                    }
                }
                ReleaseOpsCommand::PullTest(args) => {
                    if args.common.repo_root.is_none() {
                        args.common.repo_root = Some(root.clone());
                    }
                }
                ReleaseOpsCommand::BundleBuild(args) | ReleaseOpsCommand::BundleVerify(args) => {
                    if args.common.repo_root.is_none() {
                        args.common.repo_root = Some(root.clone());
                    }
                }
            },
        },
        Command::Version { .. }
        | Command::Help { .. }
        | Command::Docker { .. }
        | Command::Build { .. }
        | Command::Gates { .. }
        | Command::Capabilities { .. } => {}
    }
}

fn force_json_docker(command: &mut crate::cli::DockerCommand) {
    match command {
        crate::cli::DockerCommand::Build(args)
        | crate::cli::DockerCommand::Check(args)
        | crate::cli::DockerCommand::Smoke(args)
        | crate::cli::DockerCommand::Scan(args)
        | crate::cli::DockerCommand::Sbom(args)
        | crate::cli::DockerCommand::Lock(args) => args.format = FormatArg::Json,
        crate::cli::DockerCommand::Policy { command } => match command {
            crate::cli::DockerPolicyCommand::Check(args) => args.format = FormatArg::Json,
        },
        crate::cli::DockerCommand::Push(args) | crate::cli::DockerCommand::Release(args) => {
            args.common.format = FormatArg::Json
        }
    }
}

fn force_json_tests(command: &mut TestsCommand) {
    match command {
        TestsCommand::List { format, .. }
        | TestsCommand::Run { format, .. }
        | TestsCommand::Doctor { format, .. } => *format = FormatArg::Json,
    }
}
