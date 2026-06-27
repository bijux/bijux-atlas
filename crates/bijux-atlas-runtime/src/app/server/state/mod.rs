// SPDX-License-Identifier: Apache-2.0

use crate::adapters::outbound::redis::RedisBackend;
use crate::adapters::outbound::telemetry::rate_limiter::RateLimiter;
use crate::app::cache::{CacheError, RegistrySourceHealth};
use crate::app::ports::{CatalogFetch, DatasetStoreBackend};
use crate::app::server::cache;
use crate::domain::cluster::config::load_cluster_config_from_path;
use crate::domain::cluster::membership::MembershipPolicy;
use crate::domain::cluster::membership::MembershipRegistry;
use crate::domain::cluster::replication::ReplicaRegistry;
use crate::domain::cluster::replication::{
    ConsistencyGuarantee, ConsistencyLevel, ReplicationPolicy,
};
use crate::domain::cluster::resilience::FailureRecoveryRegistry;
use crate::domain::cluster::resilience::{
    FailureDetectionPolicy, RecoveryPolicy, ResilienceGuarantees,
};
use crate::domain::cluster::sharding::ShardRegistry;
use crate::domain::sha256_hex;
use crate::runtime::config::{ApiConfig, DatasetCacheConfig};
use crate::StatusCode;
use crate::{route_sli_class, unix_time_millis};
use bijux_atlas_model::dataset::{artifact_paths, ArtifactManifest, Catalog, DatasetId};
use bijux_atlas_query::QueryLimits;
use rusqlite::Connection;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, OwnedMutexGuard, OwnedSemaphorePermit, RwLock, Semaphore};
use tokio::time::timeout;
use tracing::{error, info, warn, Instrument};

pub(crate) mod cache_runtime;

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
pub struct RequestMetrics {
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
    pub async fn observe_request(&self, route: &str, status: StatusCode, latency: Duration) {
        self.observe_request_with_trace_and_method(route, "GET", status, latency, None)
            .await;
    }

    pub async fn observe_request_with_method(
        &self,
        route: &str,
        method: &str,
        status: StatusCode,
        latency: Duration,
    ) {
        self.observe_request_with_trace_and_method(route, method, status, latency, None)
            .await;
    }

    pub async fn observe_request_with_trace(
        &self,
        route: &str,
        status: StatusCode,
        latency: Duration,
        trace_id: Option<&str>,
    ) {
        self.observe_request_with_trace_and_method(route, "GET", status, latency, trace_id)
            .await;
    }

    pub async fn observe_request_with_trace_and_method(
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
                (id.to_string(), unix_time_millis()),
            );
        }
    }

    pub async fn observe_sqlite_query(&self, query_type: &str, latency: Duration) {
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

    pub async fn observe_stage(&self, stage: &str, latency: Duration) {
        let mut m = self.stage_latency_ns.lock().await;
        m.entry(stage.to_string())
            .or_insert_with(Vec::new)
            .push(latency.as_nanos() as u64);
    }

    pub async fn observe_request_size(&self, route: &str, bytes: usize) {
        let mut m = self.request_size_bytes.lock().await;
        m.entry(route.to_string())
            .or_insert_with(Vec::new)
            .push(bytes as u64);
    }

    pub async fn observe_response_size(&self, route: &str, bytes: usize) {
        let mut m = self.response_size_bytes.lock().await;
        m.entry(route.to_string())
            .or_insert_with(Vec::new)
            .push(bytes as u64);
    }

    pub async fn observe_query_row_count(&self, route: &str, rows: usize) {
        let mut m = self.query_row_count.lock().await;
        m.entry(route.to_string())
            .or_insert_with(Vec::new)
            .push(rows as u64);
    }

    pub fn observe_query_cache_hit(&self) {
        self.query_cache_hits_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn observe_query_cache_miss(&self) {
        self.query_cache_misses_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn observe_slow_query(&self) {
        self.slow_queries_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn slow_queries_total(&self) -> u64 {
        self.slow_queries_total.load(Ordering::Relaxed)
    }

    pub async fn observe_dataset_query(&self, dataset_key: &str) {
        let mut m = self.dataset_query_distribution.lock().await;
        *m.entry(dataset_key.to_string()).or_insert(0) += 1;
    }

    pub async fn dataset_query_distribution_snapshot(&self) -> HashMap<String, u64> {
        self.dataset_query_distribution.lock().await.clone()
    }

    pub async fn query_planner_stats_snapshot(&self) -> serde_json::Value {
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

    pub async fn runtime_stats_snapshot(&self) -> serde_json::Value {
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

    pub async fn should_shed_heavy(&self, min_samples: usize, threshold_ms: u64) -> bool {
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

    pub async fn observe_client_fingerprint(&self, client_type: &str, user_agent_family: &str) {
        let mut counts = self.client_fingerprint_counts.lock().await;
        *counts
            .entry((client_type.to_string(), user_agent_family.to_string()))
            .or_insert(0) += 1;
    }
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
pub struct RequestQueueGuard {
    counter: Arc<AtomicU64>,
}

impl Drop for RequestQueueGuard {
    fn drop(&mut self) {
        self.counter.fetch_sub(1, Ordering::Relaxed);
    }
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

impl AppState {
    fn init_request_metrics() -> Arc<RequestMetrics> {
        Arc::new(RequestMetrics::default())
    }

    fn derive_runtime_policy_hash(api: &ApiConfig, limits: &QueryLimits) -> String {
        let payload = serde_json::json!({
            "api": api,
            "limits": limits
        });
        match crate::compat::core::stable_json_bytes(&payload) {
            Ok(bytes) => sha256_hex(&bytes),
            Err(_) => sha256_hex(b"runtime-policy-hash-fallback"),
        }
    }

    fn init_membership_registry() -> MembershipRegistry {
        let policy = std::env::var("ATLAS_CLUSTER_CONFIG_PATH")
            .ok()
            .and_then(|cluster_path| {
                load_cluster_config_from_path(std::path::Path::new(&cluster_path))
                    .ok()
                    .map(|cfg| MembershipPolicy {
                        heartbeat_interval_ms: cfg.health.heartbeat_interval_ms,
                        node_timeout_ms: cfg.health.node_timeout_ms,
                    })
            })
            .unwrap_or(MembershipPolicy {
                heartbeat_interval_ms: 1_000,
                node_timeout_ms: 5_000,
            });
        MembershipRegistry::new(policy)
    }

    fn init_shard_registry() -> ShardRegistry {
        ShardRegistry::new()
    }

    fn init_replica_registry() -> ReplicaRegistry {
        ReplicaRegistry::new(
            ReplicationPolicy {
                replication_factor: 2,
                primary_required: true,
                max_replication_lag_ms: 2_000,
            },
            ConsistencyGuarantee {
                read_consistency: ConsistencyLevel::Quorum,
                write_consistency: ConsistencyLevel::Quorum,
            },
        )
    }

    fn init_resilience_registry() -> FailureRecoveryRegistry {
        FailureRecoveryRegistry::new(
            FailureDetectionPolicy {
                node_timeout_ms: 5_000,
                replica_lag_threshold_ms: 2_000,
                recovery_retry_budget: 3,
            },
            RecoveryPolicy {
                auto_recovery_enabled: true,
                shard_failover_enabled: true,
                replica_failover_enabled: true,
                rebalance_after_recovery: true,
            },
            ResilienceGuarantees {
                failover_within_ms: 10_000,
                diagnostics_available: true,
                event_logging_required: true,
            },
        )
    }

    #[must_use]
    pub fn new(cache: Arc<DatasetCacheManager>) -> Self {
        Self::with_config(cache, ApiConfig::default(), QueryLimits::default())
    }

    #[must_use]
    pub fn with_config(
        cache: Arc<DatasetCacheManager>,
        api: ApiConfig,
        limits: QueryLimits,
    ) -> Self {
        let runtime_policy_hash = Arc::new(Self::derive_runtime_policy_hash(&api, &limits));
        let redis_policy = crate::adapters::outbound::redis::RedisPolicy {
            timeout: Duration::from_millis(api.redis_timeout_ms),
            retry_attempts: api.redis_retry_attempts.max(1),
            breaker_failure_threshold: api.redis_breaker_failure_threshold,
            breaker_open_duration: Duration::from_millis(api.redis_breaker_open_ms),
            max_key_bytes: api.redis_cache_max_key_bytes,
            max_cardinality: api.redis_cache_max_cardinality,
            max_ttl_secs: api.redis_cache_ttl_max_secs,
        };
        Self {
            cache,
            ready: Arc::new(AtomicBool::new(true)),
            class_cheap: Arc::new(Semaphore::new(api.concurrency_cheap)),
            class_medium: Arc::new(Semaphore::new(api.concurrency_medium)),
            class_heavy: Arc::new(Semaphore::new(api.concurrency_heavy)),
            heavy_workers: Arc::new(Semaphore::new(api.heavy_worker_pool_size)),
            ip_limiter: Arc::new(RateLimiter::new(
                if api.enable_redis_rate_limit {
                    api.redis_url.as_deref().and_then(|u| {
                        RedisBackend::new(u, &api.redis_prefix, redis_policy.clone()).ok()
                    })
                } else {
                    None
                },
                "ip",
            )),
            sequence_ip_limiter: Arc::new(RateLimiter::new(
                if api.enable_redis_rate_limit {
                    api.redis_url.as_deref().and_then(|u| {
                        RedisBackend::new(u, &api.redis_prefix, redis_policy.clone()).ok()
                    })
                } else {
                    None
                },
                "sequence_ip",
            )),
            api_key_limiter: Arc::new(RateLimiter::new(
                if api.enable_redis_rate_limit {
                    api.redis_url.as_deref().and_then(|u| {
                        RedisBackend::new(u, &api.redis_prefix, redis_policy.clone()).ok()
                    })
                } else {
                    None
                },
                "api_key",
            )),
            metrics: Self::init_request_metrics(),
            request_id_seed: Arc::new(AtomicU64::new(1)),
            accepting_requests: Arc::new(AtomicBool::new(true)),
            coalescer: Arc::new(cache::coalesce::QueryCoalescer::new()),
            hot_query_cache: Arc::new(Mutex::new(cache::hot::HotQueryCache::new(
                Duration::from_secs(2),
                512,
            ))),
            redis_backend: api
                .redis_url
                .as_deref()
                .and_then(|u| RedisBackend::new(u, &api.redis_prefix, redis_policy).ok())
                .map(Arc::new),
            queued_requests: Arc::new(AtomicU64::new(0)),
            membership: Arc::new(Mutex::new(Self::init_membership_registry())),
            shard_registry: Arc::new(Mutex::new(Self::init_shard_registry())),
            replica_registry: Arc::new(Mutex::new(Self::init_replica_registry())),
            resilience_registry: Arc::new(Mutex::new(Self::init_resilience_registry())),
            runtime_policy_hash,
            runtime_policy_mode: Arc::new(crate::runtime::config::default_runtime_policy_mode()),
            api,
            limits,
        }
    }

    pub fn begin_shutdown_drain_heavy(&self) {
        self.class_heavy.close();
        self.heavy_workers.close();
    }

    pub fn next_request_id(&self) -> String {
        let id = self.request_id_seed.fetch_add(1, Ordering::Relaxed);
        format!("req-{id:016x}")
    }

    pub fn increment_queued_requests(&self) -> u64 {
        self.queued_requests
            .fetch_add(1, Ordering::Relaxed)
            .saturating_add(1)
    }

    pub fn decrement_queued_requests(&self) {
        self.queued_requests.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn request_queue_guard(&self) -> RequestQueueGuard {
        RequestQueueGuard {
            counter: Arc::clone(&self.queued_requests),
        }
    }

    pub fn try_acquire_query_class_permit(
        &self,
        class: bijux_atlas_query::QueryClass,
    ) -> Result<OwnedSemaphorePermit, tokio::sync::TryAcquireError> {
        let limiter = match class {
            bijux_atlas_query::QueryClass::Cheap => Arc::clone(&self.class_cheap),
            bijux_atlas_query::QueryClass::Medium => Arc::clone(&self.class_medium),
            bijux_atlas_query::QueryClass::Heavy => Arc::clone(&self.class_heavy),
            _ => Arc::clone(&self.class_heavy),
        };
        limiter.try_acquire_owned()
    }

    pub fn try_acquire_heavy_worker_permit(
        &self,
    ) -> Result<OwnedSemaphorePermit, tokio::sync::TryAcquireError> {
        Arc::clone(&self.heavy_workers).try_acquire_owned()
    }

    pub fn heavy_available_permits(&self) -> usize {
        self.class_heavy.available_permits()
    }

    pub fn metrics(&self) -> Arc<RequestMetrics> {
        Arc::clone(&self.metrics)
    }

    pub async fn allow_ip_with_factor(&self, key: &str, factor: f64) -> bool {
        self.ip_limiter
            .allow_with_factor(key, &self.api.rate_limit_per_ip, factor)
            .await
    }

    pub async fn allow_sequence_ip_with_factor(&self, key: &str, factor: f64) -> bool {
        self.sequence_ip_limiter
            .allow_with_factor(key, &self.api.sequence_rate_limit_per_ip, factor)
            .await
    }

    pub async fn allow_api_key_with_factor(&self, key: &str, factor: f64) -> bool {
        self.api_key_limiter
            .allow_with_factor(key, &self.api.rate_limit_per_api_key, factor)
            .await
    }

    pub async fn acquire_coalesced_query(&self, key: &str) -> OwnedMutexGuard<()> {
        self.coalescer.acquire(key).await
    }

    pub async fn hot_query_get(&self, key: &str) -> Option<cache::hot::HotEntry> {
        let mut cache = self.hot_query_cache.lock().await;
        cache.get(key)
    }

    pub async fn hot_query_insert(&self, key: String, entry: cache::hot::HotEntry) {
        let mut cache = self.hot_query_cache.lock().await;
        cache.insert(key, entry);
    }

    pub async fn acquire_redis_fill_lock(&self, key: &str) -> Option<OwnedMutexGuard<()>> {
        match &self.redis_backend {
            Some(redis) => Some(redis.acquire_fill_lock(key).await),
            None => None,
        }
    }

    pub async fn redis_gene_cache_get(&self, key: &str) -> Result<Option<Vec<u8>>, String> {
        match &self.redis_backend {
            Some(redis) => redis.get_gene_cache(key).await,
            None => Ok(None),
        }
    }

    pub async fn redis_gene_cache_set(
        &self,
        key: &str,
        value: &[u8],
        ttl_secs: usize,
    ) -> Result<(), String> {
        match &self.redis_backend {
            Some(redis) => redis.set_gene_cache(key, value, ttl_secs).await,
            None => Ok(()),
        }
    }

    pub fn membership_registry(&self) -> Arc<Mutex<MembershipRegistry>> {
        Arc::clone(&self.membership)
    }

    pub fn shard_registry(&self) -> Arc<Mutex<ShardRegistry>> {
        Arc::clone(&self.shard_registry)
    }

    pub fn replica_registry(&self) -> Arc<Mutex<ReplicaRegistry>> {
        Arc::clone(&self.replica_registry)
    }

    pub fn resilience_registry(&self) -> Arc<Mutex<FailureRecoveryRegistry>> {
        Arc::clone(&self.resilience_registry)
    }

    pub async fn record_policy_violation(&self, policy: &str) {
        self.cache
            .metrics
            .policy_violations_total
            .fetch_add(1, Ordering::Relaxed);
        let mut by = self.cache.metrics.policy_violations_by_policy.lock().await;
        *by.entry(policy.to_string()).or_insert(0) += 1;
    }

    pub async fn increment_policy_violation_bucket(&self, key: &str) -> u64 {
        let mut by = self.cache.metrics.policy_violations_by_policy.lock().await;
        let count = by.entry(key.to_string()).or_insert(0);
        *count += 1;
        *count
    }

    pub async fn record_invariant_violation(&self, invariant: &str) {
        let mut by = self.cache.metrics.invariant_violations_by_name.lock().await;
        *by.entry(invariant.to_string()).or_insert(0) += 1;
    }
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
