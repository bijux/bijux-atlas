use serde::Serialize;
use std::time::Duration;

pub const CONFIG_SCHEMA_VERSION: &str = "1";

#[derive(Debug, Clone, Serialize)]
pub struct RateLimitConfig {
    pub capacity: f64,
    pub refill_per_sec: f64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            capacity: 30.0,
            refill_per_sec: 10.0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiConfig {
    pub max_body_bytes: usize,
    pub max_uri_bytes: usize,
    pub max_header_bytes: usize,
    pub request_timeout: Duration,
    pub sql_timeout: Duration,
    pub response_max_bytes: usize,
    pub discovery_ttl: Duration,
    pub immutable_gene_ttl: Duration,
    pub enable_debug_datasets: bool,
    pub enable_api_key_rate_limit: bool,
    pub rate_limit_per_ip: RateLimitConfig,
    pub rate_limit_per_api_key: RateLimitConfig,
    pub concurrency_cheap: usize,
    pub concurrency_medium: usize,
    pub concurrency_heavy: usize,
    pub slow_query_threshold: Duration,
    pub enable_exemplars: bool,
    pub readiness_requires_catalog: bool,
    pub heavy_worker_pool_size: usize,
    pub shed_load_enabled: bool,
    pub shed_latency_p95_threshold_ms: u64,
    pub shed_latency_min_samples: usize,
    pub enable_response_compression: bool,
    pub compression_min_bytes: usize,
    pub query_coalesce_ttl: Duration,
    pub redis_url: Option<String>,
    pub redis_prefix: String,
    pub enable_redis_response_cache: bool,
    pub redis_response_cache_ttl_secs: usize,
    pub enable_redis_rate_limit: bool,
    pub redis_timeout_ms: u64,
    pub redis_retry_attempts: usize,
    pub redis_breaker_failure_threshold: u32,
    pub redis_breaker_open_ms: u64,
    pub redis_cache_max_key_bytes: usize,
    pub redis_cache_max_cardinality: usize,
    pub redis_cache_ttl_max_secs: usize,
    pub enable_cheap_only_survival: bool,
    pub allow_min_viable_response: bool,
    pub continue_download_on_request_timeout_for_warmup: bool,
    pub max_sequence_bases: usize,
    pub sequence_api_key_required_bases: usize,
    pub sequence_rate_limit_per_ip: RateLimitConfig,
    pub sequence_ttl: Duration,
    pub adaptive_rate_limit_factor: f64,
    pub adaptive_heavy_limit_factor: f64,
    pub emergency_global_breaker: bool,
    pub disable_heavy_endpoints: bool,
    pub memory_pressure_shed_enabled: bool,
    pub memory_pressure_rss_bytes: u64,
    pub max_request_queue_depth: usize,
    pub cors_allowed_origins: Vec<String>,
    pub enable_audit_log: bool,
    pub require_api_key: bool,
    pub allowed_api_keys: Vec<String>,
    pub hmac_secret: Option<String>,
    pub hmac_required: bool,
    pub hmac_max_skew_secs: u64,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            max_body_bytes: 16 * 1024,
            max_uri_bytes: 2048,
            max_header_bytes: 16 * 1024,
            request_timeout: Duration::from_secs(5),
            sql_timeout: Duration::from_millis(800),
            response_max_bytes: 512 * 1024,
            discovery_ttl: Duration::from_secs(30),
            immutable_gene_ttl: Duration::from_secs(900),
            enable_debug_datasets: false,
            enable_api_key_rate_limit: false,
            rate_limit_per_ip: RateLimitConfig::default(),
            rate_limit_per_api_key: RateLimitConfig {
                capacity: 100.0,
                refill_per_sec: 30.0,
            },
            concurrency_cheap: 128,
            concurrency_medium: 64,
            concurrency_heavy: 16,
            slow_query_threshold: Duration::from_millis(200),
            enable_exemplars: false,
            readiness_requires_catalog: true,
            heavy_worker_pool_size: 8,
            shed_load_enabled: false,
            shed_latency_p95_threshold_ms: 900,
            shed_latency_min_samples: 50,
            enable_response_compression: true,
            compression_min_bytes: 4096,
            query_coalesce_ttl: Duration::from_millis(500),
            redis_url: None,
            redis_prefix: "atlas".to_string(),
            enable_redis_response_cache: false,
            redis_response_cache_ttl_secs: 30,
            enable_redis_rate_limit: false,
            redis_timeout_ms: 50,
            redis_retry_attempts: 2,
            redis_breaker_failure_threshold: 8,
            redis_breaker_open_ms: 3000,
            redis_cache_max_key_bytes: 256,
            redis_cache_max_cardinality: 100_000,
            redis_cache_ttl_max_secs: 60,
            enable_cheap_only_survival: false,
            allow_min_viable_response: true,
            continue_download_on_request_timeout_for_warmup: true,
            max_sequence_bases: 20_000,
            sequence_api_key_required_bases: 5_000,
            sequence_rate_limit_per_ip: RateLimitConfig {
                capacity: 15.0,
                refill_per_sec: 5.0,
            },
            sequence_ttl: Duration::from_secs(300),
            adaptive_rate_limit_factor: 0.5,
            adaptive_heavy_limit_factor: 0.5,
            emergency_global_breaker: false,
            disable_heavy_endpoints: false,
            memory_pressure_shed_enabled: false,
            memory_pressure_rss_bytes: 3 * 1024 * 1024 * 1024,
            max_request_queue_depth: 256,
            cors_allowed_origins: Vec::new(),
            enable_audit_log: false,
            require_api_key: false,
            allowed_api_keys: Vec::new(),
            hmac_secret: None,
            hmac_required: false,
            hmac_max_skew_secs: 300,
        }
    }
}

pub fn validate_startup_config_contract(
    api: &ApiConfig,
    cache: &crate::DatasetCacheConfig,
) -> Result<(), String> {
    if api.max_body_bytes == 0 || api.max_uri_bytes == 0 || api.max_header_bytes == 0 {
        return Err("api size limits must be > 0".to_string());
    }
    if api.request_timeout.is_zero() || api.sql_timeout.is_zero() {
        return Err("timeouts must be > 0".to_string());
    }
    if cache.disk_high_watermark_pct <= cache.disk_low_watermark_pct {
        return Err("cache watermark contract requires high > low".to_string());
    }
    if cache.max_dataset_count == 0 || cache.max_total_connections == 0 {
        return Err("cache capacity limits must be > 0".to_string());
    }
    if cache.max_concurrent_downloads == 0 {
        return Err("max concurrent downloads must be > 0".to_string());
    }
    if api.require_api_key && api.allowed_api_keys.is_empty() {
        return Err("require_api_key=true requires at least one allowed api key".to_string());
    }
    if api.hmac_required && api.hmac_secret.as_deref().is_none_or(str::is_empty) {
        return Err("hmac_required=true requires a non-empty hmac_secret".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_config_validation_rejects_invalid_watermark_order() {
        let api = ApiConfig::default();
        let cache = crate::DatasetCacheConfig {
            disk_high_watermark_pct: 70,
            disk_low_watermark_pct: 75,
            ..crate::DatasetCacheConfig::default()
        };
        let err = validate_startup_config_contract(&api, &cache).expect_err("invalid watermarks");
        assert!(err.contains("high > low"));
    }

    #[test]
    fn startup_config_validation_enforces_auth_contracts() {
        let mut api = ApiConfig {
            require_api_key: true,
            ..ApiConfig::default()
        };
        let cache = crate::DatasetCacheConfig::default();
        let err = validate_startup_config_contract(&api, &cache).expect_err("missing keys");
        assert!(err.contains("allowed api key"));

        api.allowed_api_keys = vec!["k".to_string()];
        api.hmac_required = true;
        api.hmac_secret = None;
        let err = validate_startup_config_contract(&api, &cache).expect_err("missing hmac");
        assert!(err.contains("hmac_secret"));
    }
}
