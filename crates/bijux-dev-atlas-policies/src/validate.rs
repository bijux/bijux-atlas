// SPDX-License-Identifier: Apache-2.0

use crate::schema::{DevAtlasPolicySetDocument, PolicySchemaVersion, POLICY_REGISTRY};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

const POLICY_CONFIG_PATH: &str = "ops/inventory/policies/dev-atlas-policy.json";
const POLICY_SCHEMA_PATH: &str = "ops/inventory/policies/dev-atlas-policy.schema.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyValidationError(pub String);

impl std::fmt::Display for PolicyValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for PolicyValidationError {}

#[must_use]
pub fn policy_config_path(root: &Path) -> PathBuf {
    root.join(POLICY_CONFIG_PATH)
}

#[must_use]
pub fn policy_schema_path(root: &Path) -> PathBuf {
    root.join(POLICY_SCHEMA_PATH)
}

pub fn load_policy_from_workspace(
    root: &Path,
) -> Result<DevAtlasPolicySetDocument, PolicyValidationError> {
    let config_raw = fs::read_to_string(policy_config_path(root))
        .map_err(|e| PolicyValidationError(format!("read dev policy config failed: {e}")))?;
    let schema_raw = fs::read_to_string(policy_schema_path(root))
        .map_err(|e| PolicyValidationError(format!("read dev policy schema failed: {e}")))?;

    let config_val: Value = serde_json::from_str(&config_raw)
        .map_err(|e| PolicyValidationError(format!("parse dev policy config failed: {e}")))?;
    let schema_val: Value = serde_json::from_str(&schema_raw)
        .map_err(|e| PolicyValidationError(format!("parse dev policy schema failed: {e}")))?;

    validate_schema_version_match(&config_val, &schema_val)?;
    validate_documented_defaults(&config_val)?;
    validate_policy_registry_ids(&config_val)?;

    serde_json::from_value(config_val)
        .map_err(|e| PolicyValidationError(format!("decode dev policy config failed: {e}")))
}

pub fn validate_relaxation_expiry(
    config: &Value,
    today_ymd: &str,
) -> Result<(), PolicyValidationError> {
    let Some(relaxations) = config.get("relaxations").and_then(Value::as_array) else {
        return Ok(());
    };
    for item in relaxations {
        let policy_id = item
            .get("policy_id")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                PolicyValidationError("relaxations.policy_id must be string".to_string())
            })?;
        let expires_on = item
            .get("expires_on")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                PolicyValidationError("relaxations.expires_on must be string".to_string())
            })?;
        if !is_ymd(expires_on) {
            return Err(PolicyValidationError(format!(
                "relaxation for `{policy_id}` has invalid expires_on `{expires_on}`"
            )));
        }
        if expires_on < today_ymd {
            return Err(PolicyValidationError(format!(
                "relaxation for `{policy_id}` expired on `{expires_on}`"
            )));
        }
    }
    Ok(())
}

pub fn validate_policy_registry_ids(config: &Value) -> Result<(), PolicyValidationError> {
    let registered = POLICY_REGISTRY
        .iter()
        .map(|p| p.id)
        .collect::<std::collections::BTreeSet<_>>();
    if let Some(defaults) = config.get("documented_defaults").and_then(Value::as_array) {
        for item in defaults {
            let field = item.get("field").and_then(Value::as_str).ok_or_else(|| {
                PolicyValidationError("documented_defaults.field must be string".to_string())
            })?;
            if !registered.contains(field) {
                return Err(PolicyValidationError(format!(
                    "documented default references unknown policy id `{field}`"
                )));
            }
        }
    }
    if let Some(ratchets) = config.get("ratchets").and_then(Value::as_array) {
        for item in ratchets {
            let id = item
                .get("id")
                .and_then(Value::as_str)
                .ok_or_else(|| PolicyValidationError("ratchets.id must be string".to_string()))?;
            if !registered.contains(id) {
                return Err(PolicyValidationError(format!(
                    "ratchet references unknown policy id `{id}`"
                )));
            }
        }
    }
    if let Some(relaxations) = config.get("relaxations").and_then(Value::as_array) {
        for item in relaxations {
            let policy_id = item
                .get("policy_id")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    PolicyValidationError("relaxations.policy_id must be string".to_string())
                })?;
            if !registered.contains(policy_id) {
                return Err(PolicyValidationError(format!(
                    "relaxation references unknown policy id `{policy_id}`"
                )));
            }
        }
    }
    Ok(())
}

fn is_ymd(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    bytes
        .iter()
        .enumerate()
        .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
}

pub fn canonical_policy_json(
    policy: &DevAtlasPolicySetDocument,
) -> Result<String, PolicyValidationError> {
    let value = serde_json::to_value(policy)
        .map_err(|e| PolicyValidationError(format!("encode dev policy failed: {e}")))?;
    let normalized = normalize_json(value);
    serde_json::to_string_pretty(&normalized)
        .map_err(|e| PolicyValidationError(format!("print dev policy failed: {e}")))
}

pub fn validate_policy_change_requires_version_bump(
    old_cfg: &DevAtlasPolicySetDocument,
    new_cfg: &DevAtlasPolicySetDocument,
) -> Result<(), PolicyValidationError> {
    let old_json = canonical_policy_json(old_cfg)?;
    let new_json = canonical_policy_json(new_cfg)?;
    if old_json != new_json && old_cfg.schema_version == new_cfg.schema_version {
        return Err(PolicyValidationError(
            "dev policy content changed without schema_version bump".to_string(),
        ));
    }
    Ok(())
}

fn validate_schema_version_match(
    config: &Value,
    schema: &Value,
) -> Result<(), PolicyValidationError> {
    let cfg_ver = config
        .get("schema_version")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            PolicyValidationError("schema_version is required in dev policy".to_string())
        })?;

    let schema_ver = schema
        .as_object()
        .and_then(|root| root.get("properties"))
        .and_then(Value::as_object)
        .and_then(|props| props.get("schema_version"))
        .and_then(Value::as_object)
        .and_then(|field| field.get("const"))
        .and_then(Value::as_str)
        .ok_or_else(|| {
            PolicyValidationError(
                "schema properties.schema_version.const missing in dev policy schema".to_string(),
            )
        })?;

    let expected = match schema_ver {
        "1" => PolicySchemaVersion::V1,
        _ => {
            return Err(PolicyValidationError(format!(
                "unsupported dev policy schema version const: {schema_ver}"
            )))
        }
    };

    if cfg_ver != expected.as_str() {
        return Err(PolicyValidationError(format!(
            "dev policy schema_version mismatch: config={cfg_ver} schema={schema_ver}"
        )));
    }

    Ok(())
}

fn validate_documented_defaults(config: &Value) -> Result<(), PolicyValidationError> {
    let defaults = config
        .get("documented_defaults")
        .and_then(Value::as_array)
        .ok_or_else(|| PolicyValidationError("documented_defaults must be an array".to_string()))?;

    let mut seen = std::collections::BTreeSet::<String>::new();
    for item in defaults {
        let field = item.get("field").and_then(Value::as_str).ok_or_else(|| {
            PolicyValidationError("documented_defaults.field must be string".to_string())
        })?;
        let reason = item.get("reason").and_then(Value::as_str).ok_or_else(|| {
            PolicyValidationError("documented_defaults.reason must be string".to_string())
        })?;

        if field.trim().is_empty() || reason.trim().is_empty() {
            return Err(PolicyValidationError(
                "documented_defaults.field/reason must be non-empty".to_string(),
            ));
        }
        if !seen.insert(field.to_string()) {
            return Err(PolicyValidationError(format!(
                "documented_defaults.field duplicated: {field}"
            )));
        }
        if !field_path_exists(config, field) {
            return Err(PolicyValidationError(format!(
                "documented_defaults.field does not exist in dev policy: {field}"
            )));
        }
    }

    Ok(())
}

fn field_path_exists(root: &Value, path: &str) -> bool {
    let mut cur = root;
    for seg in path.split('.') {
        if seg.is_empty() {
            return false;
        }
        cur = match cur {
            Value::Object(map) => match map.get(seg) {
                Some(v) => v,
                None => return false,
            },
            _ => return false,
        };
    }
    true
}

fn normalize_json(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut entries: Vec<(String, Value)> = map
                .into_iter()
                .map(|(k, v)| (k, normalize_json(v)))
                .collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            let mut out = Map::new();
            for (k, v) in entries {
                out.insert(k, v);
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(normalize_json).collect()),
        other => other,
    }
}
