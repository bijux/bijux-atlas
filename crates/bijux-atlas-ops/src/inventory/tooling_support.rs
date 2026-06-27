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

pub trait ToolProbeRunner {
    fn probe_tool(
        &self,
        name: &str,
        probe_argv: &[String],
        version_regex: &str,
    ) -> Result<serde_json::Value, String>;
}

pub fn build_tool_probe_snapshot<R: ToolProbeRunner>(
    runner: &R,
    inventory: &ToolchainInventory,
) -> Result<serde_json::Value, String> {
    let mut rows = Vec::new();
    let mut missing_required = Vec::new();
    for (name, definition) in tool_definitions_sorted(inventory) {
        let mut row =
            runner.probe_tool(&name, &definition.probe_argv, &definition.version_regex)?;
        row["required"] = serde_json::Value::Bool(definition.required);
        if definition.required && row["installed"] != serde_json::Value::Bool(true) {
            missing_required.push(name.clone());
        }
        rows.push(row);
    }
    rows.sort_by(|a, b| a["name"].as_str().cmp(&b["name"].as_str()));
    Ok(serde_json::json!({
        "enabled": true,
        "text": if missing_required.is_empty() { "all required tools available" } else { "missing required tools" },
        "missing_required": missing_required,
        "rows": rows
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ProbeRunnerStub;

    impl ToolProbeRunner for ProbeRunnerStub {
        fn probe_tool(
            &self,
            name: &str,
            _probe_argv: &[String],
            _version_regex: &str,
        ) -> Result<serde_json::Value, String> {
            Ok(match name {
                "helm" => serde_json::json!({
                    "name": "helm",
                    "installed": true,
                    "version_raw": "v3.18.4",
                    "version": "3.18.4"
                }),
                "kubectl" => serde_json::json!({
                    "name": "kubectl",
                    "installed": false,
                    "version_raw": serde_json::Value::Null,
                    "version": serde_json::Value::Null
                }),
                other => serde_json::json!({
                    "name": other,
                    "installed": true,
                    "version_raw": "v0.0.0",
                    "version": "0.0.0"
                }),
            })
        }
    }

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

    #[test]
    fn tool_probe_snapshot_reports_missing_required_tools() {
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

        let snapshot =
            build_tool_probe_snapshot(&ProbeRunnerStub, &inventory).expect("build snapshot");

        assert_eq!(snapshot["enabled"], serde_json::Value::Bool(true));
        assert_eq!(snapshot["text"], "missing required tools");
        assert_eq!(snapshot["missing_required"], serde_json::json!(["kubectl"]));
        assert_eq!(snapshot["rows"][0]["name"], "helm");
        assert_eq!(snapshot["rows"][1]["name"], "kubectl");
        assert_eq!(
            snapshot["rows"][1]["required"],
            serde_json::Value::Bool(true)
        );
    }
}
