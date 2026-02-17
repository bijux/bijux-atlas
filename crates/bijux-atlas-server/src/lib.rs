#![forbid(unsafe_code)]

use async_trait::async_trait;
use axum::body::Body;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, HeaderValue, Request, StatusCode, Uri};
use axum::middleware::{from_fn, from_fn_with_state, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use bijux_atlas_api::params::parse_list_genes_params_with_limit;
use bijux_atlas_api::{ApiError, ApiErrorCode};
use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{artifact_paths, ArtifactManifest, Catalog, DatasetId};
use bijux_atlas_query::{
    classify_query, decode_cursor, encode_cursor, query_genes, CursorPayload, GeneFields,
    GeneFilter, GeneQueryRequest, OrderMode, QueryClass, QueryLimits, RegionFilter,
    TranscriptFilter, TranscriptQueryRequest,
};
use hmac::{Hmac, Mac};
use rusqlite::{Connection, OpenFlags};
use sha2::Sha256;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, OwnedSemaphorePermit, RwLock, Semaphore};
use tokio::time::timeout;
use tracing::{error, info, warn};

mod cache;
mod config;
mod dataset_shards;
mod http;
mod middleware;
mod store;
mod store_resilience;
mod telemetry;

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

#[derive(Debug, Clone)]
pub struct DatasetCacheConfig {
    pub disk_root: PathBuf,
    pub max_disk_bytes: u64,
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
    pub registry_ttl: Duration,
    pub registry_freeze_mode: bool,
}

impl Default for DatasetCacheConfig {
    fn default() -> Self {
        Self {
            disk_root: PathBuf::from("artifacts/server-cache"),
            max_disk_bytes: 4 * 1024 * 1024 * 1024,
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
    pub store_retry_budget_exhausted_total: AtomicU64,
    pub store_download_ttfb_ns: Mutex<Vec<u64>>,
    pub store_download_bytes_total: AtomicU64,
    pub store_download_retry_total: AtomicU64,
    pub store_error_checksum_total: AtomicU64,
    pub store_error_timeout_total: AtomicU64,
    pub store_error_network_total: AtomicU64,
    pub store_error_other_total: AtomicU64,
    pub verify_marker_fast_path_hits: AtomicU64,
    pub verify_full_hash_checks: AtomicU64,
    pub cheap_queries_served_while_overloaded_total: AtomicU64,
    pub disk_io_latency_ns: Mutex<Vec<u64>>,
    pub fs_space_pressure_events_total: AtomicU64,
    pub registry_invalidation_events_total: AtomicU64,
}

#[derive(Default)]
struct RequestMetrics {
    counts: Mutex<HashMap<(String, u16), u64>>,
    latency_ns: Mutex<HashMap<String, Vec<u64>>>,
    sqlite_latency_ns: Mutex<HashMap<String, Vec<u64>>>,
    stage_latency_ns: Mutex<HashMap<String, Vec<u64>>>,
    heavy_latency_recent_ns: Mutex<VecDeque<u64>>,
    exemplars: Mutex<HashMap<(String, u16), (String, u128)>>,
}

impl RequestMetrics {
    pub(crate) async fn observe_request(&self, route: &str, status: StatusCode, latency: Duration) {
        self.observe_request_with_trace(route, status, latency, None)
            .await;
    }

    pub(crate) async fn observe_request_with_trace(
        &self,
        route: &str,
        status: StatusCode,
        latency: Duration,
        trace_id: Option<&str>,
    ) {
        let mut counts = self.counts.lock().await;
        *counts
            .entry((route.to_string(), status.as_u16()))
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
                (route.to_string(), status.as_u16()),
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
}

fn chrono_like_unix_millis() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis())
}

fn parse_dataset_from_uri(uri: &Uri) -> Option<DatasetId> {
    let path = uri.path();
    let mut release: Option<String> = None;
    let mut species: Option<String> = None;
    let mut assembly: Option<String> = None;

    if let Some(q) = uri.query() {
        for part in q.split('&') {
            let mut kv = part.splitn(2, '=');
            let k = kv.next().unwrap_or_default();
            let v = kv.next().unwrap_or_default();
            match k {
                "release" => release = Some(v.to_string()),
                "species" => species = Some(v.to_string()),
                "assembly" => assembly = Some(v.to_string()),
                _ => {}
            }
        }
    }

    if release.is_none() || species.is_none() || assembly.is_none() {
        let seg: Vec<&str> = path.split('/').collect();
        if seg.len() >= 8 && seg.get(1) == Some(&"v1") && seg.get(2) == Some(&"releases") {
            release = seg.get(3).map(|x| (*x).to_string());
            if seg.get(4) == Some(&"species") {
                species = seg.get(5).map(|x| (*x).to_string());
            }
            if seg.get(6) == Some(&"assemblies") {
                assembly = seg.get(7).map(|x| (*x).to_string());
            }
        }
    }

    DatasetId::new(
        release.as_deref().unwrap_or_default(),
        species.as_deref().unwrap_or_default(),
        assembly.as_deref().unwrap_or_default(),
    )
    .ok()
}

async fn provenance_headers_middleware(req: Request<Body>, next: Next) -> Response {
    let dataset = parse_dataset_from_uri(req.uri());
    let mut resp = next.run(req).await;

    let (dataset_hash, release) = if let Some(ds) = dataset {
        (
            sha256_hex(ds.canonical_string().as_bytes()),
            ds.release.to_string(),
        )
    } else {
        ("unknown".to_string(), "unknown".to_string())
    };

    if let Ok(v) = HeaderValue::from_str(&dataset_hash) {
        resp.headers_mut().insert("x-atlas-dataset-hash", v);
    }
    if let Ok(v) = HeaderValue::from_str(&release) {
        resp.headers_mut().insert("x-atlas-release", v);
    }
    resp
}

async fn resilience_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let path = req.uri().path().to_string();
    if state.api.emergency_global_breaker
        && path != "/healthz"
        && path != "/healthz/overload"
        && path != "/readyz"
        && path != "/metrics"
    {
        let err = Json(ApiError::new(
            ApiErrorCode::NotReady,
            "emergency global circuit breaker is enabled",
            serde_json::json!({}),
        ));
        return (StatusCode::SERVICE_UNAVAILABLE, err).into_response();
    }
    let mut resp = next.run(req).await;
    if crate::middleware::shedding::overloaded(&state).await {
        resp.headers_mut()
            .insert("x-atlas-system-stress", HeaderValue::from_static("true"));
    }
    resp
}

fn normalized_header_value(headers: &HeaderMap, key: &str, max_len: usize) -> Option<String> {
    let raw = headers.get(key)?.to_str().ok()?.trim();
    if raw.is_empty() || raw.len() > max_len {
        return None;
    }
    Some(raw.to_string())
}

fn normalized_forwarded_for(headers: &HeaderMap) -> Option<String> {
    let raw = headers.get("x-forwarded-for")?.to_str().ok()?;
    let first = raw.split(',').next()?.trim();
    if first.is_empty() || first.len() > 64 {
        return None;
    }
    if first
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b':' || b == b'-')
    {
        Some(first.to_string())
    } else {
        None
    }
}

fn build_hmac_signature(secret: &str, method: &str, uri: &str, ts: &str) -> Option<String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).ok()?;
    let payload = format!("{method}\n{uri}\n{ts}\n");
    mac.update(payload.as_bytes());
    Some(hex::encode(mac.finalize().into_bytes()))
}

async fn security_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let uri_text = req.uri().to_string();
    if uri_text.len() > state.api.max_uri_bytes {
        let err = Json(ApiError::new(
            ApiErrorCode::QueryRejectedByPolicy,
            "request URI too large",
            serde_json::json!({"max_uri_bytes": state.api.max_uri_bytes, "actual": uri_text.len()}),
        ));
        return (StatusCode::URI_TOO_LONG, err).into_response();
    }
    let header_bytes: usize = req
        .headers()
        .iter()
        .map(|(k, v)| k.as_str().len() + v.as_bytes().len())
        .sum();
    if header_bytes > state.api.max_header_bytes {
        let err = Json(ApiError::new(
            ApiErrorCode::QueryRejectedByPolicy,
            "request headers too large",
            serde_json::json!({"max_header_bytes": state.api.max_header_bytes, "actual": header_bytes}),
        ));
        return (StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE, err).into_response();
    }

    let api_key = normalized_header_value(req.headers(), "x-api-key", 256);
    if state.api.require_api_key && api_key.is_none() {
        let err = Json(ApiError::new(
            ApiErrorCode::QueryRejectedByPolicy,
            "api key required",
            serde_json::json!({}),
        ));
        return (StatusCode::UNAUTHORIZED, err).into_response();
    }
    if let Some(key) = &api_key {
        if !state.api.allowed_api_keys.is_empty()
            && !state.api.allowed_api_keys.iter().any(|k| k == key)
        {
            let err = Json(ApiError::new(
                ApiErrorCode::QueryRejectedByPolicy,
                "invalid api key",
                serde_json::json!({}),
            ));
            return (StatusCode::UNAUTHORIZED, err).into_response();
        }
    }

    if let Some(secret) = &state.api.hmac_secret {
        let ts = normalized_header_value(req.headers(), "x-bijux-timestamp", 64);
        let sig = normalized_header_value(req.headers(), "x-bijux-signature", 128);
        if state.api.hmac_required && (ts.is_none() || sig.is_none()) {
            let err = Json(ApiError::new(
                ApiErrorCode::QueryRejectedByPolicy,
                "missing required HMAC headers",
                serde_json::json!({}),
            ));
            return (StatusCode::UNAUTHORIZED, err).into_response();
        }
        if let (Some(ts_value), Some(sig_value)) = (ts, sig) {
            let now = chrono_like_unix_millis() / 1000;
            let Some(parsed_ts) = ts_value.parse::<u128>().ok() else {
                let err = Json(ApiError::new(
                    ApiErrorCode::QueryRejectedByPolicy,
                    "invalid hmac timestamp",
                    serde_json::json!({}),
                ));
                return (StatusCode::UNAUTHORIZED, err).into_response();
            };
            let skew = now.abs_diff(parsed_ts);
            if skew > state.api.hmac_max_skew_secs as u128 {
                let err = Json(ApiError::new(
                    ApiErrorCode::QueryRejectedByPolicy,
                    "hmac timestamp outside allowed skew",
                    serde_json::json!({"max_skew_secs": state.api.hmac_max_skew_secs}),
                ));
                return (StatusCode::UNAUTHORIZED, err).into_response();
            }
            let method = req.method().as_str();
            let uri = req.uri().path_and_query().map_or("", |pq| pq.as_str());
            if build_hmac_signature(secret, method, uri, &ts_value).as_deref()
                != Some(sig_value.as_str())
            {
                let err = Json(ApiError::new(
                    ApiErrorCode::QueryRejectedByPolicy,
                    "invalid hmac signature",
                    serde_json::json!({}),
                ));
                return (StatusCode::UNAUTHORIZED, err).into_response();
            }
        }
    }

    let started = Instant::now();
    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let request_id =
        normalized_header_value(req.headers(), "x-request-id", 128).unwrap_or_default();
    let client_ip =
        normalized_forwarded_for(req.headers()).unwrap_or_else(|| "unknown".to_string());
    let resp = next.run(req).await;
    if state.api.enable_audit_log {
        info!(
            target: "atlas_audit",
            method = %method,
            path = %path,
            status = resp.status().as_u16(),
            request_id = %request_id,
            client_ip = %client_ip,
            latency_ms = started.elapsed().as_millis() as u64,
            "audit"
        );
    }
    resp
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

pub use config::{ApiConfig, RateLimitConfig};
pub use store::backends::{LocalFsBackend, RetryPolicy, S3LikeBackend};
pub use store::federated::{FederatedBackend, RegistrySource};

#[async_trait]
pub trait DatasetStoreBackend: Send + Sync + 'static {
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
    last_download_latency_ns: u64,
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

impl DatasetCacheManager {
    pub fn new(cfg: DatasetCacheConfig, store: Arc<dyn DatasetStoreBackend>) -> Arc<Self> {
        let max_concurrent_downloads = cfg.max_concurrent_downloads;
        let retry_budget = cfg.store_retry_budget as u64;
        Arc::new(Self {
            global_semaphore: Arc::new(Semaphore::new(cfg.max_total_connections)),
            shard_open_semaphore: Arc::new(Semaphore::new(cfg.max_open_shards_per_pod)),
            cfg,
            store,
            entries: Mutex::new(HashMap::new()),
            inflight: Mutex::new(HashMap::new()),
            breakers: Mutex::new(HashMap::new()),
            quarantine_failures: Mutex::new(HashMap::new()),
            quarantined: Mutex::new(HashSet::new()),
            store_breaker: Mutex::new(StoreBreakerState::default()),
            catalog_cache: Mutex::new(CatalogCache::default()),
            registry_health_cache: RwLock::new(Vec::new()),
            download_semaphore: Arc::new(Semaphore::new(max_concurrent_downloads)),
            retry_budget_remaining: AtomicU64::new(retry_budget),
            dataset_retry_budget: Mutex::new(HashMap::new()),
            metrics: Arc::new(CacheMetrics::default()),
        })
    }

    pub async fn startup_warmup(self: &Arc<Self>) -> Result<(), CacheError> {
        std::fs::create_dir_all(&self.cfg.disk_root).map_err(|e| CacheError(e.to_string()))?;
        let mut warm = self.cfg.startup_warmup.clone();
        warm.sort_by_key(DatasetId::canonical_string);
        warm.dedup();
        let bounded = warm
            .into_iter()
            .take(
                self.cfg
                    .startup_warmup_limit
                    .min(self.cfg.max_dataset_count),
            )
            .collect::<Vec<_>>();
        for ds in &bounded {
            let result = self.ensure_dataset_cached(ds).await;
            if let Err(e) = result {
                if self.cfg.fail_readiness_on_missing_warmup {
                    return Err(CacheError(format!("warmup failed for {:?}: {}", ds, e)));
                }
                error!("warmup error for {:?}: {}", ds, e);
            }
        }
        Ok(())
    }

    pub fn spawn_background_tasks(self: &Arc<Self>) {
        let me = Arc::clone(self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(me.cfg.eviction_check_interval);
            loop {
                interval.tick().await;
                if let Err(e) = me.evict_background().await {
                    error!("eviction error: {e}");
                }
            }
        });
        let me = Arc::clone(self);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(me.cfg.integrity_reverify_interval);
            loop {
                interval.tick().await;
                if let Err(e) = me.reverify_cached_datasets().await {
                    error!("reverify error: {e}");
                }
            }
        });
    }

    pub async fn refresh_catalog(&self) -> Result<(), CacheError> {
        if self.cfg.registry_freeze_mode {
            return Ok(());
        }
        let etag = {
            let cache = self.catalog_cache.lock().await;
            let now = Instant::now();
            if let Some(last) = cache.refreshed_at {
                if now.duration_since(last) < self.cfg.registry_ttl {
                    return Ok(());
                }
            }
            if let Some(until) = cache.breaker_open_until {
                if now < until {
                    return Err(CacheError("catalog circuit breaker open".to_string()));
                }
            }
            if let Some(until) = cache.backoff_until {
                if now < until {
                    return Err(CacheError("catalog backoff active".to_string()));
                }
            }
            cache.etag.clone()
        };
        let fetch_result = self.store.fetch_catalog(etag.as_deref()).await;
        let result = match fetch_result {
            Err(e) => Err(e),
            Ok(CatalogFetch::NotModified) => {
                let mut lock = self.catalog_cache.lock().await;
                lock.consecutive_errors = 0;
                lock.backoff_until = None;
                lock.breaker_open_until = None;
                lock.refreshed_at = Some(Instant::now());
                drop(lock);
                let health = self.store.registry_health().await;
                let mut h = self.registry_health_cache.write().await;
                *h = health;
                Ok(())
            }
            Ok(CatalogFetch::Updated { etag, catalog }) => {
                let epoch_hash = sha256_hex(
                    &serde_json::to_vec(&catalog).map_err(|e| CacheError(e.to_string()))?,
                );
                let old_epoch = self.metrics.catalog_epoch_hash.read().await.clone();
                {
                    let mut lock = self.catalog_cache.lock().await;
                    lock.etag = Some(etag);
                    lock.catalog = Some(catalog);
                    lock.consecutive_errors = 0;
                    lock.backoff_until = None;
                    lock.breaker_open_until = None;
                    lock.refreshed_at = Some(Instant::now());
                }
                {
                    let mut e = self.metrics.catalog_epoch_hash.write().await;
                    *e = epoch_hash.clone();
                }
                if !old_epoch.is_empty() && old_epoch != epoch_hash {
                    self.metrics
                        .registry_invalidation_events_total
                        .fetch_add(1, Ordering::Relaxed);
                }
                let health = self.store.registry_health().await;
                let mut h = self.registry_health_cache.write().await;
                *h = health;
                info!("catalog epoch updated: {epoch_hash}");
                Ok(())
            }
        };

        if let Err(err) = result {
            let mut lock = self.catalog_cache.lock().await;
            lock.consecutive_errors = lock.consecutive_errors.saturating_add(1);
            let backoff_ms = self
                .cfg
                .catalog_backoff_base_ms
                .saturating_mul(lock.consecutive_errors as u64)
                .min(5_000);
            lock.backoff_until = Some(Instant::now() + Duration::from_millis(backoff_ms));
            if lock.consecutive_errors >= self.cfg.catalog_breaker_failure_threshold {
                lock.breaker_open_until =
                    Some(Instant::now() + Duration::from_millis(self.cfg.catalog_breaker_open_ms));
            }
            return Err(err);
        }

        Ok(())
    }

    pub async fn catalog_epoch(&self) -> String {
        self.metrics.catalog_epoch_hash.read().await.clone()
    }

    pub fn cached_only_mode(&self) -> bool {
        self.cfg.cached_only_mode
    }

    pub fn registry_freeze_mode(&self) -> bool {
        self.cfg.registry_freeze_mode
    }

    pub fn registry_ttl_seconds(&self) -> u64 {
        self.cfg.registry_ttl.as_secs()
    }

    pub async fn current_catalog(&self) -> Option<Catalog> {
        self.catalog_cache.lock().await.catalog.clone()
    }

    pub async fn registry_health(&self) -> Vec<RegistrySourceHealth> {
        self.registry_health_cache.read().await.clone()
    }

    pub async fn cached_datasets_debug(&self) -> Vec<(String, u64)> {
        let entries = self.entries.lock().await;
        let mut out: Vec<(String, u64)> = entries
            .iter()
            .map(|(id, e)| {
                (
                    format!("{}/{}/{}", id.release, id.species, id.assembly),
                    e.size_bytes,
                )
            })
            .collect();
        out.sort();
        out
    }

    pub async fn fetch_manifest_summary(
        &self,
        dataset: &DatasetId,
    ) -> Result<ArtifactManifest, CacheError> {
        self.store.fetch_manifest(dataset).await
    }

    pub async fn dataset_health_snapshot(
        &self,
        dataset: &DatasetId,
    ) -> Result<DatasetHealthSnapshot, CacheError> {
        let open_failures = {
            let breakers = self.breakers.lock().await;
            breakers.get(dataset).map_or(0, |b| b.failure_count)
        };
        let quarantined = {
            let q = self.quarantined.lock().await;
            q.contains(dataset)
        };
        let (cached, last_open_seconds_ago, size_bytes) = {
            let entries = self.entries.lock().await;
            if let Some(entry) = entries.get(dataset) {
                (
                    true,
                    Some(entry.last_access.elapsed().as_secs()),
                    Some(entry.size_bytes),
                )
            } else {
                (false, None, None)
            }
        };
        let checksum_verified = if cached {
            self.verify_dataset_integrity_strict(dataset).await?
        } else {
            false
        };
        Ok(DatasetHealthSnapshot {
            cached,
            checksum_verified,
            last_open_seconds_ago,
            size_bytes,
            open_failures,
            quarantined,
        })
    }

    pub async fn ensure_sequence_inputs_cached(
        &self,
        dataset: &DatasetId,
    ) -> Result<(PathBuf, PathBuf), CacheError> {
        self.ensure_dataset_cached(dataset).await?;
        let paths = artifact_paths(Path::new(&self.cfg.disk_root), dataset);
        if paths.fasta.exists() && paths.fai.exists() {
            return Ok((paths.fasta, paths.fai));
        }
        if self.cfg.cached_only_mode {
            return Err(CacheError(
                "sequence inputs missing from cache and cached-only mode is enabled".to_string(),
            ));
        }
        if self.cfg.read_only_fs {
            return Err(CacheError(
                "sequence inputs missing from cache and read-only filesystem mode is enabled"
                    .to_string(),
            ));
        }
        let manifest = self.store.fetch_manifest(dataset).await?;
        let fasta = self.store.fetch_fasta_bytes(dataset).await?;
        let fai = self.store.fetch_fai_bytes(dataset).await?;
        if sha256_hex(&fasta) != manifest.checksums.fasta_sha256 {
            return Err(CacheError("fasta checksum verification failed".to_string()));
        }
        if sha256_hex(&fai) != manifest.checksums.fai_sha256 {
            return Err(CacheError("fai checksum verification failed".to_string()));
        }
        std::fs::create_dir_all(&paths.inputs_dir).map_err(|e| CacheError(e.to_string()))?;
        std::fs::write(&paths.fasta, fasta).map_err(|e| CacheError(e.to_string()))?;
        std::fs::write(&paths.fai, fai).map_err(|e| CacheError(e.to_string()))?;
        Ok((paths.fasta, paths.fai))
    }

    pub async fn ensure_release_gene_index_cached(
        &self,
        dataset: &DatasetId,
    ) -> Result<PathBuf, CacheError> {
        self.ensure_dataset_cached(dataset).await?;
        let paths = artifact_paths(Path::new(&self.cfg.disk_root), dataset);
        if paths.release_gene_index.exists() {
            return Ok(paths.release_gene_index);
        }
        if self.cfg.cached_only_mode {
            return Err(CacheError(
                "release gene index missing from cache and cached-only mode is enabled".to_string(),
            ));
        }
        if self.cfg.read_only_fs {
            return Err(CacheError(
                "release gene index missing from cache and read-only filesystem mode is enabled"
                    .to_string(),
            ));
        }
        let bytes = self.store.fetch_release_gene_index_bytes(dataset).await?;
        std::fs::create_dir_all(&paths.derived_dir).map_err(|e| CacheError(e.to_string()))?;
        std::fs::write(&paths.release_gene_index, bytes).map_err(|e| CacheError(e.to_string()))?;
        Ok(paths.release_gene_index)
    }

    pub async fn open_dataset_connection(
        &self,
        dataset: &DatasetId,
    ) -> Result<DatasetConnection, CacheError> {
        info!(dataset = ?dataset, "dataset open start");
        let open_started = Instant::now();
        self.check_quarantine(dataset).await?;
        self.ensure_dataset_cached(dataset).await?;

        self.check_breaker(dataset).await?;

        let (sqlite_path, dataset_sem, query_sem) = {
            let mut entries = self.entries.lock().await;
            let entry = entries
                .get_mut(dataset)
                .ok_or_else(|| CacheError("dataset cache entry missing".to_string()))?;
            entry.last_access = Instant::now();
            (
                entry.sqlite_path.clone(),
                Arc::clone(&entry.dataset_semaphore),
                Arc::clone(&entry.query_semaphore),
            )
        };

        let global_permit = self
            .global_semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| CacheError(e.to_string()))?;
        let dataset_permit = dataset_sem
            .acquire_owned()
            .await
            .map_err(|e| CacheError(e.to_string()))?;
        let query_permit = query_sem
            .acquire_owned()
            .await
            .map_err(|e| CacheError(e.to_string()))?;

        let open = timeout(self.cfg.dataset_open_timeout, async move {
            tokio::task::spawn_blocking(move || {
                Connection::open_with_flags(
                    sqlite_path,
                    OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
                )
            })
            .await
            .map_err(|e| CacheError(e.to_string()))?
            .map_err(|e| CacheError(e.to_string()))
        })
        .await;

        match open {
            Ok(Ok(conn)) => {
                let pragma_sql = format!(
                    "PRAGMA query_only=ON; PRAGMA journal_mode=OFF; PRAGMA synchronous=OFF; PRAGMA temp_store=MEMORY; PRAGMA cache_size=-{}; PRAGMA mmap_size={};",
                    self.cfg.sqlite_pragma_cache_kib, self.cfg.sqlite_pragma_mmap_bytes
                );
                conn.set_prepared_statement_cache_capacity(128);
                prime_prepared_statements(&conn);
                let _ = conn.execute_batch(&pragma_sql);
                self.reset_breaker(dataset).await;
                self.metrics
                    .store_open_latency_ns
                    .lock()
                    .await
                    .push(open_started.elapsed().as_nanos() as u64);
                Ok(DatasetConnection {
                    conn,
                    _global_permit: global_permit,
                    _dataset_permit: dataset_permit,
                    _query_permit: query_permit,
                })
            }
            Ok(Err(e)) => {
                self.record_open_failure(dataset).await;
                self.metrics
                    .store_open_failures
                    .fetch_add(1, Ordering::Relaxed);
                Err(e)
            }
            Err(_) => {
                self.record_open_failure(dataset).await;
                self.metrics
                    .store_open_failures
                    .fetch_add(1, Ordering::Relaxed);
                Err(CacheError("dataset open timeout".to_string()))
            }
        }
    }

    async fn ensure_dataset_cached(&self, dataset: &DatasetId) -> Result<(), CacheError> {
        self.check_quarantine(dataset).await?;
        if self.is_cached_and_verified(dataset).await? {
            self.metrics
                .dataset_hits
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return Ok(());
        }
        self.metrics
            .dataset_misses
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        if self.cfg.cached_only_mode {
            return Err(CacheError(
                "dataset missing from cache and cached-only mode is enabled".to_string(),
            ));
        }
        if self.cfg.read_only_fs {
            return Err(CacheError(
                "dataset missing from cache and read-only filesystem mode is enabled".to_string(),
            ));
        }

        let lock = {
            let mut map = self.inflight.lock().await;
            Arc::clone(
                map.entry(dataset.clone())
                    .or_insert_with(|| Arc::new(Mutex::new(()))),
            )
        };

        let _guard = lock.lock().await;

        if self.is_cached_and_verified(dataset).await? {
            return Ok(());
        }

        self.check_store_breaker().await?;
        let remaining = self.retry_budget_remaining.load(Ordering::Relaxed);
        if remaining == 0 {
            self.metrics
                .store_retry_budget_exhausted_total
                .fetch_add(1, Ordering::Relaxed);
            return Err(CacheError(
                "store retry budget exhausted; refusing download".to_string(),
            ));
        }
        let mut dataset_budget = self.dataset_retry_budget.lock().await;
        let per_dataset_remaining = dataset_budget
            .entry(dataset.clone())
            .or_insert(self.cfg.store_retry_budget);
        if *per_dataset_remaining == 0 {
            self.metrics
                .store_retry_budget_exhausted_total
                .fetch_add(1, Ordering::Relaxed);
            return Err(CacheError(
                "dataset retry budget exhausted; refusing download".to_string(),
            ));
        }
        if *per_dataset_remaining < self.cfg.store_retry_budget {
            self.metrics
                .store_download_retry_total
                .fetch_add(1, Ordering::Relaxed);
        }
        *per_dataset_remaining = per_dataset_remaining.saturating_sub(1);
        drop(dataset_budget);

        info!(dataset = ?dataset, "dataset download path");
        let started = Instant::now();
        info!("dataset download start {:?}", dataset);
        let _download_permit = self
            .download_semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|e| CacheError(e.to_string()))?;
        let manifest = match self.store.fetch_manifest(dataset).await {
            Ok(v) => v,
            Err(e) => {
                self.record_store_download_failure(&e.to_string()).await;
                return Err(e);
            }
        };
        self.metrics
            .store_download_ttfb_ns
            .lock()
            .await
            .push(started.elapsed().as_nanos() as u64);
        let sqlite = match self.store.fetch_sqlite_bytes(dataset).await {
            Ok(v) => v,
            Err(e) => {
                self.record_store_download_failure(&e.to_string()).await;
                return Err(e);
            }
        };
        let release_gene_index = self
            .store
            .fetch_release_gene_index_bytes(dataset)
            .await
            .ok();
        let sqlite_hash = sha256_hex(&sqlite);
        if sqlite_hash != manifest.checksums.sqlite_sha256 {
            error!("dataset verify failed {:?}", dataset);
            self.metrics
                .store_download_failures
                .fetch_add(1, Ordering::Relaxed);
            self.record_store_download_failure("checksum verification failed")
                .await;
            return Err(CacheError(
                "sqlite checksum verification failed".to_string(),
            ));
        }

        let paths = artifact_paths(Path::new(&self.cfg.disk_root), dataset);
        std::fs::create_dir_all(&paths.derived_dir).map_err(|e| CacheError(e.to_string()))?;

        let tmp_dir = self.cfg.disk_root.join(".tmp-atlas-download");
        std::fs::create_dir_all(&tmp_dir).map_err(|e| CacheError(e.to_string()))?;

        let tmp_sqlite = tmp_dir.join("gene_summary.sqlite.tmp");
        std::fs::write(&tmp_sqlite, &sqlite).map_err(|e| CacheError(e.to_string()))?;
        std::fs::rename(&tmp_sqlite, &paths.sqlite).map_err(|e| CacheError(e.to_string()))?;
        if let Some(index_bytes) = release_gene_index {
            std::fs::write(&paths.release_gene_index, index_bytes)
                .map_err(|e| CacheError(e.to_string()))?;
        }

        let manifest_bytes =
            serde_json::to_vec(&manifest).map_err(|e| CacheError(e.to_string()))?;
        std::fs::write(&paths.manifest, manifest_bytes).map_err(|e| CacheError(e.to_string()))?;

        let marker = format!(
            "{}:{}",
            manifest.checksums.sqlite_sha256, manifest.db_schema_version
        );
        std::fs::write(paths.derived_dir.join(".verified"), marker.as_bytes())
            .map_err(|e| CacheError(e.to_string()))?;

        let size_bytes = std::fs::metadata(&paths.sqlite)
            .map_err(|e| CacheError(e.to_string()))?
            .len();
        let (shard_sqlite_paths, shard_by_seqid) =
            dataset_shards::load_shard_catalog(&paths.derived_dir)?;
        let download_latency_ns = started.elapsed().as_nanos() as u64;

        {
            let mut entries = self.entries.lock().await;
            entries.insert(
                dataset.clone(),
                DatasetEntry {
                    sqlite_path: paths.sqlite,
                    shard_sqlite_paths,
                    shard_by_seqid,
                    last_access: Instant::now(),
                    size_bytes,
                    last_download_latency_ns: download_latency_ns,
                    dataset_semaphore: Arc::new(Semaphore::new(
                        self.cfg.max_connections_per_dataset,
                    )),
                    query_semaphore: Arc::new(Semaphore::new(self.cfg.max_connections_per_dataset)),
                },
            );
            self.metrics
                .dataset_count
                .store(entries.len() as u64, std::sync::atomic::Ordering::Relaxed);
            let usage = entries.values().map(|e| e.size_bytes).sum::<u64>();
            self.metrics
                .disk_usage_bytes
                .store(usage, std::sync::atomic::Ordering::Relaxed);
        }

        self.metrics
            .store_download_latency_ns
            .lock()
            .await
            .push(download_latency_ns);
        self.metrics
            .store_download_bytes_total
            .fetch_add(size_bytes, Ordering::Relaxed);
        self.retry_budget_remaining
            .store(self.cfg.store_retry_budget as u64, Ordering::Relaxed);
        let mut dataset_budget = self.dataset_retry_budget.lock().await;
        dataset_budget.insert(dataset.clone(), self.cfg.store_retry_budget);
        self.reset_store_breaker().await;
        info!("dataset download complete {:?}", dataset);
        Ok(())
    }

    async fn is_cached_and_verified(&self, dataset: &DatasetId) -> Result<bool, CacheError> {
        let paths = artifact_paths(Path::new(&self.cfg.disk_root), dataset);
        if !paths.sqlite.exists() || !paths.manifest.exists() {
            return Ok(false);
        }

        let manifest_raw = std::fs::read(&paths.manifest).map_err(|e| CacheError(e.to_string()))?;
        let manifest: ArtifactManifest =
            serde_json::from_slice(&manifest_raw).map_err(|e| CacheError(e.to_string()))?;

        let marker_expected = format!(
            "{}:{}",
            manifest.checksums.sqlite_sha256, manifest.db_schema_version
        );
        let marker_path = paths.derived_dir.join(".verified");
        let marker_ok = marker_path.exists()
            && std::fs::read_to_string(&marker_path)
                .map(|s| s == marker_expected)
                .unwrap_or(false);

        if marker_ok {
            self.metrics
                .verify_marker_fast_path_hits
                .fetch_add(1, Ordering::Relaxed);
            let (shard_sqlite_paths, shard_by_seqid) =
                dataset_shards::load_shard_catalog(&paths.derived_dir)?;
            let mut entries = self.entries.lock().await;
            entries
                .entry(dataset.clone())
                .or_insert_with(|| DatasetEntry {
                    sqlite_path: paths.sqlite.clone(),
                    shard_sqlite_paths,
                    shard_by_seqid,
                    last_access: Instant::now(),
                    size_bytes: std::fs::metadata(&paths.sqlite)
                        .map(|m| m.len())
                        .unwrap_or(0),
                    last_download_latency_ns: 1_000_000,
                    dataset_semaphore: Arc::new(Semaphore::new(
                        self.cfg.max_connections_per_dataset,
                    )),
                    query_semaphore: Arc::new(Semaphore::new(self.cfg.max_connections_per_dataset)),
                });
            return Ok(true);
        }

        self.metrics
            .verify_full_hash_checks
            .fetch_add(1, Ordering::Relaxed);
        let sqlite_hash =
            sha256_hex(&std::fs::read(&paths.sqlite).map_err(|e| CacheError(e.to_string()))?);
        if sqlite_hash == manifest.checksums.sqlite_sha256 {
            std::fs::write(marker_path, marker_expected.as_bytes())
                .map_err(|e| CacheError(e.to_string()))?;
            let (shard_sqlite_paths, shard_by_seqid) =
                dataset_shards::load_shard_catalog(&paths.derived_dir)?;
            let mut entries = self.entries.lock().await;
            entries.insert(
                dataset.clone(),
                DatasetEntry {
                    sqlite_path: paths.sqlite,
                    shard_sqlite_paths,
                    shard_by_seqid,
                    last_access: Instant::now(),
                    size_bytes: std::fs::metadata(paths.derived_dir.join("gene_summary.sqlite"))
                        .map(|m| m.len())
                        .unwrap_or(0),
                    last_download_latency_ns: 1_000_000,
                    dataset_semaphore: Arc::new(Semaphore::new(
                        self.cfg.max_connections_per_dataset,
                    )),
                    query_semaphore: Arc::new(Semaphore::new(self.cfg.max_connections_per_dataset)),
                },
            );
            return Ok(true);
        }
        Ok(false)
    }

    async fn evict_background(&self) -> Result<(), CacheError> {
        let disk_io_started = Instant::now();
        let now = Instant::now();
        let mut entries = self.entries.lock().await;

        let mut victims: Vec<DatasetId> = entries
            .iter()
            .filter_map(|(id, e)| {
                if self.cfg.pinned_datasets.contains(id) {
                    return None;
                }
                if now.duration_since(e.last_access) > self.cfg.idle_ttl {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .collect();

        let mut total_size: u64 = entries.values().map(|e| e.size_bytes).sum();
        if entries.len() > self.cfg.max_dataset_count || total_size > self.cfg.max_disk_bytes {
            let mut ranked: Vec<(DatasetId, f64)> = entries
                .iter()
                .filter(|(id, _)| !self.cfg.pinned_datasets.contains(*id))
                .map(|(id, e)| {
                    let age = now.duration_since(e.last_access).as_secs_f64().max(1.0);
                    let redownload_cost = (e.last_download_latency_ns as f64).max(1.0);
                    let score = age * (e.size_bytes as f64) / redownload_cost;
                    (id.clone(), score)
                })
                .collect();
            ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
            for (id, _) in ranked {
                if entries.len() <= self.cfg.max_dataset_count
                    && total_size <= self.cfg.max_disk_bytes
                {
                    break;
                }
                victims.push(id.clone());
                if let Some(e) = entries.get(&id) {
                    total_size = total_size.saturating_sub(e.size_bytes);
                }
            }
        }

        victims.sort();
        victims.dedup();
        for id in victims {
            if let Some(entry) = entries.remove(&id) {
                let _ = std::fs::remove_file(&entry.sqlite_path);
                for shard in &entry.shard_sqlite_paths {
                    let _ = std::fs::remove_file(shard);
                }
                let _ = std::fs::remove_file(
                    entry
                        .sqlite_path
                        .parent()
                        .unwrap_or_else(|| Path::new("."))
                        .join("manifest.json"),
                );
                let _ = std::fs::remove_file(
                    entry
                        .sqlite_path
                        .parent()
                        .unwrap_or_else(|| Path::new("."))
                        .join(".verified"),
                );
                info!("dataset evicted {:?}", id);
            }
        }

        self.metrics
            .dataset_count
            .store(entries.len() as u64, std::sync::atomic::Ordering::Relaxed);
        self.metrics.disk_usage_bytes.store(
            entries.values().map(|e| e.size_bytes).sum::<u64>(),
            std::sync::atomic::Ordering::Relaxed,
        );
        let usage = self.metrics.disk_usage_bytes.load(Ordering::Relaxed);
        if self.cfg.max_disk_bytes > 0 && usage.saturating_mul(100) / self.cfg.max_disk_bytes >= 90
        {
            self.metrics
                .fs_space_pressure_events_total
                .fetch_add(1, Ordering::Relaxed);
        }
        self.metrics
            .disk_io_latency_ns
            .lock()
            .await
            .push(disk_io_started.elapsed().as_nanos() as u64);

        Ok(())
    }

    async fn reverify_cached_datasets(&self) -> Result<(), CacheError> {
        let datasets: Vec<DatasetId> = {
            let entries = self.entries.lock().await;
            entries.keys().cloned().collect()
        };
        for dataset in datasets {
            if !self.verify_dataset_integrity_strict(&dataset).await? {
                warn!(dataset = ?dataset, "cached dataset failed re-verification");
                self.record_corruption_failure(&dataset).await;
                let mut entries = self.entries.lock().await;
                if let Some(entry) = entries.remove(&dataset) {
                    let _ = std::fs::remove_file(&entry.sqlite_path);
                    for shard in &entry.shard_sqlite_paths {
                        let _ = std::fs::remove_file(shard);
                    }
                }
            }
        }
        Ok(())
    }

    async fn check_quarantine(&self, dataset: &DatasetId) -> Result<(), CacheError> {
        let quarantined = self.quarantined.lock().await;
        if quarantined.contains(dataset) {
            return Err(CacheError("dataset is quarantined".to_string()));
        }
        Ok(())
    }

    async fn record_corruption_failure(&self, dataset: &DatasetId) {
        let mut failures = self.quarantine_failures.lock().await;
        let count = failures.entry(dataset.clone()).or_insert(0);
        *count += 1;
        if self.cfg.quarantine_after_corruption_failures > 0
            && *count >= self.cfg.quarantine_after_corruption_failures
        {
            drop(failures);
            self.quarantined.lock().await.insert(dataset.clone());
        }
    }

    async fn verify_dataset_integrity_strict(
        &self,
        dataset: &DatasetId,
    ) -> Result<bool, CacheError> {
        let paths = artifact_paths(Path::new(&self.cfg.disk_root), dataset);
        if !paths.sqlite.exists() || !paths.manifest.exists() {
            return Ok(false);
        }
        let manifest_raw = std::fs::read(&paths.manifest).map_err(|e| CacheError(e.to_string()))?;
        let manifest: ArtifactManifest =
            serde_json::from_slice(&manifest_raw).map_err(|e| CacheError(e.to_string()))?;
        self.metrics
            .verify_full_hash_checks
            .fetch_add(1, Ordering::Relaxed);
        let sqlite_hash =
            sha256_hex(&std::fs::read(&paths.sqlite).map_err(|e| CacheError(e.to_string()))?);
        Ok(sqlite_hash == manifest.checksums.sqlite_sha256)
    }

    async fn check_breaker(&self, dataset: &DatasetId) -> Result<(), CacheError> {
        let mut lock = self.breakers.lock().await;
        let state = lock.entry(dataset.clone()).or_default();
        if let Some(until) = state.open_until {
            if Instant::now() < until {
                return Err(CacheError("dataset circuit breaker open".to_string()));
            }
        }
        Ok(())
    }

    async fn record_open_failure(&self, dataset: &DatasetId) {
        let mut lock = self.breakers.lock().await;
        let state = lock.entry(dataset.clone()).or_default();
        state.failure_count += 1;
        if state.failure_count >= self.cfg.breaker_failure_threshold {
            state.open_until = Some(Instant::now() + self.cfg.breaker_open_duration);
        }
    }

    async fn reset_breaker(&self, dataset: &DatasetId) {
        let mut lock = self.breakers.lock().await;
        let state = lock.entry(dataset.clone()).or_default();
        state.failure_count = 0;
        state.open_until = None;
    }
}

fn prime_prepared_statements(conn: &Connection) {
    let hot_sql = [
        "SELECT gene_id, name, seqid, start, end, biotype, transcript_count, sequence_length FROM gene_summary WHERE gene_id = ?1 ORDER BY gene_id LIMIT ?2",
        "SELECT gene_id, name, seqid, start, end, biotype, transcript_count, sequence_length FROM gene_summary WHERE biotype = ?1 ORDER BY gene_id LIMIT ?2",
        "SELECT g.gene_id, g.name, g.seqid, g.start, g.end, g.biotype, g.transcript_count, g.sequence_length FROM gene_summary g JOIN gene_summary_rtree r ON r.gene_rowid = g.id WHERE g.seqid = ?1 AND r.start <= ?2 AND r.end >= ?3 ORDER BY g.seqid, g.start, g.gene_id LIMIT ?4",
        "SELECT transcript_id, parent_gene_id, transcript_type, biotype, seqid, start, end, exon_count, total_exon_span, cds_present FROM transcript_summary WHERE transcript_id=?1 LIMIT 1",
        "SELECT transcript_id, parent_gene_id, transcript_type, biotype, seqid, start, end, exon_count, total_exon_span, cds_present FROM transcript_summary WHERE parent_gene_id = ? ORDER BY seqid ASC, start ASC, transcript_id ASC LIMIT ?",
        "SELECT seqid,start,end FROM gene_summary WHERE gene_id = ?1 LIMIT 1",
    ];
    for sql in hot_sql {
        let _ = conn.prepare_cached(sql);
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
    pub(crate) class_cheap: Arc<Semaphore>,
    pub(crate) class_medium: Arc<Semaphore>,
    pub(crate) class_heavy: Arc<Semaphore>,
    pub(crate) heavy_workers: Arc<Semaphore>,
    pub(crate) metrics: Arc<RequestMetrics>,
    pub(crate) request_id_seed: Arc<AtomicU64>,
    pub(crate) coalescer: Arc<cache::coalesce::QueryCoalescer>,
    pub(crate) hot_query_cache: Arc<Mutex<cache::hot::HotQueryCache>>,
    pub(crate) redis_backend: Option<Arc<RedisBackend>>,
    pub(crate) queued_requests: Arc<AtomicU64>,
}

impl AppState {
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
        let redis_policy = telemetry::redis_backend::RedisPolicy {
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
            metrics: Arc::new(RequestMetrics::default()),
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
            api,
            limits,
        }
    }

    pub fn begin_shutdown_drain_heavy(&self) {
        self.class_heavy.close();
        self.heavy_workers.close();
    }
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(http::handlers::landing_handler))
        .route("/healthz", get(http::handlers::healthz_handler))
        .route(
            "/healthz/overload",
            get(http::handlers::overload_health_handler),
        )
        .route("/readyz", get(http::handlers::readyz_handler))
        .route("/metrics", get(http::handlers::metrics_handler))
        .route("/v1/version", get(http::handlers::version_handler))
        .route("/v1/datasets", get(http::handlers::datasets_handler))
        .route(
            "/v1/releases/:release/species/:species/assemblies/:assembly",
            get(http::handlers::release_dataset_handler),
        )
        .route("/v1/genes", get(http::handlers::genes_handler))
        .route("/v1/genes/count", get(http::handlers::genes_count_handler))
        .route("/v1/diff/genes", get(http::diff::diff_genes_handler))
        .route("/v1/diff/region", get(http::diff::diff_region_handler))
        .route(
            "/v1/sequence/region",
            get(http::sequence::sequence_region_handler),
        )
        .route(
            "/v1/genes/:gene_id/sequence",
            get(http::sequence::gene_sequence_handler),
        )
        .route(
            "/v1/genes/:gene_id/transcripts",
            get(http::handlers::gene_transcripts_handler),
        )
        .route(
            "/v1/transcripts/:tx_id",
            get(http::handlers::transcript_summary_handler),
        )
        .route(
            "/debug/datasets",
            get(http::handlers::debug_datasets_handler),
        )
        .route(
            "/debug/dataset-health",
            get(http::handlers::dataset_health_handler),
        )
        .route(
            "/debug/registry-health",
            get(http::handlers::registry_health_handler),
        )
        .layer(from_fn_with_state(state.clone(), cors_middleware))
        .layer(from_fn_with_state(state.clone(), security_middleware))
        .layer(from_fn_with_state(state.clone(), resilience_middleware))
        .layer(from_fn(provenance_headers_middleware))
        .layer(DefaultBodyLimit::max(state.api.max_body_bytes))
        .with_state(state)
}

pub use store::fake::FakeStore;

#[cfg(test)]
mod cache_manager_tests;
#[cfg(test)]
mod registry_tests;

#[cfg(test)]
mod bulkhead_tests {
    use super::*;

    #[tokio::test]
    async fn heavy_bulkhead_saturation_does_not_block_cheap_permits() {
        let store = Arc::new(FakeStore::default());
        let cache = DatasetCacheManager::new(DatasetCacheConfig::default(), store);
        let api = ApiConfig {
            concurrency_cheap: 2,
            concurrency_heavy: 1,
            ..ApiConfig::default()
        };
        let state = AppState::with_config(cache, api, QueryLimits::default());

        let heavy = state
            .class_heavy
            .clone()
            .try_acquire_owned()
            .expect("heavy permit");
        let cheap = state
            .class_cheap
            .clone()
            .try_acquire_owned()
            .expect("cheap should remain available");

        drop((heavy, cheap));
    }
}
