use crate::evaluation::{evaluate_policy_set, PolicySeverity};
use crate::schema::{PolicyConfig, PolicyMode, PolicyModeProfile, PolicySchema, PolicySet};
use crate::validate::{
    canonical_config_json, decode_schema_version, field_path_exists, normalize_json,
    validate_schema_version_transition, PolicyValidationError,
};
use serde_json::Value;

pub fn parse_policy_set_json(config_raw: &str, schema_raw: &str) -> Result<PolicySet, PolicyValidationError> {
    let config_val: Value = serde_json::from_str(config_raw)
        .map_err(|e| PolicyValidationError(format!("parse policy config failed: {e}")))?;
    let schema_val: Value = serde_json::from_str(schema_raw)
        .map_err(|e| PolicyValidationError(format!("parse policy schema failed: {e}")))?;

    validate_defaults_policy(&config_val)?;

    let cfg: PolicyConfig = serde_json::from_value(config_val)
        .map_err(|e| PolicyValidationError(format!("decode policy config failed: {e}")))?;
    let schema: PolicySchema = decode_schema_version(&schema_val)?;

    validate_policy_set(&cfg)?;
    validate_schema_version_transition(schema.schema_version.as_str(), cfg.schema_version.as_str())?;

    Ok(cfg)
}

pub fn validate_policy_set(cfg: &PolicySet) -> Result<(), PolicyValidationError> {
    let violations = evaluate_policy_set(cfg);
    if let Some(v) = violations
        .into_iter()
        .find(|v| matches!(v.severity, PolicySeverity::Error))
    {
        return Err(PolicyValidationError(format!("{}: {} ({})", v.id, v.message, v.evidence)));
    }

    validate_documented_defaults_on_config(cfg)
}

pub fn resolve_mode_profile(
    cfg: &PolicySet,
    mode: PolicyMode,
) -> Result<PolicyModeProfile, PolicyValidationError> {
    let p = match mode {
        PolicyMode::Strict => cfg.modes.strict.clone(),
        PolicyMode::Compat => cfg.modes.compat.clone(),
        PolicyMode::Dev => cfg.modes.dev.clone(),
    };
    if p.max_page_size == 0 || p.max_region_span == 0 || p.max_response_bytes == 0 {
        return Err(PolicyValidationError(format!(
            "mode {} has zero cap values",
            mode.as_str()
        )));
    }
    Ok(p)
}

pub fn validate_policy_change_requires_version_bump(
    old_cfg: &PolicySet,
    new_cfg: &PolicySet,
) -> Result<(), PolicyValidationError> {
    let old_json = canonical_config_json(old_cfg)?;
    let new_json = canonical_config_json(new_cfg)?;
    if old_json != new_json && old_cfg.schema_version == new_cfg.schema_version {
        return Err(PolicyValidationError(
            "policy content changed without schema_version bump".to_string(),
        ));
    }
    Ok(())
}

fn validate_defaults_policy(value: &Value) -> Result<(), PolicyValidationError> {
    let obj = value
        .as_object()
        .ok_or_else(|| PolicyValidationError("policy config must be object".to_string()))?;
    let defaults = obj
        .get("documented_defaults")
        .ok_or_else(|| PolicyValidationError("documented_defaults is required".to_string()))?;
    let arr = defaults
        .as_array()
        .ok_or_else(|| PolicyValidationError("documented_defaults must be an array".to_string()))?;

    let mut seen = std::collections::BTreeSet::<String>::new();
    for item in arr {
        let obj = item.as_object().ok_or_else(|| {
            PolicyValidationError("documented_defaults entries must be objects".to_string())
        })?;
        let field = obj.get("field").and_then(Value::as_str).ok_or_else(|| {
            PolicyValidationError("documented_defaults.field must be a string".to_string())
        })?;
        let reason = obj.get("reason").and_then(Value::as_str).ok_or_else(|| {
            PolicyValidationError("documented_defaults.reason must be a string".to_string())
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
        if field == "documented_defaults" || field.starts_with("documented_defaults.") {
            return Err(PolicyValidationError(
                "documented_defaults entries cannot describe documented_defaults itself"
                    .to_string(),
            ));
        }
        if !field_path_exists(value, field) {
            return Err(PolicyValidationError(format!(
                "documented_defaults.field does not exist in policy: {field}"
            )));
        }
    }

    Ok(())
}

fn validate_documented_defaults_on_config(cfg: &PolicySet) -> Result<(), PolicyValidationError> {
    let root = serde_json::to_value(cfg)
        .map_err(|e| PolicyValidationError(format!("encode config failed: {e}")))?;
    let mut seen = std::collections::BTreeSet::<String>::new();
    for item in &cfg.documented_defaults {
        let field = item.field.trim();
        let reason = item.reason.trim();
        if field.is_empty() || reason.is_empty() {
            return Err(PolicyValidationError(
                "documented_defaults.field/reason must be non-empty".to_string(),
            ));
        }
        if !seen.insert(field.to_string()) {
            return Err(PolicyValidationError(format!(
                "documented_defaults.field duplicated: {field}"
            )));
        }
        if field == "documented_defaults" || field.starts_with("documented_defaults.") {
            return Err(PolicyValidationError(
                "documented_defaults entries cannot describe documented_defaults itself"
                    .to_string(),
            ));
        }
        if !field_path_exists(&root, field) {
            return Err(PolicyValidationError(format!(
                "documented_defaults.field does not exist in policy: {field}"
            )));
        }
    }
    Ok(())
}

#[must_use]
pub fn canonical_policy_set_json(cfg: &PolicySet) -> Result<String, PolicyValidationError> {
    let value = serde_json::to_value(cfg)
        .map_err(|e| PolicyValidationError(format!("encode config failed: {e}")))?;
    let normalized = normalize_json(value);
    serde_json::to_string_pretty(&normalized)
        .map_err(|e| PolicyValidationError(format!("print config failed: {e}")) )
}
