use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PolicySchemaVersion {
    #[serde(rename = "1")]
    V1,
}

impl PolicySchemaVersion {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::V1 => "1",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyConfig {
    pub schema_version: PolicySchemaVersion,
    pub mode: PolicyMode,
    pub allow_override: bool,
    pub network_in_unit_tests: bool,
    pub modes: PolicyModes,
    pub query_budget: QueryBudgetPolicy,
    pub response_budget: ResponseBudgetPolicy,
    pub cache_budget: CacheBudget,
    pub store_resilience: StoreResiliencePolicy,
    pub rate_limit: RateLimitPolicy,
    pub concurrency_bulkheads: ConcurrencyBulkheads,
    pub telemetry: TelemetryPolicy,
    pub publish_gates: PublishGates,
    pub documented_defaults: Vec<DocumentedDefault>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyMode {
    Strict,
    Compat,
    Dev,
}

impl PolicyMode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Strict => "strict",
            Self::Compat => "compat",
            Self::Dev => "dev",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyModeProfile {
    pub allow_override: bool,
    pub max_page_size: u32,
    pub max_region_span: u64,
    pub max_response_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyModes {
    pub strict: PolicyModeProfile,
    pub compat: PolicyModeProfile,
    pub dev: PolicyModeProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EndpointClassBudget {
    pub max_limit: u32,
    pub max_region_span: u64,
    pub max_region_estimated_rows: u64,
    pub max_prefix_cost_units: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryBudgetPolicy {
    pub cheap: EndpointClassBudget,
    pub medium: EndpointClassBudget,
    pub heavy: EndpointClassBudget,
    pub max_limit: u32,
    pub max_transcript_limit: u32,
    pub heavy_projection_limit: u32,
    pub max_prefix_length: u32,
    pub max_sequence_bases: u32,
    pub sequence_api_key_required_bases: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseBudgetPolicy {
    pub cheap_max_bytes: u64,
    pub medium_max_bytes: u64,
    pub heavy_max_bytes: u64,
    pub max_serialization_bytes: u64,
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
pub struct StoreResiliencePolicy {
    pub retry_budget: u32,
    pub retry_attempts: u32,
    pub retry_base_backoff_ms: u64,
    pub breaker_failure_threshold: u32,
    pub breaker_open_ms: u64,
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
    pub required_metric_labels: Vec<String>,
    pub trace_sampling_per_10k: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublishGates {
    pub required_indexes: Vec<String>,
    pub min_gene_count: u64,
    pub max_missing_parents: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DocumentedDefault {
    pub field: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySchema {
    pub schema_version: PolicySchemaVersion,
}
