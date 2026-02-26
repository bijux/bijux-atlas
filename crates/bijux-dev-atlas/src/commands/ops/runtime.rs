// SPDX-License-Identifier: Apache-2.0

use crate::cli::OpsInstallArgs;
use crate::cli::{
    OpsE2eCommand, OpsK8sCommand, OpsLoadCommand, OpsObsCommand, OpsObsDrillCommand,
    OpsStackCommand, OpsSuiteCommand, OpsToolsCommand,
};
use crate::ops_command_support::{
    build_ops_run_report, load_load_manifest, load_stack_manifest, load_stack_pins,
    load_toolchain_inventory_for_ops, load_tools_manifest, ops_exit, ops_pins_check_payload,
    parse_tool_overrides, render_ops_human, render_ops_validation_output, run_ops_checks,
    validate_load_manifest, validate_pins_completeness, validate_stack_manifest,
    verify_tools_snapshot, ToolMismatchCode,
};
use crate::ops_command_support::{
    emit_payload, load_profiles, resolve_ops_root, resolve_profile, run_id_or_default, sha256_hex,
};
use crate::*;

#[path = "runtime_mod/core_handler.rs"]
mod core_handler;
#[path = "runtime_mod/execution_handler.rs"]
mod execution_handler;
#[path = "runtime_mod/profile_handler.rs"]
mod profile_handler;

pub(crate) fn run_ops_command(quiet: bool, debug: bool, command: OpsCommand) -> i32 {
    let command = match command {
        OpsCommand::Stack { command } => match command {
            OpsStackCommand::Plan(common) => OpsCommand::Plan(common),
            OpsStackCommand::Up(common) => OpsCommand::Up(common),
            OpsStackCommand::Down(common) => OpsCommand::Down(common),
            OpsStackCommand::Status(mut args) => {
                args.target = OpsStatusTarget::K8s;
                OpsCommand::Status(args)
            }
            OpsStackCommand::Reset(args) => OpsCommand::Reset(args),
        },
        OpsCommand::K8s { command } => match command {
            OpsK8sCommand::Render(args) => OpsCommand::Render(args),
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
        },
        OpsCommand::E2e { command } => match command {
            OpsE2eCommand::Run(common) => OpsCommand::Explain {
                action: "e2e-run".to_string(),
                common,
            },
        },
        OpsCommand::Obs { command } => match command {
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
        other => other,
    };

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
        eprintln!(
            "{}",
            serde_json::json!({
                "event": "ops.command.completed",
                "ok": run.is_ok(),
            })
        );
    }

    match run {
        Ok((rendered, code)) => {
            if !quiet && !rendered.is_empty() {
                println!("{rendered}");
            }
            code
        }
        Err(err) => {
            eprintln!("bijux-dev-atlas ops failed: {err}");
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
