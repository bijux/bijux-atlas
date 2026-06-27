// SPDX-License-Identifier: Apache-2.0

use super::*;

fn expected_kind_context(profile: &StackProfile) -> String {
    format!("kind-{}", profile.kind_profile)
}

fn ensure_kind_context(
    process: &OpsProcess,
    profile: &StackProfile,
    force: bool,
) -> Result<(), OpsCommandError> {
    let args = vec!["config".to_string(), "current-context".to_string()];
    let (stdout, _) = process.run_subprocess("kubectl", &args, Path::new("."))?;
    let current = stdout.trim();
    let expected = expected_kind_context(profile);
    if is_context_allowed(&expected, current, force) {
        Ok(())
    } else {
        Err(OpsCommandError::Effect(format!(
            "kubectl context guard failed: expected `{expected}` got `{current}`; pass --force to override"
        )))
    }
}

pub(super) fn is_context_allowed(expected: &str, current: &str, force: bool) -> bool {
    current == expected || force
}

pub(super) fn ensure_namespace_exists(
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

pub(super) fn ensure_k8s_safety(
    process: &OpsProcess,
    repo_root: &Path,
    profile: &StackProfile,
    force: bool,
    namespace: &str,
) -> Result<(), OpsCommandError> {
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
            OpsCommandError::Effect(format!(
                "namespace guard failed for `{namespace}`: {}",
                e.to_stable_message()
            ))
        })
}
