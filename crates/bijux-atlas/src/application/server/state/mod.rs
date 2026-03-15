// SPDX-License-Identifier: Apache-2.0

use crate::application::server::cache;
use crate::application::config::ApiConfig;
use crate::contracts::api::{ApiError, ApiErrorCode};
use crate::domain::dataset::{artifact_paths, ArtifactManifest, Catalog, DatasetId};
use crate::domain::{
    FailureRecoveryRegistry, MembershipRegistry, ReplicaRegistry, ShardRegistry, sha256_hex,
};
use crate::http;
use async_trait::async_trait;
use axum::body::Body;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, HeaderValue, Request, StatusCode, Uri};
use axum::middleware::{from_fn_with_state, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use bijux_atlas::query::QueryLimits;
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

pub(crate) mod cache_runtime;
mod request_utils;
mod router;

use self::request_utils::*;

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
            disk_root: crate::application::config::default_runtime_cache_root(),
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
    pub warmup_lock_contention_total: AtomicU64,
    pub warmup_lock_expired_total: AtomicU64,
    pub warmup_lock_wait_ns: Mutex<Vec<u64>>,
    pub cache_evictions_total: AtomicU64,
    pub registry_invalidation_events_total: AtomicU64,
    pub registry_refresh_failures_total: AtomicU64,
    pub policy_violations_total: AtomicU64,
    pub policy_violations_by_policy: Mutex<HashMap<String, u64>>,
    pub shed_total_by_reason: Mutex<HashMap<String, u64>>,
    pub dataset_missing_by_hash_bucket: Mutex<HashMap<String, u64>>,
    pub invariant_violations_by_name: Mutex<HashMap<String, u64>>,
    pub encryption_operations_total: AtomicU64,
    pub integrity_violations_total: AtomicU64,
    pub tamper_detections_total: AtomicU64,
}

#[derive(Default)]
pub(crate) struct RequestMetrics {
    pub(crate) counts: Mutex<HashMap<(String, String, u16, String), u64>>,
    pub(crate) latency_ns: Mutex<HashMap<String, Vec<u64>>>,
    pub(crate) sqlite_latency_ns: Mutex<HashMap<String, Vec<u64>>>,
    pub(crate) stage_latency_ns: Mutex<HashMap<String, Vec<u64>>>,
    pub(crate) query_row_count: Mutex<HashMap<String, Vec<u64>>>,
    pub(crate) request_size_bytes: Mutex<HashMap<String, Vec<u64>>>,
    pub(crate) response_size_bytes: Mutex<HashMap<String, Vec<u64>>>,
    pub(crate) heavy_latency_recent_ns: Mutex<VecDeque<u64>>,
    pub(crate) exemplars: Mutex<HashMap<RequestMetricKey, RequestExemplar>>,
    pub(crate) client_fingerprint_counts: Mutex<HashMap<(String, String), u64>>,
    pub(crate) query_cache_hits_total: AtomicU64,
    pub(crate) query_cache_misses_total: AtomicU64,
    pub(crate) slow_queries_total: AtomicU64,
    pub(crate) dataset_query_distribution: Mutex<HashMap<String, u64>>,
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

    pub(crate) async fn observe_query_row_count(&self, route: &str, rows: usize) {
        let mut m = self.query_row_count.lock().await;
        m.entry(route.to_string())
            .or_insert_with(Vec::new)
            .push(rows as u64);
    }

    pub(crate) fn observe_query_cache_hit(&self) {
        self.query_cache_hits_total.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn observe_query_cache_miss(&self) {
        self.query_cache_misses_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn observe_slow_query(&self) {
        self.slow_queries_total.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn slow_queries_total(&self) -> u64 {
        self.slow_queries_total.load(Ordering::Relaxed)
    }

    pub(crate) async fn observe_dataset_query(&self, dataset_key: &str) {
        let mut m = self.dataset_query_distribution.lock().await;
        *m.entry(dataset_key.to_string()).or_insert(0) += 1;
    }

    pub(crate) async fn dataset_query_distribution_snapshot(&self) -> HashMap<String, u64> {
        self.dataset_query_distribution.lock().await.clone()
    }

    pub(crate) async fn query_planner_stats_snapshot(&self) -> serde_json::Value {
        let stage_latency = self.stage_latency_ns.lock().await;
        let query_plan = stage_latency.get("query_plan").cloned().unwrap_or_default();
        let query_exec = stage_latency.get("query").cloned().unwrap_or_default();
        let sqlite_latency = self.sqlite_latency_ns.lock().await.clone();
        let query_rows = self.query_row_count.lock().await.clone();
        serde_json::json!({
            "query_plan_samples": query_plan.len(),
            "query_plan_latency_ns": query_plan,
            "query_execution_samples": query_exec.len(),
            "query_execution_latency_ns": query_exec,
            "sqlite_latency_ns_by_type": sqlite_latency,
            "query_row_count_by_route": query_rows
        })
    }

    pub(crate) async fn runtime_stats_snapshot(&self) -> serde_json::Value {
        let counts = self.counts.lock().await.clone();
        let latency = self.latency_ns.lock().await.clone();
        let request_sizes = self.request_size_bytes.lock().await.clone();
        let response_sizes = self.response_size_bytes.lock().await.clone();
        let client_fingerprints = self.client_fingerprint_counts.lock().await.clone();
        serde_json::json!({
            "request_counts": counts,
            "latency_ns_by_route": latency,
            "request_size_bytes_by_route": request_sizes,
            "response_size_bytes_by_route": response_sizes,
            "client_fingerprints": client_fingerprints,
            "query_cache_hits_total": self.query_cache_hits_total.load(Ordering::Relaxed),
            "query_cache_misses_total": self.query_cache_misses_total.load(Ordering::Relaxed),
            "slow_queries_total": self.slow_queries_total.load(Ordering::Relaxed),
            "dataset_query_distribution": self.dataset_query_distribution.lock().await.clone()
        })
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

pub use crate::store::registry::backends::{LocalFsBackend, RetryPolicy, S3LikeBackend};
pub use crate::store::registry::federated::{FederatedBackend, RegistrySource};
pub use self::request_utils::{chrono_like_unix_millis, record_shed_reason, route_sli_class};
pub use self::router::build_router;

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

pub(crate) struct DatasetEntry {
    pub(crate) sqlite_path: PathBuf,
    pub(crate) shard_sqlite_paths: Vec<PathBuf>,
    pub(crate) shard_by_seqid: HashMap<String, Vec<PathBuf>>,
    pub(crate) last_access: Instant,
    pub(crate) size_bytes: u64,
    pub(crate) dataset_semaphore: Arc<Semaphore>,
    pub(crate) query_semaphore: Arc<Semaphore>,
}

#[derive(Default)]
pub(crate) struct CatalogCache {
    etag: Option<String>,
    catalog: Option<Catalog>,
    consecutive_errors: u32,
    backoff_until: Option<Instant>,
    breaker_open_until: Option<Instant>,
    refreshed_at: Option<Instant>,
}

#[derive(Default)]
pub(crate) struct BreakerState {
    failure_count: u32,
    open_until: Option<Instant>,
}

#[derive(Default)]
pub(crate) struct StoreBreakerState {
    pub(crate) failure_count: u32,
    pub(crate) open_until: Option<Instant>,
}

use crate::telemetry::rate_limiter::RateLimiter;
use crate::redis::RedisBackend;

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
    pub(crate) cfg: DatasetCacheConfig,
    pub(crate) store: Arc<dyn DatasetStoreBackend>,
    pub(crate) entries: Mutex<HashMap<DatasetId, DatasetEntry>>,
    pub(crate) inflight: Mutex<HashMap<DatasetId, Arc<Mutex<()>>>>,
    pub(crate) breakers: Mutex<HashMap<DatasetId, BreakerState>>,
    pub(crate) quarantine_failures: Mutex<HashMap<DatasetId, u32>>,
    pub(crate) quarantined: Mutex<HashSet<DatasetId>>,
    pub(crate) store_breaker: Mutex<StoreBreakerState>,
    pub(crate) catalog_cache: Mutex<CatalogCache>,
    pub(crate) registry_health_cache: RwLock<Vec<RegistrySourceHealth>>,
    pub(crate) global_semaphore: Arc<Semaphore>,
    pub(crate) download_semaphore: Arc<Semaphore>,
    pub(crate) shard_open_semaphore: Arc<Semaphore>,
    pub(crate) retry_budget_remaining: AtomicU64,
    pub(crate) dataset_retry_budget: Mutex<HashMap<DatasetId, u32>>,
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
    pub(crate) membership: Arc<Mutex<MembershipRegistry>>,
    pub(crate) shard_registry: Arc<Mutex<ShardRegistry>>,
    pub(crate) replica_registry: Arc<Mutex<ReplicaRegistry>>,
    pub(crate) resilience_registry: Arc<Mutex<FailureRecoveryRegistry>>,
    pub runtime_policy_hash: Arc<String>,
    pub runtime_policy_mode: Arc<String>,
}

#[cfg(test)]
mod metrics_tests {
    use super::*;

    #[tokio::test]
    async fn request_metrics_track_query_cache_hits_misses_and_row_count() {
        let metrics = RequestMetrics::default();
        metrics.observe_query_cache_hit();
        metrics.observe_query_cache_hit();
        metrics.observe_query_cache_miss();
        metrics.observe_query_row_count("/v1/genes", 12).await;

        assert_eq!(metrics.query_cache_hits_total.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.query_cache_misses_total.load(Ordering::Relaxed), 1);

        let rows = metrics.query_row_count.lock().await;
        let samples = rows.get("/v1/genes").expect("row count samples must exist");
        assert_eq!(samples.as_slice(), &[12]);
    }

    #[tokio::test]
    async fn request_metrics_track_slow_queries_and_dataset_distribution() {
        let metrics = RequestMetrics::default();
        metrics.observe_slow_query();
        metrics.observe_dataset_query("ds-abc123").await;
        metrics.observe_dataset_query("ds-abc123").await;
        metrics.observe_dataset_query("ds-def456").await;

        assert_eq!(metrics.slow_queries_total.load(Ordering::Relaxed), 1);
        let dist = metrics.dataset_query_distribution.lock().await;
        assert_eq!(dist.get("ds-abc123"), Some(&2));
        assert_eq!(dist.get("ds-def456"), Some(&1));
    }
}
