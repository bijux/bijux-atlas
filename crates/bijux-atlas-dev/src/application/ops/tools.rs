// SPDX-License-Identifier: Apache-2.0

use crate::*;
pub(crate) use bijux_atlas_ops::inventory::tooling_support::{
    build_tool_probe_snapshot, normalize_tool_version_with_regex, parse_tool_overrides,
    tool_probe_skipped_snapshot, ToolMismatchCode,
};

pub(crate) fn verify_tools_snapshot(
    allow_subprocess: bool,
    inventory: &ToolchainInventory,
) -> Result<serde_json::Value, String> {
    if !allow_subprocess {
        return Ok(tool_probe_skipped_snapshot());
    }
    build_tool_probe_snapshot(&OpsProcess::new(true), inventory)
}
