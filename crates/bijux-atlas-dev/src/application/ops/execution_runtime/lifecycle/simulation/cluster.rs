// SPDX-License-Identifier: Apache-2.0
//! Kind and simulation-cluster operations for install-status flows.

use super::*;
use crate::cli::OpsCommonArgs;
use crate::ops_commands::{emit_payload, run_id_or_default};
use crate::{resolve_repo_root, OpsProcess};

pub(crate) fn run_ops_observe_slo_list(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let payload = bijux_atlas_ops::observe::commands::render_slo_list_payload(&repo_root)?;
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_observe_slo_verify(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_write {
        return Err("observe slo verify requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let (payload, exit_code) =
        bijux_atlas_ops::observe::commands::verify_slo_contract(&repo_root, run_id.as_str())?;
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, exit_code))
}

pub(crate) fn run_ops_observe_alerts_verify(
    common: &OpsCommonArgs,
) -> Result<(String, i32), String> {
    if !common.allow_write {
        return Err("observe alerts verify requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let (payload, exit_code) =
        bijux_atlas_ops::observe::commands::verify_alert_contract(&repo_root, run_id.as_str())?;
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, exit_code))
}

pub(crate) fn run_ops_observe_runbooks_verify(
    common: &OpsCommonArgs,
) -> Result<(String, i32), String> {
    if !common.allow_write {
        return Err("observe runbooks verify requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let (payload, exit_code) =
        bijux_atlas_ops::observe::commands::verify_runbook_contract(&repo_root, run_id.as_str())?;
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, exit_code))
}

pub(crate) fn run_ops_observe_readiness(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_write {
        return Err("observe readiness requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let (payload, exit_code) =
        bijux_atlas_ops::observe::commands::build_operational_readiness_payload(
            &repo_root,
            run_id.as_str(),
        )?;
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, exit_code))
}

pub(crate) fn run_ops_obs_verify(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_subprocess {
        return Err("obs verify requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("obs verify requires --allow-write".to_string());
    }
    if !common.allow_network {
        return Err("obs verify requires --allow-network".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let profile = common
        .profile
        .clone()
        .unwrap_or_else(|| "profile-baseline".to_string());
    let namespace = bijux_atlas_ops::workspace::profiles::simulation_namespace(&profile, None);
    let (payload, exit_code) = bijux_atlas_ops::observe::commands::verify_observability_runtime(
        &repo_root,
        run_id.as_str(),
        &profile,
        &namespace,
    )?;
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, exit_code))
}

pub(crate) fn run_ops_drill(args: &crate::cli::OpsDrillRunArgs) -> Result<(String, i32), String> {
    let common = &args.common;
    if !common.allow_write {
        return Err("drills run requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let (payload, exit_code) = bijux_atlas_ops::lifecycle::simulation::drill_contract_payload(
        &repo_root,
        run_id.as_str(),
        &args.name,
    )?;
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, exit_code))
}

pub(crate) fn run_ops_kind_up(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_subprocess {
        return Err("kind up requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("kind up requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    let run_id = run_id_or_default(common.run_id.clone())?;
    let (envelope, exit_code) = bijux_atlas_ops::lifecycle::simulation::kind_up_payload(
        &process,
        &repo_root,
        run_id.as_str(),
    )?;
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, exit_code))
}

pub(crate) fn run_ops_kind_down(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_subprocess {
        return Err("kind down requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("kind down requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    let run_id = run_id_or_default(common.run_id.clone())?;
    let (envelope, exit_code) = bijux_atlas_ops::lifecycle::simulation::kind_down_payload(
        &process,
        &repo_root,
        run_id.as_str(),
    )?;
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, exit_code))
}

pub(crate) fn run_ops_kind_status(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_subprocess {
        return Err("kind status requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("kind status requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    let run_id = run_id_or_default(common.run_id.clone())?;
    let (envelope, exit_code) = bijux_atlas_ops::lifecycle::simulation::kind_status_payload(
        &process,
        &repo_root,
        run_id.as_str(),
    )?;
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, exit_code))
}

pub(crate) fn run_ops_kind_preload(
    args: &crate::cli::OpsKindPreloadArgs,
) -> Result<(String, i32), String> {
    let common = &args.common;
    if !common.allow_subprocess {
        return Err("kind preload-image requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("kind preload-image requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let process = OpsProcess::new(true);
    let run_id = run_id_or_default(common.run_id.clone())?;
    let (envelope, exit_code) = bijux_atlas_ops::lifecycle::simulation::kind_preload_payload(
        &process,
        &repo_root,
        run_id.as_str(),
        &args.image,
    )?;
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, exit_code))
}

pub(crate) fn run_ops_helm_install(
    args: &crate::cli::OpsHelmInstallArgs,
) -> Result<(String, i32), String> {
    let common = &args.release.common;
    match args.release.cluster {
        crate::cli::OpsClusterTarget::Kind => {}
    }
    if !common.allow_subprocess {
        return Err("helm install requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("helm install requires --allow-write".to_string());
    }
    if !common.allow_network {
        return Err("helm install requires --allow-network".to_string());
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
    let chart_source = match args.chart_source {
        crate::cli::OpsHelmChartSource::Current => {
            bijux_atlas_ops::lifecycle::release::contracts::ReleaseChartSource::Current
        }
        crate::cli::OpsHelmChartSource::Previous => {
            bijux_atlas_ops::lifecycle::release::contracts::ReleaseChartSource::Previous
        }
    };
    let (envelope, exit_code) = bijux_atlas_ops::lifecycle::simulation::helm_install_payload(
        &process,
        &repo_root,
        run_id.as_str(),
        &profile,
        &namespace,
        chart_source,
        args.release.timeout_seconds,
    )?;
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, exit_code))
}

pub(crate) fn run_ops_helm_uninstall(
    args: &crate::cli::OpsHelmReleaseArgs,
) -> Result<(String, i32), String> {
    let common = &args.common;
    match args.cluster {
        crate::cli::OpsClusterTarget::Kind => {}
    }
    if !common.allow_subprocess {
        return Err("helm uninstall requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("helm uninstall requires --allow-write".to_string());
    }
    if !common.allow_network {
        return Err("helm uninstall requires --allow-network".to_string());
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
    let (envelope, exit_code) = bijux_atlas_ops::lifecycle::simulation::helm_uninstall_payload(
        &process,
        &repo_root,
        run_id.as_str(),
        &profile,
        &namespace,
    )?;
    let rendered = emit_payload(common.format, common.out.clone(), &envelope)?;
    Ok((rendered, exit_code))
}
