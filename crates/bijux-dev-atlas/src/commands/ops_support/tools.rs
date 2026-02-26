// SPDX-License-Identifier: Apache-2.0

use crate::*;

pub(crate) fn normalize_tool_version_with_regex(raw: &str, pattern: &str) -> Option<String> {
    let re = Regex::new(pattern).ok()?;
    re.captures(raw)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ToolMismatchCode {
    MissingBinary,
    VersionMismatch,
}

impl ToolMismatchCode {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::MissingBinary => "TOOLS_MISSING_BINARY",
            Self::VersionMismatch => "TOOLS_VERSION_MISMATCH",
        }
    }
}

pub(crate) fn parse_tool_overrides(
    values: &[String],
) -> Result<std::collections::BTreeMap<String, String>, String> {
    let mut out = std::collections::BTreeMap::new();
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

fn walk_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if root.is_file() {
        out.push(root.to_path_buf());
        return out;
    }
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            out.extend(walk_files(&entry.path()));
        }
    }
    out
}

pub(crate) fn validate_pins_completeness(
    repo_root: &Path,
    pins: &crate::ops_command_support::StackPinsToml,
) -> Result<Vec<String>, OpsCommandError> {
    let mut errors = Vec::new();
    let stack_manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/stack/generated/version-manifest.json"))
            .map_err(|err| {
                OpsCommandError::Manifest(format!(
                    "failed to read ops/stack/generated/version-manifest.json: {err}"
                ))
            })?,
    )
    .map_err(|err| OpsCommandError::Schema(format!("invalid version manifest json: {err}")))?;
    if let Some(obj) = stack_manifest.as_object() {
        for (k, v) in obj {
            if k == "schema_version" {
                continue;
            }
            if !pins.images.contains_key(k) {
                errors.push(format!("pins missing image key `{k}`"));
            }
            if let Some(value) = v.as_str() {
                if value.contains(":latest") {
                    errors.push(format!("floating tag forbidden in stack manifest `{k}`"));
                }
            }
        }
    }
    for (k, v) in &pins.images {
        if v.contains(":latest") {
            errors.push(format!("floating tag forbidden in pins image `{k}`"));
        }
    }
    for (k, v) in &pins.charts {
        if v.contains(":latest") {
            errors.push(format!("floating tag forbidden in pins chart `{k}`"));
        }
    }
    let contracts_json: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(repo_root.join("ops/inventory/contracts.json")).map_err(
            |err| {
                OpsCommandError::Manifest(format!(
                    "failed to read ops/inventory/contracts.json: {err}"
                ))
            },
        )?,
    )
    .map_err(|err| OpsCommandError::Schema(format!("invalid contracts.json: {err}")))?;
    let contract_paths = contracts_json
        .get("contracts")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| {
            v.get("path")
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        })
        .collect::<std::collections::BTreeSet<_>>();
    for required in ["ops/inventory/tools.toml", "ops/inventory/pins.yaml"] {
        if !contract_paths.contains(required) {
            errors.push(format!(
                "contracts inventory missing required entry `{required}`"
            ));
        }
    }

    for file in [
        "ops/k8s/charts/bijux-atlas/values.yaml",
        "ops/k8s/charts/bijux-atlas/values-offline.yaml",
    ] {
        let text = std::fs::read_to_string(repo_root.join(file))
            .map_err(|err| OpsCommandError::Manifest(format!("failed to read {file}: {err}")))?;
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.contains(":latest") {
                errors.push(format!("floating latest forbidden in {file}: `{trimmed}`"));
            }
            if (trimmed.contains("image:") || trimmed.contains("repository:"))
                && trimmed.contains(':')
                && !trimmed.contains("@sha256:")
                && !trimmed.ends_with(':')
            {
                errors.push(format!(
                    "base image pin must include digest in {file}: `{trimmed}`"
                ));
            }
        }
    }

    let hardcoded_tool_patterns = ["helm ", "kubectl ", "kind ", "k6 "];
    for root in ["makefiles", ".github/workflows"] {
        let walk_root = repo_root.join(root);
        if !walk_root.exists() {
            continue;
        }
        for path in walk_files(&walk_root) {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            for pattern in hardcoded_tool_patterns {
                if text.contains(pattern) && !text.contains("bijux dev atlas") {
                    let rel = path
                        .strip_prefix(repo_root)
                        .unwrap_or(path.as_path())
                        .display()
                        .to_string();
                    errors.push(format!(
                        "hardcoded tool invocation forbidden (`{pattern}`) in {rel}"
                    ));
                }
            }
            if text.contains("kubectl apply") && !text.contains("bijux dev atlas ops k8s apply") {
                let rel = path
                    .strip_prefix(repo_root)
                    .unwrap_or(path.as_path())
                    .display()
                    .to_string();
                errors.push(format!(
                    "direct kubectl apply forbidden in {rel}; use `bijux dev atlas ops k8s apply`"
                ));
            }
        }
    }

    errors.sort();
    errors.dedup();
    Ok(errors)
}

pub(crate) fn tool_definitions_sorted(
    inventory: &ToolchainInventory,
) -> Vec<(String, ToolDefinition)> {
    inventory
        .tools
        .iter()
        .map(|(name, definition)| (name.clone(), definition.clone()))
        .collect()
}

pub(crate) fn verify_tools_snapshot(
    allow_subprocess: bool,
    inventory: &ToolchainInventory,
) -> Result<serde_json::Value, String> {
    if !allow_subprocess {
        return Ok(serde_json::json!({
            "enabled": false,
            "text": "tool verification skipped (pass --allow-subprocess)",
            "missing_required": [],
            "rows": []
        }));
    }
    let process = OpsProcess::new(true);
    let mut rows = Vec::new();
    let mut missing_required = Vec::new();
    for (name, definition) in tool_definitions_sorted(inventory) {
        let mut row = process
            .probe_tool(&name, &definition.probe_argv, &definition.version_regex)
            .map_err(|e| e.to_stable_message())?;
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

    #[test]
    fn pins_validation_rejects_latest_tag() {
        let root = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(root.path().join("ops/stack")).expect("mkdir stack");
        std::fs::create_dir_all(root.path().join("ops/stack/generated")).expect("mkdir generated");
        std::fs::create_dir_all(root.path().join("ops/k8s/charts/bijux-atlas"))
            .expect("mkdir chart");
        std::fs::create_dir_all(root.path().join("ops/inventory")).expect("mkdir inventory");
        std::fs::write(
            root.path()
                .join("ops/stack/generated/version-manifest.json"),
            "{\"schema_version\":1,\"redis\":\"redis:latest\"}",
        )
        .expect("write manifest");
        std::fs::write(
            root.path().join("ops/k8s/charts/bijux-atlas/values.yaml"),
            "image: redis:latest\n",
        )
        .expect("write values");
        std::fs::write(
            root.path()
                .join("ops/k8s/charts/bijux-atlas/values-offline.yaml"),
            "image: redis:latest\n",
        )
        .expect("write values offline");
        std::fs::write(root.path().join("ops/inventory/contracts.json"),"{\"contracts\":[{\"path\":\"ops/inventory/tools.toml\"},{\"path\":\"ops/inventory/pins.yaml\"}]}").expect("write contracts");
        let pins = crate::ops_command_support::StackPinsToml {
            charts: std::collections::BTreeMap::new(),
            images: std::collections::BTreeMap::from([(
                "redis".to_string(),
                "redis:latest".to_string(),
            )]),
            crds: std::collections::BTreeMap::new(),
        };
        let errors = validate_pins_completeness(root.path(), &pins).expect("validate");
        assert!(errors.iter().any(|e| e.contains("floating tag forbidden")));
    }
}
