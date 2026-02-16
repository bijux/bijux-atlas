use std::time::Duration;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub max_body_bytes: usize,
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
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            max_body_bytes: 16 * 1024,
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
        }
    }
}
