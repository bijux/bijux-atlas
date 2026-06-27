// SPDX-License-Identifier: Apache-2.0
//! RunId-aware adapters over atlas-ops simulation record contracts.

use crate::RunId;
pub(super) use bijux_atlas_ops::lifecycle::simulation_records::{
    drill_check_paths, SimulationSummaryUpdate,
};

pub(super) fn update_simulation_summary(
    repo_root: &std::path::Path,
    run_id: &RunId,
    profile: &str,
    namespace: &str,
    update: SimulationSummaryUpdate<'_>,
) -> Result<std::path::PathBuf, String> {
    bijux_atlas_ops::lifecycle::simulation_records::update_simulation_summary(
        repo_root,
        run_id.as_str(),
        profile,
        namespace,
        update,
    )
}
