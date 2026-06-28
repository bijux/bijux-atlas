// SPDX-License-Identifier: Apache-2.0

use crate::cli::OpsCommonArgs;
use crate::ops_commands::{emit_payload, load_profiles, resolve_profile, run_id_or_default};
use crate::ops_support::resolve_ops_root;
use crate::{resolve_repo_root, OpsProcess};
use bijux_atlas_ops::kubernetes::commands::{
    k8s_apply_command_payload, k8s_conformance_command_payload, k8s_logs_command_payload,
    k8s_plan_command_payload, k8s_port_forward_command_payload, k8s_ports_command_payload,
    k8s_wait_command_payload,
};
use serde_json::Value;
use std::fs;

use super::resolve_render_inputs;

pub(crate) fn run_ops_k8s_plan(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let (render_path, index_path) = resolve_render_inputs(&repo_root, &run_id, &profile.name)
        .map_err(|e| e.to_stable_message())?;
    let index_json: Value = serde_json::from_str(
        &fs::read_to_string(&index_path)
            .map_err(|err| format!("failed to read {}: {err}", index_path.display()))?,
    )
    .map_err(|err| format!("failed to parse {}: {err}", index_path.display()))?;
    let payload = k8s_plan_command_payload(
        &profile.name,
        run_id.as_str(),
        &render_path,
        &index_path,
        index_json,
    );
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_k8s_apply(
    args: &crate::cli::OpsK8sApplyArgs,
    dry_run: bool,
) -> Result<(String, i32), String> {
    let common = &args.common;
    if !args.apply && !dry_run {
        return Err("OPS_USAGE_ERROR: k8s apply requires explicit --apply".to_string());
    }
    if !common.allow_subprocess {
        return Err("OPS_EFFECT_ERROR: k8s apply requires --allow-subprocess".to_string());
    }
    if !common.allow_write {
        return Err("OPS_EFFECT_ERROR: k8s apply requires --allow-write".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let (render_path, _) = resolve_render_inputs(&repo_root, &run_id, &profile.name)
        .map_err(|e| e.to_stable_message())?;
    let process = OpsProcess::new(true);
    let payload = k8s_apply_command_payload(
        &process,
        &repo_root,
        &profile.name,
        &profile.kind_profile,
        run_id.as_str(),
        &render_path,
        dry_run,
        common.force,
    )?;
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_k8s_conformance(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_subprocess {
        return Err("OPS_EFFECT_ERROR: k8s conformance requires --allow-subprocess".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let run_id = run_id_or_default(common.run_id.clone())?;
    let process = OpsProcess::new(true);
    let (payload, exit_code) = k8s_conformance_command_payload(
        &process,
        &repo_root,
        &profile.kind_profile,
        run_id.as_str(),
        common.allow_write,
        common.force,
    )?;
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, exit_code))
}

pub(crate) fn run_ops_k8s_ports(common: &OpsCommonArgs) -> Result<(String, i32), String> {
    if !common.allow_subprocess {
        return Err("OPS_EFFECT_ERROR: k8s ports requires --allow-subprocess".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let process = OpsProcess::new(true);
    let payload =
        k8s_ports_command_payload(&process, &repo_root, &profile.kind_profile, common.force)?;
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_k8s_wait(args: &crate::cli::OpsK8sWaitArgs) -> Result<(String, i32), String> {
    let common = &args.common;
    if !common.allow_subprocess {
        return Err("OPS_EFFECT_ERROR: k8s wait requires --allow-subprocess".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let process = OpsProcess::new(true);
    let (payload, exit_code) = k8s_wait_command_payload(
        &process,
        &repo_root,
        &profile.kind_profile,
        args.timeout_seconds,
        common.fail_fast,
        common.force,
    )?;
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, exit_code))
}

pub(crate) fn run_ops_k8s_logs(args: &crate::cli::OpsK8sLogsArgs) -> Result<(String, i32), String> {
    let common = &args.common;
    if !common.allow_subprocess {
        return Err("OPS_EFFECT_ERROR: k8s logs requires --allow-subprocess".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let process = OpsProcess::new(true);
    let pod = args
        .pod
        .clone()
        .unwrap_or_else(|| "deployment/bijux-atlas".to_string());
    let payload = k8s_logs_command_payload(
        &process,
        &repo_root,
        &profile.kind_profile,
        &pod,
        args.tail,
        common.force,
    )?;
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}

pub(crate) fn run_ops_k8s_port_forward(
    args: &crate::cli::OpsK8sPortForwardArgs,
) -> Result<(String, i32), String> {
    let common = &args.common;
    if !common.allow_subprocess {
        return Err("OPS_EFFECT_ERROR: k8s port-forward requires --allow-subprocess".to_string());
    }
    if !common.allow_network {
        return Err("OPS_EFFECT_ERROR: k8s port-forward requires --allow-network".to_string());
    }
    let repo_root = resolve_repo_root(common.repo_root.clone())?;
    let ops_root =
        resolve_ops_root(&repo_root, common.ops_root.clone()).map_err(|e| e.to_stable_message())?;
    let mut profiles = load_profiles(&ops_root).map_err(|e| e.to_stable_message())?;
    profiles.sort_by(|a, b| a.name.cmp(&b.name));
    let profile =
        resolve_profile(common.profile.clone(), &profiles).map_err(|e| e.to_stable_message())?;
    let process = OpsProcess::new(true);
    let payload = k8s_port_forward_command_payload(
        &process,
        &repo_root,
        &profile.kind_profile,
        &args.resource,
        args.local_port,
        args.remote_port,
        common.force,
    )?;
    let rendered = emit_payload(common.format, common.out.clone(), &payload)?;
    Ok((rendered, 0))
}
