// SPDX-License-Identifier: Apache-2.0

use super::*;
use bijux_atlas_ops::kubernetes::safety_policy::{
    expected_kind_context as expected_kind_context_for_profile, ClusterSafetyPolicy,
};

pub(crate) fn expected_kind_context(profile: &StackProfile) -> String {
    expected_kind_context_for_profile(&profile.kind_profile)
}

pub(crate) fn ensure_kind_context(
    process: &OpsProcess,
    profile: &StackProfile,
    force: bool,
) -> Result<(), OpsCommandError> {
    let args = vec!["config".to_string(), "current-context".to_string()];
    let (stdout, _) = process.run_subprocess("kubectl", &args, Path::new("."))?;
    let current = stdout.trim();
    let policy = ClusterSafetyPolicy::for_kind_profile(&profile.kind_profile, "bijux-atlas");
    if policy.allows_context(current, force) {
        Ok(())
    } else {
        Err(OpsCommandError::Effect(
            policy.context_guard_message(current),
        ))
    }
}

pub(crate) fn ensure_namespace_exists(
    process: &OpsProcess,
    namespace: &str,
    dry_run: &str,
) -> Result<(), OpsCommandError> {
    let get_args = vec![
        "get".to_string(),
        "namespace".to_string(),
        namespace.to_string(),
        "-o".to_string(),
        "name".to_string(),
    ];
    if process
        .run_subprocess("kubectl", &get_args, Path::new("."))
        .is_ok()
    {
        return Ok(());
    }
    let mut create_args = vec![
        "create".to_string(),
        "namespace".to_string(),
        namespace.to_string(),
    ];
    if dry_run == "client" {
        create_args.push("--dry-run=client".to_string());
    }
    let _ = process.run_subprocess("kubectl", &create_args, Path::new("."))?;
    Ok(())
}

pub(crate) fn ensure_k8s_safety(
    process: &OpsProcess,
    repo_root: &Path,
    profile: &StackProfile,
    force: bool,
    namespace: &str,
) -> Result<(), OpsCommandError> {
    let policy = ClusterSafetyPolicy::for_kind_profile(&profile.kind_profile, namespace);
    ensure_kind_context(process, profile, force)?;
    let args = vec![
        "get".to_string(),
        "namespace".to_string(),
        namespace.to_string(),
        "-o".to_string(),
        "name".to_string(),
    ];
    process
        .run_subprocess("kubectl", &args, repo_root)
        .map(|_| ())
        .map_err(|e| {
            OpsCommandError::Effect(policy.namespace_guard_message(&e.to_stable_message()))
        })
}
