// SPDX-License-Identifier: Apache-2.0
//! Release lifecycle simulation commands for install-status flows.

use super::*;
use crate::cli;
use crate::ops_commands::{emit_payload, run_id_or_default};
use crate::{resolve_repo_root, OpsCommandError, OpsProcess};

pub(crate) fn run_ops_helm_upgrade(
    args: &crate::cli::OpsHelmUpgradeArgs,
) -> Result<(String, i32), String> {
    let common = &args.release.common;
    match args.release.cluster {
        crate::cli::OpsClusterTarget::Kind => {}
    }
    if !common.allow_subprocess {
        return Err("helm upgrade requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("helm upgrade requires --allow-write".to_string());
    }
    if !common.allow_network {
        return Err("helm upgrade requires --allow-network".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    ensure_simulation_context(&process, common.force)?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let profile = common.profile.clone().unwrap_or_else(|| "kind".to_string());
    let namespace = bijux_atlas_ops::workspace::profiles::simulation_namespace(
        &profile,
        args.release.namespace.as_deref(),
    );
    let target = match args.to {
        crate::cli::OpsHelmTarget::Current => {
            bijux_atlas_ops::lifecycle::release::contracts::ReleaseChartSource::Current
        }
        crate::cli::OpsHelmTarget::Previous => {
            bijux_atlas_ops::lifecycle::release::contracts::ReleaseChartSource::Previous
        }
    };
    let (envelope, exit_code) = bijux_atlas_ops::lifecycle::simulation::helm_upgrade_payload(
        &process,
        &repo_root,
        bijux_atlas_ops::lifecycle::simulation::HelmUpgradeRequest {
            run_id: run_id.as_str(),
            profile: &profile,
            namespace: &namespace,
            target,
            timeout_seconds: args.release.timeout_seconds,
        },
    )?;
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, exit_code))
}

pub(crate) fn run_ops_helm_rollback(
    args: &crate::cli::OpsHelmRollbackArgs,
) -> Result<(String, i32), String> {
    let common = &args.release.common;
    match args.release.cluster {
        crate::cli::OpsClusterTarget::Kind => {}
    }
    if !common.allow_subprocess {
        return Err("helm rollback requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("helm rollback requires --allow-write".to_string());
    }
    if !common.allow_network {
        return Err("helm rollback requires --allow-network".to_string());
    }
    if !matches!(args.to, crate::cli::OpsHelmTarget::Previous) {
        return Err("helm rollback currently supports only --to previous".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    ensure_simulation_context(&process, common.force)?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let profile = common.profile.clone().unwrap_or_else(|| "kind".to_string());
    let namespace = bijux_atlas_ops::workspace::profiles::simulation_namespace(
        &profile,
        args.release.namespace.as_deref(),
    );
    let (envelope, exit_code) = bijux_atlas_ops::lifecycle::simulation::helm_rollback_payload(
        &process,
        &repo_root,
        bijux_atlas_ops::lifecycle::simulation::HelmRollbackRequest {
            run_id: run_id.as_str(),
            profile: &profile,
            namespace: &namespace,
            timeout_seconds: args.release.timeout_seconds,
        },
    )?;
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, exit_code))
}

pub(crate) fn run_ops_smoke(args: &crate::cli::OpsSmokeArgs) -> Result<(String, i32), String> {
    let common = &args.common;
    match args.cluster {
        crate::cli::OpsClusterTarget::Kind => {}
    }
    if !common.allow_subprocess {
        return Err("k8s conformance requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("smoke requires --allow-write".to_string());
    }
    if !common.allow_network {
        return Err("smoke requires --allow-network".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    ensure_simulation_context(&process, common.force)?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let profile = common.profile.clone().unwrap_or_else(|| "kind".to_string());
    let namespace = bijux_atlas_ops::workspace::profiles::simulation_namespace(
        &profile,
        args.namespace.as_deref(),
    );
    let (envelope, exit_code) = bijux_atlas_ops::lifecycle::simulation::smoke_command_payload(
        &repo_root,
        run_id.as_str(),
        &namespace,
        args.local_port,
    )?;
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, exit_code))
}

fn run_collect_command(
    args: &crate::cli::OpsCollectArgs,
    action: impl FnOnce(&OpsProcess, &std::path::Path, &str, &str) -> Result<serde_json::Value, String>,
) -> Result<(String, i32), String> {
    let common = &args.common;
    match args.cluster {
        crate::cli::OpsClusterTarget::Kind => {}
    }
    if !common.allow_subprocess {
        return Err("collect requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("collect requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    ensure_simulation_context(&process, common.force)?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let profile = common.profile.clone().unwrap_or_else(|| "kind".to_string());
    let namespace = bijux_atlas_ops::workspace::profiles::simulation_namespace(
        &profile,
        args.namespace.as_deref(),
    );
    let envelope = action(&process, &repo_root, run_id.as_str(), &namespace)?;
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_logs_collect(
    args: &crate::cli::OpsCollectArgs,
) -> Result<(String, i32), String> {
    run_collect_command(args, |process, repo_root, run_id, namespace| {
        bijux_atlas_ops::lifecycle::simulation::logs_collect_payload(
            process, repo_root, run_id, namespace,
        )
    })
}

pub(crate) fn run_ops_describe_collect(
    args: &crate::cli::OpsCollectArgs,
) -> Result<(String, i32), String> {
    run_collect_command(args, |process, repo_root, run_id, namespace| {
        bijux_atlas_ops::lifecycle::simulation::describe_collect_payload(
            process, repo_root, run_id, namespace,
        )
    })
}

pub(crate) fn run_ops_events_collect(
    args: &crate::cli::OpsCollectArgs,
) -> Result<(String, i32), String> {
    run_collect_command(args, |process, repo_root, run_id, namespace| {
        bijux_atlas_ops::lifecycle::simulation::events_collect_payload(
            process, repo_root, run_id, namespace,
        )
    })
}

pub(crate) fn run_ops_resources_snapshot(
    args: &crate::cli::OpsCollectArgs,
) -> Result<(String, i32), String> {
    run_collect_command(args, |process, repo_root, run_id, namespace| {
        bijux_atlas_ops::lifecycle::simulation::resources_snapshot_payload(
            process, repo_root, run_id, namespace,
        )
    })
}

pub(crate) fn run_ops_install(args: &cli::OpsInstallArgs) -> Result<(String, i32), String> {
    let common = &args.common;
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let process = OpsProcess::new(common.allow_subprocess);
    let (envelope, exit_code) = bijux_atlas_ops::lifecycle::simulation::stack_install_payload(
        &process,
        &repo_root,
        bijux_atlas_ops::lifecycle::simulation::StackInstallRequest {
            ops_root: common.ops_root.clone(),
            requested_profile: common.profile.clone(),
            run_id: run_id.as_str(),
            evidence_mode: common.evidence,
            plan_mode: args.plan,
            dry_run_mode: &args.dry_run,
            enable_kind: args.kind,
            enable_apply: args.apply,
            allow_subprocess: common.allow_subprocess,
            allow_write: common.allow_write,
            allow_network: common.allow_network,
            force: common.force,
        },
    )
    .map_err(OpsCommandError::Effect)
    .map_err(|err| err.to_stable_message())?;
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, exit_code))
}
