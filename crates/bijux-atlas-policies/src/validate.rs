use crate::limits::{MAX_SCHEMA_BUMP_STEP, MIN_POLICY_SCHEMA_VERSION};
use crate::schema::{PolicyConfig, PolicySchema, PolicySchemaVersion};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

const POLICY_CONFIG_PATH: &str = "configs/policy/policy.json";
const POLICY_SCHEMA_PATH: &str = "configs/policy/policy.schema.json";

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

pub fn load_policy_from_workspace(root: &Path) -> Result<PolicyConfig, PolicyValidationError> {
    let config_raw = fs::read_to_string(policy_config_path(root))
        .map_err(|e| PolicyValidationError(format!("read policy config failed: {e}")))?;
    let schema_raw = fs::read_to_string(policy_schema_path(root))
        .map_err(|e| PolicyValidationError(format!("read policy schema failed: {e}")))?;

    let config_val: Value = serde_json::from_str(&config_raw)
        .map_err(|e| PolicyValidationError(format!("parse policy config failed: {e}")))?;
    let schema_val: Value = serde_json::from_str(&schema_raw)
        .map_err(|e| PolicyValidationError(format!("parse policy schema failed: {e}")))?;

    validate_strict_unknown_keys(&config_val)?;
    validate_defaults_policy(&config_val)?;

    let cfg: PolicyConfig = serde_json::from_value(config_val)
        .map_err(|e| PolicyValidationError(format!("decode policy config failed: {e}")))?;
    let schema: PolicySchema = decode_schema_version(&schema_val)?;

    validate_policy_config(&cfg)?;
    validate_schema_version_transition(
        schema.schema_version.as_str(),
        cfg.schema_version.as_str(),
    )?;

    Ok(cfg)
}

pub fn validate_policy_config(cfg: &PolicyConfig) -> Result<(), PolicyValidationError> {
    if cfg.allow_override {
        return Err(PolicyValidationError(
            "allow_override must be false".to_string(),
        ));
    }
    if cfg.network_in_unit_tests {
        return Err(PolicyValidationError(
            "network_in_unit_tests must be false".to_string(),
        ));
    }

    if cfg.query_budget.cheap.max_limit == 0
        || cfg.query_budget.medium.max_limit == 0
        || cfg.query_budget.heavy.max_limit == 0
    {
        return Err(PolicyValidationError(
            "query_budget.{cheap,medium,heavy}.max_limit must be > 0".to_string(),
        ));
    }
    if cfg.query_budget.cheap.max_region_span == 0
        || cfg.query_budget.medium.max_region_span == 0
        || cfg.query_budget.heavy.max_region_span == 0
    {
        return Err(PolicyValidationError(
            "query_budget.{cheap,medium,heavy}.max_region_span must be > 0".to_string(),
        ));
    }
    if cfg.query_budget.cheap.max_region_estimated_rows == 0
        || cfg.query_budget.medium.max_region_estimated_rows == 0
        || cfg.query_budget.heavy.max_region_estimated_rows == 0
    {
        return Err(PolicyValidationError(
            "query_budget.{cheap,medium,heavy}.max_region_estimated_rows must be > 0".to_string(),
        ));
    }
    if cfg.query_budget.cheap.max_prefix_cost_units == 0
        || cfg.query_budget.medium.max_prefix_cost_units == 0
        || cfg.query_budget.heavy.max_prefix_cost_units == 0
    {
        return Err(PolicyValidationError(
            "query_budget.{cheap,medium,heavy}.max_prefix_cost_units must be > 0".to_string(),
        ));
    }
    if cfg.query_budget.max_limit == 0 {
        return Err(PolicyValidationError(
            "query_budget.max_limit must be > 0".to_string(),
        ));
    }
    if cfg.query_budget.max_transcript_limit == 0 {
        return Err(PolicyValidationError(
            "query_budget.max_transcript_limit must be > 0".to_string(),
        ));
    }
    if cfg.query_budget.heavy_projection_limit == 0 {
        return Err(PolicyValidationError(
            "query_budget.heavy_projection_limit must be > 0".to_string(),
        ));
    }
    if cfg.response_budget.max_serialization_bytes == 0 {
        return Err(PolicyValidationError(
            "response_budget.max_serialization_bytes must be > 0".to_string(),
        ));
    }
    if cfg.response_budget.cheap_max_bytes == 0
        || cfg.response_budget.medium_max_bytes == 0
        || cfg.response_budget.heavy_max_bytes == 0
    {
        return Err(PolicyValidationError(
            "response_budget class max bytes must be > 0".to_string(),
        ));
    }
    if cfg.query_budget.max_prefix_length == 0 {
        return Err(PolicyValidationError(
            "query_budget.max_prefix_length must be > 0".to_string(),
        ));
    }
    if cfg.query_budget.max_sequence_bases == 0 {
        return Err(PolicyValidationError(
            "query_budget.max_sequence_bases must be > 0".to_string(),
        ));
    }
    if cfg.query_budget.sequence_api_key_required_bases == 0 {
        return Err(PolicyValidationError(
            "query_budget.sequence_api_key_required_bases must be > 0".to_string(),
        ));
    }

    if cfg.cache_budget.max_disk_bytes == 0 {
        return Err(PolicyValidationError(
            "cache_budget.max_disk_bytes must be > 0".to_string(),
        ));
    }
    if cfg.cache_budget.max_dataset_count == 0 {
        return Err(PolicyValidationError(
            "cache_budget.max_dataset_count must be > 0".to_string(),
        ));
    }
    if cfg.cache_budget.shard_count_policy_max == 0 {
        return Err(PolicyValidationError(
            "cache_budget.shard_count_policy_max must be > 0".to_string(),
        ));
    }
    if cfg.cache_budget.max_open_shards_per_pod == 0 {
        return Err(PolicyValidationError(
            "cache_budget.max_open_shards_per_pod must be > 0".to_string(),
        ));
    }
    if cfg.store_resilience.retry_budget == 0
        || cfg.store_resilience.retry_attempts == 0
        || cfg.store_resilience.retry_base_backoff_ms == 0
        || cfg.store_resilience.breaker_failure_threshold == 0
        || cfg.store_resilience.breaker_open_ms == 0
    {
        return Err(PolicyValidationError(
            "store_resilience values must be > 0".to_string(),
        ));
    }

    if cfg.rate_limit.per_ip_rps == 0
        || cfg.rate_limit.per_api_key_rps == 0
        || cfg.rate_limit.sequence_per_ip_rps == 0
    {
        return Err(PolicyValidationError(
            "rate_limit values must be > 0".to_string(),
        ));
    }

    if cfg.concurrency_bulkheads.cheap == 0
        || cfg.concurrency_bulkheads.medium == 0
        || cfg.concurrency_bulkheads.heavy == 0
    {
        return Err(PolicyValidationError(
            "concurrency bulkheads must be > 0".to_string(),
        ));
    }

    if !cfg.telemetry.metrics_enabled {
        return Err(PolicyValidationError(
            "telemetry.metrics_enabled must be true".to_string(),
        ));
    }
    if !cfg.telemetry.tracing_enabled {
        return Err(PolicyValidationError(
            "telemetry.tracing_enabled must be true".to_string(),
        ));
    }
    if !cfg.telemetry.request_id_required {
        return Err(PolicyValidationError(
            "telemetry.request_id_required must be true".to_string(),
        ));
    }
    if cfg.telemetry.required_metric_labels.is_empty() {
        return Err(PolicyValidationError(
            "telemetry.required_metric_labels must not be empty".to_string(),
        ));
    }
    if cfg.telemetry.trace_sampling_per_10k == 0 {
        return Err(PolicyValidationError(
            "telemetry.trace_sampling_per_10k must be > 0".to_string(),
        ));
    }
    if cfg.publish_gates.required_indexes.is_empty() {
        return Err(PolicyValidationError(
            "publish_gates.required_indexes must not be empty".to_string(),
        ));
    }
    if cfg.publish_gates.min_gene_count == 0 {
        return Err(PolicyValidationError(
            "publish_gates.min_gene_count must be > 0".to_string(),
        ));
    }

    Ok(())
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

pub fn validate_policy_change_requires_version_bump(
    old_cfg: &PolicyConfig,
    new_cfg: &PolicyConfig,
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

pub fn canonical_config_json(cfg: &PolicyConfig) -> Result<String, PolicyValidationError> {
    let value = serde_json::to_value(cfg)
        .map_err(|e| PolicyValidationError(format!("encode config failed: {e}")))?;
    let normalized = normalize_json(value);
    serde_json::to_string_pretty(&normalized)
        .map_err(|e| PolicyValidationError(format!("print config failed: {e}")))
}

fn validate_strict_unknown_keys(value: &Value) -> Result<(), PolicyValidationError> {
    let obj = value
        .as_object()
        .ok_or_else(|| PolicyValidationError("policy config must be object".to_string()))?;

    let allowed: [&str; 12] = [
        "schema_version",
        "allow_override",
        "network_in_unit_tests",
        "query_budget",
        "response_budget",
        "cache_budget",
        "store_resilience",
        "rate_limit",
        "concurrency_bulkheads",
        "telemetry",
        "publish_gates",
        "documented_defaults",
    ];

    for key in obj.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(PolicyValidationError(format!(
                "unknown top-level policy key: {key}"
            )));
        }
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
    }

    Ok(())
}

fn decode_schema_version(schema: &Value) -> Result<PolicySchema, PolicyValidationError> {
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
