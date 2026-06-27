// SPDX-License-Identifier: Apache-2.0

use regex::Regex;
use std::collections::BTreeMap;

use super::toolchain::{ToolDefinition, ToolchainInventory};

pub fn normalize_tool_version_with_regex(raw: &str, pattern: &str) -> Option<String> {
    let re = Regex::new(pattern).ok()?;
    re.captures(raw)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolMismatchCode {
    MissingBinary,
    VersionMismatch,
}

impl ToolMismatchCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingBinary => "TOOLS_MISSING_BINARY",
            Self::VersionMismatch => "TOOLS_VERSION_MISMATCH",
        }
    }
}

pub fn parse_tool_overrides(values: &[String]) -> Result<BTreeMap<String, String>, String> {
    let mut out = BTreeMap::new();
    for raw in values {
        let Some((name, path)) = raw.split_once('=') else {
            return Err(format!(
                "invalid --tool override `{raw}`; expected name=path"
            ));
        };
        let name = name.trim();
        let path = path.trim();
        if name.is_empty() || path.is_empty() {
            return Err(format!(
                "invalid --tool override `{raw}`; expected name=path"
            ));
        }
        out.insert(name.to_string(), path.to_string());
    }
    Ok(out)
}

#[must_use]
pub fn tool_definitions_sorted(inventory: &ToolchainInventory) -> Vec<(String, ToolDefinition)> {
    inventory
        .tools
        .iter()
        .map(|(name, definition)| (name.clone(), definition.clone()))
        .collect()
}

#[must_use]
pub fn tool_probe_skipped_snapshot() -> serde_json::Value {
    serde_json::json!({
        "enabled": false,
        "text": "tool verification skipped (pass --allow-subprocess)",
        "missing_required": [],
        "rows": []
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_tool_version_extracts_captured_semver() {
        let version =
            normalize_tool_version_with_regex("helm version v3.18.4", r"v(\d+\.\d+\.\d+)");
        assert_eq!(version.as_deref(), Some("3.18.4"));
    }

    #[test]
    fn mismatch_codes_stay_stable() {
        assert_eq!(
            ToolMismatchCode::MissingBinary.as_str(),
            "TOOLS_MISSING_BINARY"
        );
        assert_eq!(
            ToolMismatchCode::VersionMismatch.as_str(),
            "TOOLS_VERSION_MISMATCH"
        );
    }

    #[test]
    fn tool_override_parser_rejects_malformed_entries() {
        let error = parse_tool_overrides(&["helm".to_string()]).expect_err("invalid override");
        assert_eq!(error, "invalid --tool override `helm`; expected name=path");
    }

    #[test]
    fn tool_override_parser_preserves_stable_pairs() {
        let overrides = parse_tool_overrides(&[
            "helm=/opt/bin/helm".to_string(),
            "kubectl=/opt/bin/kubectl".to_string(),
        ])
        .expect("parse overrides");
        assert_eq!(overrides["helm"], "/opt/bin/helm");
        assert_eq!(overrides["kubectl"], "/opt/bin/kubectl");
    }

    #[test]
    fn tool_definition_sorting_preserves_inventory_members() {
        let inventory = ToolchainInventory {
            tools: BTreeMap::from([
                (
                    "kubectl".to_string(),
                    ToolDefinition {
                        required: true,
                        version_regex: "(.*)".to_string(),
                        probe_argv: vec!["version".to_string()],
                    },
                ),
                (
                    "helm".to_string(),
                    ToolDefinition {
                        required: false,
                        version_regex: "(.*)".to_string(),
                        probe_argv: vec!["version".to_string()],
                    },
                ),
            ]),
        };

        let names = tool_definitions_sorted(&inventory)
            .into_iter()
            .map(|(name, _)| name)
            .collect::<Vec<_>>();

        assert_eq!(names, vec!["helm".to_string(), "kubectl".to_string()]);
    }

    #[test]
    fn skipped_probe_snapshot_stays_stable() {
        let snapshot = tool_probe_skipped_snapshot();
        assert_eq!(snapshot["enabled"], serde_json::Value::Bool(false));
        assert_eq!(
            snapshot["text"],
            serde_json::Value::String(
                "tool verification skipped (pass --allow-subprocess)".to_string()
            )
        );
        assert_eq!(snapshot["rows"], serde_json::json!([]));
    }
}
