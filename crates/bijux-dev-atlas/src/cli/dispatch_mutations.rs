// SPDX-License-Identifier: Apache-2.0

use crate::cli::{
    CheckCommand, CheckRegistryCommand, Command, ConfigsCommand, ContractsCommand,
    DocsCommand, FormatArg, OpsCommand, PoliciesCommand, ReleaseCommand,
};

pub(super) fn force_json_output(command: &mut Command) {
    match command {
        Command::Version { format, .. } => *format = FormatArg::Json,
        Command::Help { format, .. } => *format = FormatArg::Json,
        Command::Ops { command } => force_json_ops(command),
        Command::Docs { command } => force_json_docs(command),
        Command::Demo { command } => force_json_demo(command),
        Command::Contracts { command } => force_json_contracts(command),
        Command::Configs { command } => force_json_configs(command),
        Command::Policies { command } => force_json_policies(command),
        Command::Check { command } => force_json_check(command),
        Command::Validate { format, .. } => *format = FormatArg::Json,
        Command::Release { command } => match command {
            ReleaseCommand::Check(args) => args.format = FormatArg::Json,
        },
        Command::Docker { .. }
        | Command::Build { .. }
        | Command::Ci { .. }
        | Command::Workflows { .. }
        | Command::Gates { .. }
        | Command::Capabilities { .. } => {}
    }
}

fn force_json_demo(command: &mut crate::cli::DemoCommand) {
    match command {
        crate::cli::DemoCommand::Quickstart(args) => args.format = FormatArg::Json,
    }
}

fn force_json_ops(command: &mut OpsCommand) {
    match command {
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
        | OpsCommand::Up(common)
        | OpsCommand::Down(common)
        | OpsCommand::Clean(common)
        | OpsCommand::Cleanup(common)
        | OpsCommand::K8sPlan(common)
        | OpsCommand::K8sDryRun(common)
        | OpsCommand::K8sPorts(common)
        | OpsCommand::K8sConformance(common)
        | OpsCommand::LoadPlan { common, .. }
        | OpsCommand::LoadRun { common, .. }
        | OpsCommand::LoadReport { common, .. } => common.format = FormatArg::Json,
        OpsCommand::Explain { common, .. } => common.format = FormatArg::Json,
        OpsCommand::Render(args) => args.common.format = FormatArg::Json,
        OpsCommand::Install(args) => args.common.format = FormatArg::Json,
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
            | crate::cli::OpsGenerateCommand::Runbook { common, .. } => {
                common.format = FormatArg::Json
            }
        },
        OpsCommand::Evidence { command } => match command {
            crate::cli::OpsEvidenceCommand::Collect(common)
            | crate::cli::OpsEvidenceCommand::Verify(common) => common.format = FormatArg::Json,
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
            | crate::cli::OpsK8sCommand::Plan(common)
            | crate::cli::OpsK8sCommand::Uninstall(common)
            | crate::cli::OpsK8sCommand::Ports(common)
            | crate::cli::OpsK8sCommand::Diff(common)
            | crate::cli::OpsK8sCommand::Rollout(common)
            | crate::cli::OpsK8sCommand::DryRun(common)
            | crate::cli::OpsK8sCommand::Conformance(common)
            | crate::cli::OpsK8sCommand::Test(common)
            | crate::cli::OpsK8sCommand::Smoke(common) => common.format = FormatArg::Json,
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
        OpsCommand::Obs { command } => match command {
            crate::cli::OpsObsCommand::Up(common)
            | crate::cli::OpsObsCommand::Down(common)
            | crate::cli::OpsObsCommand::Validate(common)
            | crate::cli::OpsObsCommand::Snapshot(common)
            | crate::cli::OpsObsCommand::Dashboards(common)
            | crate::cli::OpsObsCommand::Verify(common) => common.format = FormatArg::Json,
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
        | DocsCommand::Validate(common)
        | DocsCommand::Build(common)
        | DocsCommand::Clean(common)
        | DocsCommand::Lint(common)
        | DocsCommand::Links(common)
        | DocsCommand::Inventory(common)
        | DocsCommand::ShrinkReport(common) => common.format = FormatArg::Json,
        DocsCommand::Serve(args) => args.common.format = FormatArg::Json,
        DocsCommand::Grep(args) => args.common.format = FormatArg::Json,
        DocsCommand::Registry { command } => match command {
            crate::cli::DocsRegistryCommand::Build(common)
            | crate::cli::DocsRegistryCommand::Validate(common) => common.format = FormatArg::Json,
        },
        DocsCommand::Reference { command } => match command {
            crate::cli::DocsReferenceCommand::Generate(common)
            | crate::cli::DocsReferenceCommand::Check(common) => common.format = FormatArg::Json,
        },
    }
}

fn force_json_configs(command: &mut ConfigsCommand) {
    match command {
        ConfigsCommand::Print(common)
        | ConfigsCommand::List(common)
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

fn force_json_contracts(command: &mut ContractsCommand) {
    match command {
        ContractsCommand::Docker(args) => args.json = true,
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

fn force_json_check(command: &mut CheckCommand) {
    match command {
        CheckCommand::Registry { command } => match command {
            CheckRegistryCommand::Doctor { format, .. } => *format = FormatArg::Json,
        },
        CheckCommand::List { format, json, .. } => {
            *format = FormatArg::Json;
            *json = true;
        }
        CheckCommand::Explain { format, .. }
        | CheckCommand::Doctor { format, .. }
        | CheckCommand::Run { format, .. }
        | CheckCommand::TreeBudgets { format, .. }
        | CheckCommand::RepoDoctor { format, .. } => *format = FormatArg::Json,
    }
}

pub(super) fn apply_fail_fast(command: &mut Command) {
    match command {
        Command::Check {
            command: CheckCommand::Run { fail_fast, .. },
        } => *fail_fast = true,
        Command::Docs { command } => match command {
            DocsCommand::Check(common)
            | DocsCommand::Doctor(common)
            | DocsCommand::Validate(common)
            | DocsCommand::Lint(common)
            | DocsCommand::Links(common)
            | DocsCommand::VerifyContracts(common) => common.strict = true,
            DocsCommand::Build(_)
            | DocsCommand::Serve(_)
            | DocsCommand::Clean(_)
            | DocsCommand::Inventory(_)
            | DocsCommand::ShrinkReport(_)
            | DocsCommand::Grep(_) => {}
            DocsCommand::Registry { command } => match command {
                crate::cli::DocsRegistryCommand::Build(_)
                | crate::cli::DocsRegistryCommand::Validate(_) => {}
            },
            DocsCommand::Reference { command } => match command {
                crate::cli::DocsReferenceCommand::Generate(_)
                | crate::cli::DocsReferenceCommand::Check(_) => {}
            },
        },
        Command::Configs { command } => match command {
            ConfigsCommand::Doctor(common)
            | ConfigsCommand::Verify(common)
            | ConfigsCommand::Validate(common)
            | ConfigsCommand::Lint(common)
            | ConfigsCommand::Inventory(common)
            | ConfigsCommand::Diff(common) => common.strict = true,
            ConfigsCommand::Fmt { check, .. } => *check = true,
            ConfigsCommand::Print(_)
            | ConfigsCommand::List(_)
            | ConfigsCommand::Compile(_) => {}
        },
        _ => {}
    }
}

pub(super) fn propagate_repo_root(command: &mut Command, repo_root: Option<std::path::PathBuf>) {
    let Some(root) = repo_root else {
        return;
    };
    match command {
        Command::Ops { command } => match command {
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
            | OpsCommand::Up(common)
            | OpsCommand::Down(common)
            | OpsCommand::Clean(common)
            | OpsCommand::Cleanup(common)
            | OpsCommand::K8sPlan(common)
            | OpsCommand::K8sDryRun(common)
            | OpsCommand::K8sPorts(common)
            | OpsCommand::K8sConformance(common) => common.repo_root = Some(root.clone()),
            OpsCommand::LoadPlan { common, .. }
            | OpsCommand::LoadRun { common, .. }
            | OpsCommand::LoadReport { common, .. } => common.repo_root = Some(root.clone()),
            OpsCommand::Explain { common, .. } => common.repo_root = Some(root.clone()),
            OpsCommand::Render(args) => args.common.repo_root = Some(root.clone()),
            OpsCommand::Install(args) => args.common.repo_root = Some(root.clone()),
            OpsCommand::Status(args) => args.common.repo_root = Some(root.clone()),
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
                | crate::cli::OpsGenerateCommand::Runbook { common, .. } => {
                    common.repo_root = Some(root.clone())
                }
            },
            OpsCommand::Evidence { command } => match command {
                crate::cli::OpsEvidenceCommand::Collect(common)
                | crate::cli::OpsEvidenceCommand::Verify(common) => {
                    common.repo_root = Some(root.clone())
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
                | crate::cli::OpsK8sCommand::Plan(common)
                | crate::cli::OpsK8sCommand::Uninstall(common)
                | crate::cli::OpsK8sCommand::Ports(common)
                | crate::cli::OpsK8sCommand::Diff(common)
                | crate::cli::OpsK8sCommand::Rollout(common)
                | crate::cli::OpsK8sCommand::DryRun(common)
                | crate::cli::OpsK8sCommand::Conformance(common)
                | crate::cli::OpsK8sCommand::Test(common)
                | crate::cli::OpsK8sCommand::Smoke(common) => common.repo_root = Some(root.clone()),
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
            OpsCommand::Obs { command } => match command {
                crate::cli::OpsObsCommand::Up(common)
                | crate::cli::OpsObsCommand::Down(common)
                | crate::cli::OpsObsCommand::Validate(common)
                | crate::cli::OpsObsCommand::Snapshot(common)
                | crate::cli::OpsObsCommand::Dashboards(common)
                | crate::cli::OpsObsCommand::Verify(common) => {
                    common.repo_root = Some(root.clone())
                }
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
            | DocsCommand::Validate(common)
            | DocsCommand::Build(common)
            | DocsCommand::Clean(common)
            | DocsCommand::Lint(common)
            | DocsCommand::Links(common)
            | DocsCommand::Inventory(common)
            | DocsCommand::ShrinkReport(common) => common.repo_root = Some(root.clone()),
            DocsCommand::Serve(args) => args.common.repo_root = Some(root.clone()),
            DocsCommand::Grep(args) => args.common.repo_root = Some(root.clone()),
            DocsCommand::Registry { command } => match command {
                crate::cli::DocsRegistryCommand::Build(common)
                | crate::cli::DocsRegistryCommand::Validate(common) => {
                    common.repo_root = Some(root.clone())
                }
            },
            DocsCommand::Reference { command } => match command {
                crate::cli::DocsReferenceCommand::Generate(common)
                | crate::cli::DocsReferenceCommand::Check(common) => {
                    common.repo_root = Some(root.clone())
                }
            },
        },
        Command::Configs { command } => match command {
            ConfigsCommand::Print(common)
            | ConfigsCommand::List(common)
            | ConfigsCommand::Verify(common)
            | ConfigsCommand::Doctor(common)
            | ConfigsCommand::Validate(common)
            | ConfigsCommand::Lint(common)
            | ConfigsCommand::Inventory(common)
            | ConfigsCommand::Compile(common)
            | ConfigsCommand::Diff(common) => common.repo_root = Some(root.clone()),
            ConfigsCommand::Fmt { common, .. } => common.repo_root = Some(root.clone()),
        },
        Command::Policies { command } => match command {
            PoliciesCommand::List { repo_root, .. }
            | PoliciesCommand::Explain { repo_root, .. }
            | PoliciesCommand::Report { repo_root, .. }
            | PoliciesCommand::Print { repo_root, .. }
            | PoliciesCommand::Validate { repo_root, .. } => *repo_root = Some(root.clone()),
        },
        Command::Check { command } => match command {
            CheckCommand::Registry { command } => match command {
                CheckRegistryCommand::Doctor { repo_root, .. } => *repo_root = Some(root.clone()),
            },
            CheckCommand::List { repo_root, .. }
            | CheckCommand::Explain { repo_root, .. }
            | CheckCommand::Doctor { repo_root, .. }
            | CheckCommand::Run { repo_root, .. }
            | CheckCommand::TreeBudgets { repo_root, .. }
            | CheckCommand::RepoDoctor { repo_root, .. } => *repo_root = Some(root.clone()),
        },
        Command::Demo { command } => match command {
            crate::cli::DemoCommand::Quickstart(args) => args.repo_root = Some(root.clone()),
        },
        Command::Contracts { command } => match command {
            ContractsCommand::Docker(args) => args.repo_root = Some(root.clone()),
        },
        Command::Ci { command } | Command::Workflows { command } => match command {
            crate::cli::WorkflowsCommand::Validate { repo_root, .. }
            | crate::cli::WorkflowsCommand::Doctor { repo_root, .. }
            | crate::cli::WorkflowsCommand::Surface { repo_root, .. } => {
                *repo_root = Some(root.clone())
            }
        },
        Command::Validate { repo_root, .. } => {
            if repo_root.is_none() {
                *repo_root = Some(root.clone());
            }
        }
        Command::Release { command } => match command {
            ReleaseCommand::Check(args) => {
                if args.repo_root.is_none() {
                    args.repo_root = Some(root.clone());
                }
            }
        },
        Command::Version { .. }
        | Command::Help { .. }
        | Command::Docker { .. }
        | Command::Build { .. }
        | Command::Gates { .. }
        | Command::Capabilities { .. } => {}
    }
}
