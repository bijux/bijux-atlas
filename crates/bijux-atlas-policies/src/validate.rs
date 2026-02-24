use crate::adapters::load_policy_set_from_workspace;
use crate::limits::{MAX_SCHEMA_BUMP_STEP, MIN_POLICY_SCHEMA_VERSION};
use crate::policy_set::{
    canonical_policy_set_json, resolve_mode_profile as resolve_mode_profile_impl,
    validate_policy_change_requires_version_bump as validate_policy_change_requires_version_bump_impl,
    validate_policy_set,
};
use crate::schema::{
    PolicyConfig, PolicyMode, PolicyModeProfile, PolicySchema, PolicySchemaVersion,
};
use serde_json::{Map, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyValidationError(pub String);

impl std::fmt::Display for PolicyValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for PolicyValidationError {}

pub fn load_policy_from_workspace(
    root: &std::path::Path,
) -> Result<PolicyConfig, PolicyValidationError> {
    load_policy_set_from_workspace(root)
}

pub fn validate_policy_config(cfg: &PolicyConfig) -> Result<(), PolicyValidationError> {
    validate_policy_set(cfg)
}

pub fn resolve_mode_profile(
    cfg: &PolicyConfig,
    mode: PolicyMode,
) -> Result<PolicyModeProfile, PolicyValidationError> {
    resolve_mode_profile_impl(cfg, mode)
}

pub fn validate_policy_change_requires_version_bump(
    old_cfg: &PolicyConfig,
    new_cfg: &PolicyConfig,
) -> Result<(), PolicyValidationError> {
    validate_policy_change_requires_version_bump_impl(old_cfg, new_cfg)
}

pub fn canonical_config_json(cfg: &PolicyConfig) -> Result<String, PolicyValidationError> {
    canonical_policy_set_json(cfg)
}

pub fn validate_schema_version_transition(
    from: &str,
    to: &str,
) -> Result<(), PolicyValidationError> {
    let from_num = from
        .parse::<u32>()
        .map_err(|_| PolicyValidationError("schema version must be numeric".to_string()))?;
    let to_num = to
        .parse::<u32>()
        .map_err(|_| PolicyValidationError("schema version must be numeric".to_string()))?;

    if from_num < MIN_POLICY_SCHEMA_VERSION || to_num < MIN_POLICY_SCHEMA_VERSION {
        return Err(PolicyValidationError(format!(
            "schema version must be >= {MIN_POLICY_SCHEMA_VERSION}"
        )));
    }

    if to_num < from_num {
        return Err(PolicyValidationError(
            "schema version must not decrease".to_string(),
        ));
    }

    if to_num.saturating_sub(from_num) > MAX_SCHEMA_BUMP_STEP {
        return Err(PolicyValidationError(format!(
            "schema version bump must be <= {MAX_SCHEMA_BUMP_STEP}"
        )));
    }

    Ok(())
}

pub(crate) fn field_path_exists(root: &Value, path: &str) -> bool {
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

pub(crate) fn decode_schema_version(schema: &Value) -> Result<PolicySchema, PolicyValidationError> {
    let root = schema
        .as_object()
        .ok_or_else(|| PolicyValidationError("policy schema must be object".to_string()))?;
    let props = root
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_else(|| PolicyValidationError("schema missing properties".to_string()))?;
    let schema_ver = props
        .get("schema_version")
        .and_then(Value::as_object)
        .and_then(|p| p.get("const"))
        .and_then(Value::as_str)
        .ok_or_else(|| {
            PolicyValidationError("schema properties.schema_version.const missing".to_string())
        })?;

    let parsed = match schema_ver {
        "1" => PolicySchemaVersion::V1,
        _ => {
            return Err(PolicyValidationError(format!(
                "unsupported policy schema version const: {schema_ver}"
            )));
        }
    };
    Ok(PolicySchema {
        schema_version: parsed,
    })
}

pub(crate) fn normalize_json(value: Value) -> Value {
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
