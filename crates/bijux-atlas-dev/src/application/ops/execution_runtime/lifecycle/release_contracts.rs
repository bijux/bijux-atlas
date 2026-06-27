// SPDX-License-Identifier: Apache-2.0
//! Release manifest source resolution and compatibility contract helpers.

use super::{simulation_current_chart_path, simulation_previous_chart_path};
use crate::ops_commands::sha256_hex;

pub(super) fn resolve_chart_source(
    repo_root: &std::path::Path,
    chart_source: crate::cli::OpsHelmChartSource,
) -> Result<std::path::PathBuf, String> {
    let path = match chart_source {
        crate::cli::OpsHelmChartSource::Current => simulation_current_chart_path(repo_root),
        crate::cli::OpsHelmChartSource::Previous => simulation_previous_chart_path(repo_root),
    };
    if path.exists() {
        Ok(path)
    } else {
        Err(format!(
            "missing chart source {}; current uses the working tree chart and previous uses artifacts/ops/chart-sources/previous/bijux-atlas.tgz",
            path.display()
        ))
    }
}

pub(super) fn manifest_diff_summary(before: &str, after: &str) -> serde_json::Value {
    let before_lines = before.lines().collect::<Vec<_>>();
    let after_lines = after.lines().collect::<Vec<_>>();
    let shared = before_lines.len().min(after_lines.len());
    let changed_lines = (0..shared)
        .filter(|index| before_lines[*index] != after_lines[*index])
        .count()
        + before_lines.len().saturating_sub(shared)
        + after_lines.len().saturating_sub(shared);
    serde_json::json!({
        "before_sha256": sha256_hex(before),
        "after_sha256": sha256_hex(after),
        "before_lines": before_lines.len(),
        "after_lines": after_lines.len(),
        "changed_lines": changed_lines
    })
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
