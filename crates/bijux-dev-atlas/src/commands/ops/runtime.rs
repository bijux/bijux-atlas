// SPDX-License-Identifier: Apache-2.0

use crate::cli::OpsInstallArgs;
use crate::cli::{
    OpsCommonArgs, OpsDatasetsCommand, OpsE2eCommand, OpsEvidenceCommand, OpsInventoryCommand,
    OpsGenerateCommand, OpsK8sCommand, OpsLoadBaselineCommand, OpsLoadCommand, OpsObsCommand,
    OpsObsDrillCommand, OpsPinsCommand, OpsReportCommand, OpsSchemaCommand, OpsStackCommand,
    OpsSuiteCommand, OpsToolsCommand,
};
use crate::ops_support::{
    build_ops_run_report, load_load_manifest, load_stack_manifest, load_stack_pins,
    load_toolchain_inventory_for_ops, load_tools_manifest, ops_exit, ops_pins_check_payload,
    parse_tool_overrides, render_ops_human, render_ops_validation_output, run_ops_checks,
    validate_load_manifest, validate_pins_completeness, validate_stack_manifest,
    verify_tools_snapshot, ToolMismatchCode,
};
use crate::ops_support::{
    emit_payload, load_profiles, resolve_ops_root, resolve_profile, run_id_or_default, sha256_hex,
};
use crate::*;
use std::io::{self, Write};

#[path = "runtime_mod/core_handler.rs"]
mod core_handler;
#[path = "runtime_mod/execution_handler.rs"]
mod execution_handler;
#[path = "runtime_mod/profile_handler.rs"]
mod profile_handler;

fn command_common(command: &OpsCommand) -> Option<&OpsCommonArgs> {
    match command {
        OpsCommand::List(common)
        | OpsCommand::Doctor(common)
        | OpsCommand::Validate(common)
        | OpsCommand::Inventory(common)
        | OpsCommand::Docs(common)
        | OpsCommand::Conformance(common)
        | OpsCommand::Report(common)
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
        | OpsCommand::K8sConformance(common) => Some(common),
        OpsCommand::Explain { common, .. } | OpsCommand::ExplainProfile { common, .. } => {
            Some(common)
        }
        OpsCommand::LoadPlan { common, .. }
        | OpsCommand::LoadRun { common, .. }
        | OpsCommand::LoadReport { common, .. } => Some(common),
        OpsCommand::Render(args) => Some(&args.common),
        OpsCommand::Install(args) => Some(&args.common),
        OpsCommand::Status(args) => Some(&args.common),
        OpsCommand::Reset(args) => Some(&args.common),
        OpsCommand::K8sApply(args) => Some(&args.common),
        OpsCommand::K8sWait(args) => Some(&args.common),
        OpsCommand::K8sLogs(args) => Some(&args.common),
        OpsCommand::K8sPortForward(args) => Some(&args.common),
        OpsCommand::Pins { command } => match command {
            OpsPinsCommand::Check(common) | OpsPinsCommand::Update { common, .. } => Some(common),
        },
        OpsCommand::Generate { command } => match command {
            OpsGenerateCommand::PinsIndex { common, .. }
            | OpsGenerateCommand::SurfaceList { common, .. }
            | OpsGenerateCommand::Runbook { common, .. } => Some(common),
        },
        OpsCommand::Evidence { command } => match command {
            OpsEvidenceCommand::Collect(common) | OpsEvidenceCommand::Verify(common) => Some(common),
        },
        OpsCommand::Schema { .. }
        | OpsCommand::InventoryDomain { .. }
        | OpsCommand::ReportDomain { .. }
        | OpsCommand::Tools { .. }
        | OpsCommand::Suite { .. }
        | OpsCommand::Stack { .. }
        | OpsCommand::K8s { .. }
        | OpsCommand::Load { .. }
        | OpsCommand::Datasets { .. }
        | OpsCommand::E2e { .. }
        | OpsCommand::Obs { .. } => None,
    }
}

fn command_run_id(command: &OpsCommand) -> String {
    command_common(command)
        .and_then(|common| common.run_id.clone())
        .unwrap_or_else(|| "ops_run".to_string())
}

pub(crate) fn run_ops_command(quiet: bool, debug: bool, command: OpsCommand) -> i32 {
    let command = match command {
        OpsCommand::Stack { command } => match command {
            OpsStackCommand::Plan(common) => OpsCommand::Plan(common),
            OpsStackCommand::Up(common) => OpsCommand::Up(common),
            OpsStackCommand::Down(common) => OpsCommand::Down(common),
            OpsStackCommand::Logs(common) => OpsCommand::Explain {
                action: "stack-logs".to_string(),
                common,
            },
            OpsStackCommand::Ports(common) => OpsCommand::Explain {
                action: "stack-ports".to_string(),
                common,
            },
            OpsStackCommand::Versions(common) => OpsCommand::Explain {
                action: "stack-versions".to_string(),
                common,
            },
            OpsStackCommand::Doctor(common) => OpsCommand::Doctor(common),
            OpsStackCommand::Status(mut args) => {
                args.target = OpsStatusTarget::K8s;
                OpsCommand::Status(args)
            }
            OpsStackCommand::Reset(args) => OpsCommand::Reset(args),
        },
        OpsCommand::K8s { command } => match command {
            OpsK8sCommand::Render(args) => OpsCommand::Render(args),
            OpsK8sCommand::Install(args) => OpsCommand::Install(args),
            OpsK8sCommand::Uninstall(common) => OpsCommand::Down(common),
            OpsK8sCommand::Diff(common) => OpsCommand::Explain {
                action: "k8s-diff".to_string(),
                common,
            },
            OpsK8sCommand::Rollout(common) => OpsCommand::Explain {
                action: "k8s-rollout".to_string(),
                common,
            },
            OpsK8sCommand::Plan(common) => OpsCommand::K8sPlan(common),
            OpsK8sCommand::Apply(args) => OpsCommand::K8sApply(args),
            OpsK8sCommand::DryRun(common) => OpsCommand::K8sDryRun(common),
            OpsK8sCommand::Conformance(common) => OpsCommand::K8sConformance(common),
            OpsK8sCommand::Wait(args) => OpsCommand::K8sWait(args),
            OpsK8sCommand::Logs(args) => OpsCommand::K8sLogs(args),
            OpsK8sCommand::PortForward(args) => OpsCommand::K8sPortForward(args),
            OpsK8sCommand::Test(common) => OpsCommand::K8sConformance(common),
            OpsK8sCommand::Status(args) => OpsCommand::Status(args),
        },
        OpsCommand::Load { command } => match command {
            OpsLoadCommand::Plan { suite, common } => OpsCommand::LoadPlan { suite, common },
            OpsLoadCommand::Run { suite, common } => OpsCommand::LoadRun { suite, common },
            OpsLoadCommand::Report { suite, common } => OpsCommand::LoadReport { suite, common },
            OpsLoadCommand::Baseline { command } => match command {
                OpsLoadBaselineCommand::Update(common) => OpsCommand::Explain {
                    action: "load-baseline-update".to_string(),
                    common,
                },
            },
            OpsLoadCommand::Evaluate(common) => OpsCommand::Explain {
                action: "load-evaluate".to_string(),
                common,
            },
            OpsLoadCommand::ListSuites(common) => OpsCommand::Suite {
                command: OpsSuiteCommand::List(common),
            },
        },
        OpsCommand::Datasets { command } => match command {
            OpsDatasetsCommand::List(common) => OpsCommand::Explain {
                action: "datasets-list".to_string(),
                common,
            },
            OpsDatasetsCommand::Ingest(common) => OpsCommand::Explain {
                action: "datasets-ingest".to_string(),
                common,
            },
            OpsDatasetsCommand::Publish(common) => OpsCommand::Explain {
                action: "datasets-publish".to_string(),
                common,
            },
            OpsDatasetsCommand::Promote(common) => OpsCommand::Explain {
                action: "datasets-promote".to_string(),
                common,
            },
            OpsDatasetsCommand::Rollback(common) => OpsCommand::Explain {
                action: "datasets-rollback".to_string(),
                common,
            },
            OpsDatasetsCommand::Qc(common) => OpsCommand::Explain {
                action: "datasets-qc".to_string(),
                common,
            },
        },
        OpsCommand::E2e { command } => match command {
            OpsE2eCommand::Run(common) => OpsCommand::Explain {
                action: "e2e-run".to_string(),
                common,
            },
            OpsE2eCommand::Smoke(common) => OpsCommand::Explain {
                action: "e2e-smoke".to_string(),
                common,
            },
            OpsE2eCommand::Realdata(common) => OpsCommand::Explain {
                action: "e2e-realdata".to_string(),
                common,
            },
            OpsE2eCommand::ListSuites(common) => OpsCommand::Suite {
                command: OpsSuiteCommand::List(common),
            },
        },
        OpsCommand::Obs { command } => match command {
            OpsObsCommand::Up(common) => OpsCommand::Explain {
                action: "observe-up".to_string(),
                common,
            },
            OpsObsCommand::Down(common) => OpsCommand::Explain {
                action: "observe-down".to_string(),
                common,
            },
            OpsObsCommand::Validate(common) => OpsCommand::Explain {
                action: "observe-validate".to_string(),
                common,
            },
            OpsObsCommand::Snapshot(common) => OpsCommand::Explain {
                action: "observe-snapshot".to_string(),
                common,
            },
            OpsObsCommand::Dashboards(common) => OpsCommand::Explain {
                action: "observe-dashboards".to_string(),
                common,
            },
            OpsObsCommand::Drill { command } => match command {
                OpsObsDrillCommand::Run(common) => OpsCommand::Explain {
                    action: "obs-drill-run".to_string(),
                    common,
                },
            },
            OpsObsCommand::Verify(common) => OpsCommand::Explain {
                action: "obs-verify".to_string(),
                common,
            },
        },
        OpsCommand::Schema { command } => match command {
            OpsSchemaCommand::Validate(common) => OpsCommand::Validate(common),
            OpsSchemaCommand::Diff(common) => OpsCommand::Explain {
                action: "schema-diff".to_string(),
                common,
            },
            OpsSchemaCommand::Coverage(common) => OpsCommand::Explain {
                action: "schema-coverage".to_string(),
                common,
            },
            OpsSchemaCommand::RegenIndex(common) => OpsCommand::Explain {
                action: "schema-regen-index".to_string(),
                common,
            },
        },
        OpsCommand::InventoryDomain { command } => match command {
            OpsInventoryCommand::Validate(common) => OpsCommand::Validate(common),
            OpsInventoryCommand::Graph(common) => OpsCommand::Explain {
                action: "inventory-graph".to_string(),
                common,
            },
            OpsInventoryCommand::Diff(common) => OpsCommand::Explain {
                action: "inventory-diff".to_string(),
                common,
            },
            OpsInventoryCommand::Coverage(common) => OpsCommand::Explain {
                action: "inventory-coverage".to_string(),
                common,
            },
            OpsInventoryCommand::OrphanCheck(common) => OpsCommand::Explain {
                action: "inventory-orphan-check".to_string(),
                common,
            },
        },
        OpsCommand::ReportDomain { command } => match command {
            OpsReportCommand::Generate(common) => OpsCommand::Report(common),
            OpsReportCommand::Diff(common) => OpsCommand::Explain {
                action: "report-diff".to_string(),
                common,
            },
            OpsReportCommand::Readiness(common) => OpsCommand::Explain {
                action: "report-readiness".to_string(),
                common,
            },
            OpsReportCommand::Bundle(common) => OpsCommand::Explain {
                action: "report-bundle".to_string(),
                common,
            },
        },
        OpsCommand::Evidence { command } => match command {
            OpsEvidenceCommand::Collect(common) => OpsCommand::Explain {
                action: "evidence-collect".to_string(),
                common,
            },
            OpsEvidenceCommand::Verify(common) => OpsCommand::Explain {
                action: "evidence-verify".to_string(),
                common,
            },
        },
        other => other,
    };

    let run_id = command_run_id(&command);
    if debug {
        let _ = writeln!(
            io::stderr(),
            "{}",
            serde_json::json!({
                "event": "ops.command.start",
                "run_id": run_id,
            })
        );
    }

    let run = core_handler::dispatch_core(command.clone(), debug)
        .or_else(|err| {
            if err == "__UNHANDLED__" {
                profile_handler::dispatch_profiles(command.clone(), debug)
            } else {
                Err(err)
            }
        })
        .or_else(|err| {
            if err == "__UNHANDLED__" {
                execution_handler::dispatch_execution(command, debug)
            } else {
                Err(err)
            }
        });

    if debug {
        let _ = writeln!(
            io::stderr(),
            "{}",
            serde_json::json!({
                "event": "ops.command.completed",
                "run_id": run_id,
                "ok": run.is_ok(),
            })
        );
    }

    match run {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                let _ = writeln!(io::stdout(), "{rendered}");
            }
            code
        }
        Err(err) => {
            let _ = writeln!(io::stderr(), "bijux-dev-atlas ops failed: {err}");
            if err.contains("unknown ops action")
                || err.contains("unknown suite")
                || err.contains("requires --")
            {
                ops_exit::USAGE
            } else if err.contains("missing required ops tools")
                || err.contains("required external tools are missing")
            {
                ops_exit::TOOL_MISSING
            } else if err.contains("OPS_MANIFEST_ERROR")
                || err.contains("OPS_SCHEMA_ERROR")
                || err.contains("cannot resolve ops root")
            {
                ops_exit::INFRA
            } else {
                ops_exit::FAIL
            }
        }
    }
}
