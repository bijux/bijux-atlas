// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(not(test), allow(dead_code))]

use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct CommandSurfaceEntry {
    pub command: String,
    pub owner: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct CommandSurfaceSnapshot {
    pub schema_version: u32,
    pub surfaces: Vec<CommandSurfaceEntry>,
}

pub(crate) fn render_surface_snapshot_json(
    entries: &[CommandSurfaceEntry],
) -> Result<String, String> {
    let mut by_command: BTreeMap<String, CommandSurfaceEntry> = BTreeMap::new();
    for entry in entries {
        by_command.insert(entry.command.clone(), entry.clone());
    }
    let snapshot = CommandSurfaceSnapshot {
        schema_version: 1,
        surfaces: by_command.into_values().collect(),
    };
    serde_json::to_string_pretty(&snapshot).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::{render_surface_snapshot_json, CommandSurfaceEntry};

    #[test]
    fn surface_snapshot_output_is_sorted_and_deduplicated_by_command() {
        let rendered_result = render_surface_snapshot_json(&[
            CommandSurfaceEntry {
                command: "ops inventory".to_string(),
                owner: "ops".to_string(),
                category: "inventory".to_string(),
            },
            CommandSurfaceEntry {
                command: "check list".to_string(),
                owner: "checks".to_string(),
                category: "governance".to_string(),
            },
            CommandSurfaceEntry {
                command: "ops inventory".to_string(),
                owner: "ops-updated".to_string(),
                category: "inventory".to_string(),
            },
        ]);
        assert!(
            rendered_result.is_ok(),
            "surface snapshot json encode failed"
        );
        let rendered = rendered_result.unwrap_or_default();
        let json_result: Result<serde_json::Value, _> = serde_json::from_str(&rendered);
        assert!(json_result.is_ok(), "json parse failed");
        let json = json_result.unwrap_or(serde_json::Value::Null);
        let surfaces = match json["surfaces"].as_array() {
            Some(surfaces) => surfaces,
            None => panic!("surfaces array missing"),
        };
        assert_eq!(surfaces[0]["command"], "check list");
        assert_eq!(surfaces[1]["command"], "ops inventory");
        assert_eq!(surfaces[1]["owner"], "ops-updated");
    }
}
