// SPDX-License-Identifier: Apache-2.0

use async_trait::async_trait;
use axum::body::Body;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, HeaderValue, Request, StatusCode, Uri};
use axum::middleware::{from_fn_with_state, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use bijux_atlas_api::{ApiError, ApiErrorCode};
use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{artifact_paths, ArtifactManifest, Catalog, DatasetId};
use bijux_atlas_query::{
    classify_query, decode_cursor, encode_cursor, estimate_query_cost, query_genes, CursorPayload,
    GeneFields, GeneFilter, GeneQueryRequest, OrderMode, QueryClass, QueryLimits, RegionFilter,
    TranscriptFilter, TranscriptQueryRequest,
};
use hmac::{Hmac, Mac};
use rusqlite::Connection;
use sha2::Sha256;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, OwnedSemaphorePermit, RwLock, Semaphore};
use tokio::time::timeout;
use tracing::{error, info, warn, Instrument};

pub const CRATE_NAME: &str = "bijux-atlas-server";

#[derive(Debug)]
pub struct CacheError(pub String);

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for CacheError {}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RegistrySourceHealth {
    pub name: String,
    pub priority: u32,
    pub healthy: bool,
    pub last_error: Option<String>,
    pub shadowed_datasets: u64,
    pub ttl_seconds: u64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DatasetCacheConfig {
    pub disk_root: PathBuf,
    pub max_disk_bytes: u64,
    pub disk_high_watermark_pct: u8,
    pub disk_low_watermark_pct: u8,
    pub max_dataset_count: usize,
    pub idle_ttl: Duration,
    pub pinned_datasets: HashSet<DatasetId>,
    pub read_only_fs: bool,
    pub cached_only_mode: bool,
    pub startup_warmup: Vec<DatasetId>,
    pub startup_warmup_limit: usize,
    pub fail_readiness_on_missing_warmup: bool,
    pub max_connections_per_dataset: usize,
    pub max_total_connections: usize,
    pub dataset_open_timeout: Duration,
    pub breaker_failure_threshold: u32,
    pub breaker_open_duration: Duration,
    pub store_breaker_failure_threshold: u32,
    pub store_breaker_open_duration: Duration,
    pub store_retry_budget: u32,
    pub max_concurrent_downloads: usize,
    pub max_concurrent_downloads_node: Option<usize>,
    pub eviction_check_interval: Duration,
    pub integrity_reverify_interval: Duration,
    pub sqlite_pragma_cache_kib: i64,
    pub sqlite_pragma_mmap_bytes: i64,
    pub max_open_shards_per_pod: usize,
    pub startup_warmup_jitter_max_ms: u64,
    pub catalog_backoff_base_ms: u64,
    pub catalog_breaker_failure_threshold: u32,
    pub catalog_breaker_open_ms: u64,
    pub quarantine_after_corruption_failures: u32,
    pub quarantine_retry_ttl: Duration,
    pub registry_ttl: Duration,
    pub registry_freeze_mode: bool,
}

impl Default for DatasetCacheConfig {
    fn default() -> Self {
        Self {
            disk_root: PathBuf::from("artifacts/server-cache"),
            max_disk_bytes: 4 * 1024 * 1024 * 1024,
            disk_high_watermark_pct: 90,
            disk_low_watermark_pct: 75,
            max_dataset_count: 8,
            idle_ttl: Duration::from_secs(1800),
            pinned_datasets: HashSet::new(),
            read_only_fs: false,
            cached_only_mode: false,
            startup_warmup: Vec::new(),
            startup_warmup_limit: 8,
            fail_readiness_on_missing_warmup: false,
            max_connections_per_dataset: 8,
            max_total_connections: 64,
            dataset_open_timeout: Duration::from_secs(3),
            breaker_failure_threshold: 3,
            breaker_open_duration: Duration::from_secs(30),
            store_breaker_failure_threshold: 5,
            store_breaker_open_duration: Duration::from_secs(20),
            store_retry_budget: 20,
            max_concurrent_downloads: 3,
            max_concurrent_downloads_node: None,
            eviction_check_interval: Duration::from_secs(30),
            integrity_reverify_interval: Duration::from_secs(300),
            sqlite_pragma_cache_kib: 32 * 1024,
            sqlite_pragma_mmap_bytes: 256 * 1024 * 1024,
            max_open_shards_per_pod: 16,
            startup_warmup_jitter_max_ms: 0,
            catalog_backoff_base_ms: 250,
            catalog_breaker_failure_threshold: 5,
            catalog_breaker_open_ms: 5000,
            quarantine_after_corruption_failures: 3,
            quarantine_retry_ttl: Duration::from_secs(300),
            registry_ttl: Duration::from_secs(15),
            registry_freeze_mode: false,
        }
    }
}

#[derive(Default)]
pub struct CacheMetrics {
    pub dataset_hits: AtomicU64,
    pub dataset_misses: AtomicU64,
    pub dataset_count: AtomicU64,
    pub disk_usage_bytes: AtomicU64,
    pub catalog_epoch_hash: RwLock<String>,
    pub store_download_latency_ns: Mutex<Vec<u64>>,
    pub store_open_latency_ns: Mutex<Vec<u64>>,
    pub store_download_failures: AtomicU64,
    pub store_open_failures: AtomicU64,
    pub store_breaker_open_total: AtomicU64,
    pub store_breaker_half_open_total: AtomicU64,
    pub store_retry_budget_exhausted_total: AtomicU64,
    pub store_download_ttfb_ns: Mutex<Vec<u64>>,
    pub store_download_bytes_total: AtomicU64,
    pub store_download_retry_total: AtomicU64,
    pub store_error_checksum_total: AtomicU64,
    pub store_error_timeout_total: AtomicU64,
    pub store_error_network_total: AtomicU64,
    pub store_error_other_total: AtomicU64,
    pub store_errors_by_backend_and_class: Mutex<HashMap<(String, String), u64>>,
    pub verify_marker_fast_path_hits: AtomicU64,
    pub verify_full_hash_checks: AtomicU64,
    pub cheap_queries_served_while_overloaded_total: AtomicU64,
    pub disk_io_latency_ns: Mutex<Vec<u64>>,
    pub fs_space_pressure_events_total: AtomicU64,
    pub cache_evictions_total: AtomicU64,
    pub registry_invalidation_events_total: AtomicU64,
    pub registry_refresh_failures_total: AtomicU64,
    pub policy_violations_total: AtomicU64,
    pub policy_violations_by_policy: Mutex<HashMap<String, u64>>,
    pub shed_total_by_reason: Mutex<HashMap<String, u64>>,
    pub dataset_missing_by_hash_bucket: Mutex<HashMap<String, u64>>,
    pub invariant_violations_by_name: Mutex<HashMap<String, u64>>,
}

#[derive(Default)]
struct RequestMetrics {
    counts: Mutex<HashMap<(String, String, u16, String), u64>>,
    latency_ns: Mutex<HashMap<String, Vec<u64>>>,
    sqlite_latency_ns: Mutex<HashMap<String, Vec<u64>>>,
    stage_latency_ns: Mutex<HashMap<String, Vec<u64>>>,
    request_size_bytes: Mutex<HashMap<String, Vec<u64>>>,
    response_size_bytes: Mutex<HashMap<String, Vec<u64>>>,
    heavy_latency_recent_ns: Mutex<VecDeque<u64>>,
    exemplars: Mutex<HashMap<RequestMetricKey, RequestExemplar>>,
    client_fingerprint_counts: Mutex<HashMap<(String, String), u64>>,
}

type RequestMetricKey = (String, String, u16, String);
type RequestExemplar = (String, u128);

impl RequestMetrics {
    pub(crate) async fn observe_request(&self, route: &str, status: StatusCode, latency: Duration) {
        self.observe_request_with_trace_and_method(route, "GET", status, latency, None)
            .await;
    }

    pub(crate) async fn observe_request_with_method(
        &self,
        route: &str,
        method: &str,
        status: StatusCode,
        latency: Duration,
    ) {
        self.observe_request_with_trace_and_method(route, method, status, latency, None)
            .await;
    }

    pub(crate) async fn observe_request_with_trace(
        &self,
        route: &str,
        status: StatusCode,
        latency: Duration,
        trace_id: Option<&str>,
    ) {
        self.observe_request_with_trace_and_method(route, "GET", status, latency, trace_id)
            .await;
    }

    pub(crate) async fn observe_request_with_trace_and_method(
        &self,
        route: &str,
        method: &str,
        status: StatusCode,
        latency: Duration,
        trace_id: Option<&str>,
    ) {
        let class = route_sli_class(route);
        let mut counts = self.counts.lock().await;
        *counts
            .entry((
                route.to_string(),
                method.to_ascii_uppercase(),
                status.as_u16(),
                class.to_string(),
            ))
            .or_insert(0) += 1;
        drop(counts);
        let mut latency_map = self.latency_ns.lock().await;
        latency_map
            .entry(route.to_string())
            .or_insert_with(Vec::new)
            .push(latency.as_nanos() as u64);
        if let Some(id) = trace_id {
            let mut ex = self.exemplars.lock().await;
            ex.insert(
                (
                    route.to_string(),
                    method.to_ascii_uppercase(),
                    status.as_u16(),
                    class.to_string(),
                ),
                (id.to_string(), chrono_like_unix_millis()),
            );
        }
    }

    pub(crate) async fn observe_sqlite_query(&self, query_type: &str, latency: Duration) {
        let mut q = self.sqlite_latency_ns.lock().await;
        q.entry(query_type.to_string())
            .or_insert_with(Vec::new)
            .push(latency.as_nanos() as u64);
        if query_type == "heavy" {
            let mut recent = self.heavy_latency_recent_ns.lock().await;
            recent.push_back(latency.as_nanos() as u64);
            while recent.len() > 512 {
                recent.pop_front();
            }
        }
    }

    pub(crate) async fn observe_stage(&self, stage: &str, latency: Duration) {
        let mut m = self.stage_latency_ns.lock().await;
        m.entry(stage.to_string())
            .or_insert_with(Vec::new)
            .push(latency.as_nanos() as u64);
    }

    pub(crate) async fn observe_request_size(&self, route: &str, bytes: usize) {
        let mut m = self.request_size_bytes.lock().await;
        m.entry(route.to_string())
            .or_insert_with(Vec::new)
            .push(bytes as u64);
    }

    pub(crate) async fn observe_response_size(&self, route: &str, bytes: usize) {
        let mut m = self.response_size_bytes.lock().await;
        m.entry(route.to_string())
            .or_insert_with(Vec::new)
            .push(bytes as u64);
    }

    pub(crate) async fn should_shed_heavy(&self, min_samples: usize, threshold_ms: u64) -> bool {
        let recent = self.heavy_latency_recent_ns.lock().await;
        if recent.len() < min_samples {
            return false;
        }
        let mut v: Vec<u64> = recent.iter().copied().collect();
        v.sort_unstable();
        let idx = ((v.len() as f64) * 0.95).ceil() as usize - 1;
        let p95_ns = v[idx.min(v.len() - 1)];
        p95_ns > (threshold_ms * 1_000_000)
    }

    pub(crate) async fn observe_client_fingerprint(
        &self,
        client_type: &str,
        user_agent_family: &str,
    ) {
        let mut counts = self.client_fingerprint_counts.lock().await;
        *counts
            .entry((client_type.to_string(), user_agent_family.to_string()))
            .or_insert(0) += 1;
    }
}

include!("request_utils.rs");
async fn cors_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let origin = normalized_header_value(req.headers(), "origin", 256);
    let method = req.method().clone();
    if method == axum::http::Method::OPTIONS {
        let mut resp = StatusCode::NO_CONTENT.into_response();
        if let Some(origin_value) = origin {
            if state
                .api
                .cors_allowed_origins
                .iter()
                .any(|x| x == &origin_value)
            {
                if let Ok(v) = HeaderValue::from_str(&origin_value) {
                    resp.headers_mut().insert("access-control-allow-origin", v);
                }
                resp.headers_mut().insert(
                    "access-control-allow-methods",
                    HeaderValue::from_static("GET,OPTIONS"),
                );
                resp.headers_mut().insert(
                    "access-control-allow-headers",
                    HeaderValue::from_static(
                        "x-api-key,x-bijux-signature,x-bijux-timestamp,content-type",
                    ),
                );
            }
        }
        return resp;
    }

    let mut resp = next.run(req).await;
    if let Some(origin_value) = origin {
        if state
            .api
            .cors_allowed_origins
            .iter()
            .any(|x| x == &origin_value)
        {
            if let Ok(v) = HeaderValue::from_str(&origin_value) {
                resp.headers_mut().insert("access-control-allow-origin", v);
            }
            resp.headers_mut()
                .insert("vary", HeaderValue::from_static("Origin"));
        }
    }
    resp
}

pub use config::{
    effective_config_payload, load_runtime_startup_config, runtime_startup_config_docs_markdown,
    runtime_startup_config_schema_json, validate_startup_config_contract, ApiConfig,
    RateLimitConfig, RuntimeStartupConfig,
};
pub use routing_hash::consistent_route_dataset;
pub use store::backends::{LocalFsBackend, RetryPolicy, S3LikeBackend};
pub use store::federated::{FederatedBackend, RegistrySource};

#[async_trait]
pub trait DatasetStoreBackend: Send + Sync + 'static {
    fn backend_tag(&self) -> &'static str {
        "unknown"
    }

    async fn fetch_catalog(&self, if_none_match: Option<&str>) -> Result<CatalogFetch, CacheError>;
    async fn fetch_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, CacheError>;
    async fn fetch_sqlite_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, CacheError>;
    async fn fetch_fasta_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, CacheError>;
    async fn fetch_fai_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, CacheError>;
    async fn fetch_release_gene_index_bytes(
        &self,
        dataset: &DatasetId,
    ) -> Result<Vec<u8>, CacheError>;

    async fn registry_health(&self) -> Vec<RegistrySourceHealth> {
        vec![RegistrySourceHealth {
            name: "primary".to_string(),
            priority: 0,
            healthy: true,
            last_error: None,
            shadowed_datasets: 0,
            ttl_seconds: 0,
        }]
    }
}

#[non_exhaustive]
pub enum CatalogFetch {
    NotModified,
    Updated { etag: String, catalog: Catalog },
}

struct DatasetEntry {
    sqlite_path: PathBuf,
    shard_sqlite_paths: Vec<PathBuf>,
    shard_by_seqid: HashMap<String, Vec<PathBuf>>,
    last_access: Instant,
    size_bytes: u64,
    dataset_semaphore: Arc<Semaphore>,
    query_semaphore: Arc<Semaphore>,
}

#[derive(Default)]
struct CatalogCache {
    etag: Option<String>,
    catalog: Option<Catalog>,
    consecutive_errors: u32,
    backoff_until: Option<Instant>,
    breaker_open_until: Option<Instant>,
    refreshed_at: Option<Instant>,
}

#[derive(Default)]
struct BreakerState {
    failure_count: u32,
    open_until: Option<Instant>,
}

#[derive(Default)]
struct StoreBreakerState {
    failure_count: u32,
    open_until: Option<Instant>,
}

use telemetry::rate_limiter::RateLimiter;
use telemetry::redis_backend::RedisBackend;

pub struct DatasetConnection {
    pub conn: Connection,
    _global_permit: OwnedSemaphorePermit,
    _dataset_permit: OwnedSemaphorePermit,
    _query_permit: OwnedSemaphorePermit,
}

#[derive(Debug, Clone)]
pub struct DatasetHealthSnapshot {
    pub cached: bool,
    pub checksum_verified: bool,
    pub last_open_seconds_ago: Option<u64>,
    pub size_bytes: Option<u64>,
    pub open_failures: u32,
    pub quarantined: bool,
}

pub struct DatasetCacheManager {
    cfg: DatasetCacheConfig,
    store: Arc<dyn DatasetStoreBackend>,
    entries: Mutex<HashMap<DatasetId, DatasetEntry>>,
    inflight: Mutex<HashMap<DatasetId, Arc<Mutex<()>>>>,
    breakers: Mutex<HashMap<DatasetId, BreakerState>>,
    quarantine_failures: Mutex<HashMap<DatasetId, u32>>,
    quarantined: Mutex<HashSet<DatasetId>>,
    store_breaker: Mutex<StoreBreakerState>,
    catalog_cache: Mutex<CatalogCache>,
    registry_health_cache: RwLock<Vec<RegistrySourceHealth>>,
    global_semaphore: Arc<Semaphore>,
    download_semaphore: Arc<Semaphore>,
    shard_open_semaphore: Arc<Semaphore>,
    retry_budget_remaining: AtomicU64,
    dataset_retry_budget: Mutex<HashMap<DatasetId, u32>>,
    pub metrics: Arc<CacheMetrics>,
}

#[derive(Clone)]
pub struct AppState {
    pub cache: Arc<DatasetCacheManager>,
    pub api: ApiConfig,
    pub limits: QueryLimits,
    pub ready: Arc<AtomicBool>,
    pub accepting_requests: Arc<AtomicBool>,
    pub(crate) ip_limiter: Arc<RateLimiter>,
    pub(crate) sequence_ip_limiter: Arc<RateLimiter>,
    pub(crate) api_key_limiter: Arc<RateLimiter>,
    pub class_cheap: Arc<Semaphore>,
    pub class_medium: Arc<Semaphore>,
    pub class_heavy: Arc<Semaphore>,
    pub(crate) heavy_workers: Arc<Semaphore>,
    pub(crate) metrics: Arc<RequestMetrics>,
    pub(crate) request_id_seed: Arc<AtomicU64>,
    pub(crate) coalescer: Arc<cache::coalesce::QueryCoalescer>,
    pub(crate) hot_query_cache: Arc<Mutex<cache::hot::HotQueryCache>>,
    pub(crate) redis_backend: Option<Arc<RedisBackend>>,
    pub(crate) queued_requests: Arc<AtomicU64>,
    pub runtime_policy_hash: Arc<String>,
    pub runtime_policy_mode: Arc<String>,
}
