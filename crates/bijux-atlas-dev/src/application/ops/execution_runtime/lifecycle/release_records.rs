// SPDX-License-Identifier: Apache-2.0
//! Release rollout observation and readiness baseline record helpers.

use crate::{OpsProcess, RunId};
use bijux_atlas_ops::lifecycle::release_observation;
pub(super) use bijux_atlas_ops::lifecycle::release_records::LifecycleSummaryUpdate;

pub(super) fn deployment_revision(
    process: &OpsProcess,
    repo_root: &std::path::Path,
    namespace: &str,
) -> Option<i64> {
    release_observation::deployment_revision(process, repo_root, namespace)
}

pub(super) fn rollout_history(
    process: &OpsProcess,
    repo_root: &std::path::Path,
    namespace: &str,
) -> serde_json::Value {
    release_observation::rollout_history(process, repo_root, namespace)
}

pub(super) fn pods_restart_count(
    process: &OpsProcess,
    repo_root: &std::path::Path,
    namespace: &str,
) -> u64 {
    release_observation::pods_restart_count(process, repo_root, namespace)
}

pub(super) fn update_lifecycle_summary(
    repo_root: &std::path::Path,
    run_id: &RunId,
    profile: &str,
    namespace: &str,
    update: LifecycleSummaryUpdate<'_>,
) -> Result<std::path::PathBuf, String> {
    bijux_atlas_ops::lifecycle::release_records::update_lifecycle_summary(
        repo_root,
        run_id.as_str(),
        profile,
        namespace,
        update,
    )
}

pub(super) fn load_readiness_baseline(
    repo_root: &std::path::Path,
    profile: &str,
) -> Result<Option<u128>, String> {
    bijux_atlas_ops::lifecycle::release_records::load_readiness_baseline(repo_root, profile)
}

pub(super) fn update_readiness_baseline(
    repo_root: &std::path::Path,
    profile: &str,
    elapsed_ms: u128,
) -> Result<std::path::PathBuf, String> {
    bijux_atlas_ops::lifecycle::release_records::update_readiness_baseline(
        repo_root, profile, elapsed_ms,
    )
}
