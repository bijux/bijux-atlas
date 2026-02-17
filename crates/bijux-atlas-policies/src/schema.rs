use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyConfig {
    pub schema_version: String,
    pub allow_override: bool,
    pub network_in_unit_tests: bool,
    pub query_budget: QueryBudget,
    pub cache_budget: CacheBudget,
    pub rate_limit: RateLimitPolicy,
    pub concurrency_bulkheads: ConcurrencyBulkheads,
    pub telemetry: TelemetryPolicy,
    pub documented_defaults: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryBudget {
    pub max_limit: u32,
    pub max_transcript_limit: u32,
    pub max_region_span: u64,
    pub max_region_estimated_rows: u64,
    pub max_prefix_cost_units: u64,
    pub heavy_projection_limit: u32,
    pub max_serialization_bytes: u64,
    pub max_prefix_length: u32,
    pub max_sequence_bases: u32,
    pub sequence_api_key_required_bases: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CacheBudget {
    pub max_disk_bytes: u64,
    pub max_dataset_count: u32,
    pub pinned_datasets_max: u32,
    pub shard_count_policy_max: u32,
    pub max_open_shards_per_pod: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RateLimitPolicy {
    pub per_ip_rps: u32,
    pub per_api_key_rps: u32,
    pub sequence_per_ip_rps: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConcurrencyBulkheads {
    pub cheap: u32,
    pub medium: u32,
    pub heavy: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetryPolicy {
    pub metrics_enabled: bool,
    pub tracing_enabled: bool,
    pub slow_query_log_enabled: bool,
    pub request_id_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySchema {
    pub schema_version: String,
}
