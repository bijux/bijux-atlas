// SPDX-License-Identifier: Apache-2.0
//! Release manifest source resolution and compatibility contract helpers.

use bijux_atlas_ops::lifecycle::release_contracts::{self, ReleaseChartSource};

pub(super) fn resolve_chart_source(
    repo_root: &std::path::Path,
    chart_source: crate::cli::OpsHelmChartSource,
) -> Result<std::path::PathBuf, String> {
    let source = match chart_source {
        crate::cli::OpsHelmChartSource::Current => ReleaseChartSource::Current,
        crate::cli::OpsHelmChartSource::Previous => ReleaseChartSource::Previous,
    };
    release_contracts::release_chart_source_path(repo_root, source)
}

pub(super) fn manifest_diff_summary(before: &str, after: &str) -> serde_json::Value {
    release_contracts::manifest_diff_summary(before, after)
}

pub(super) fn lifecycle_compatibility_checks(
    before_manifest: &str,
    after_manifest: &str,
) -> serde_json::Value {
    bijux_atlas_ops::lifecycle::release_contracts::lifecycle_compatibility_checks(
        before_manifest,
        after_manifest,
    )
}
