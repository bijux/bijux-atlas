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
    pub policy_violations_total: AtomicU64,
    pub policy_violations_by_policy: Mutex<HashMap<String, u64>>,
    pub shed_total_by_reason: Mutex<HashMap<String, u64>>,
}

#[derive(Default)]
struct RequestMetrics {
    counts: Mutex<HashMap<(String, u16), u64>>,
    latency_ns: Mutex<HashMap<String, Vec<u64>>>,
    sqlite_latency_ns: Mutex<HashMap<String, Vec<u64>>>,
    stage_latency_ns: Mutex<HashMap<String, Vec<u64>>>,
    request_size_bytes: Mutex<HashMap<String, Vec<u64>>>,
    response_size_bytes: Mutex<HashMap<String, Vec<u64>>>,
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

async fn provenance_headers_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let dataset = parse_dataset_from_uri(req.uri());
    let mut resp = next.run(req).await;

    let (dataset_hash, release, artifact_hash) = if let Some(ds) = dataset {
        let artifact_hash = state
            .cache
            .fetch_manifest_summary(&ds)
            .await
            .ok()
            .map(|m| m.dataset_signature_sha256);
        (
            sha256_hex(ds.canonical_string().as_bytes()),
            ds.release.to_string(),
            artifact_hash,
        )
    } else {
        ("unknown".to_string(), "unknown".to_string(), None)
    };

    if let Ok(v) = HeaderValue::from_str(&dataset_hash) {
        resp.headers_mut().insert("x-atlas-dataset-hash", v);
    }
    if let Some(artifact_hash) = artifact_hash {
        if let Ok(v) = HeaderValue::from_str(&artifact_hash) {
            resp.headers_mut().insert("x-atlas-artifact-hash", v);
        }
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
            "req-unknown",
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

async fn record_policy_violation(state: &AppState, policy: &str) {
    state
        .cache
        .metrics
        .policy_violations_total
        .fetch_add(1, Ordering::Relaxed);
    let mut by = state.cache.metrics.policy_violations_by_policy.lock().await;
    *by.entry(policy.to_string()).or_insert(0) += 1;
}

pub(crate) async fn record_shed_reason(state: &AppState, reason: &str) {
    let mut by = state.cache.metrics.shed_total_by_reason.lock().await;
    *by.entry(reason.to_string()).or_insert(0) += 1;
}

async fn security_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let uri_text = req.uri().to_string();
    let route = req.uri().path().to_string();
    if uri_text.len() > state.api.max_uri_bytes {
        record_policy_violation(&state, "uri_bytes").await;
        let err = Json(ApiError::new(
            ApiErrorCode::QueryRejectedByPolicy,
            "request URI too large",
            serde_json::json!({"max_uri_bytes": state.api.max_uri_bytes, "actual": uri_text.len()}),
            "req-unknown",
        ));
        return (StatusCode::BAD_REQUEST, err).into_response();
    }
    let header_bytes: usize = req
        .headers()
        .iter()
        .map(|(k, v)| k.as_str().len() + v.as_bytes().len())
        .sum();
    state
        .metrics
        .observe_request_size(&route, uri_text.len().saturating_add(header_bytes))
        .await;
    if header_bytes > state.api.max_header_bytes {
        record_policy_violation(&state, "header_bytes").await;
        let err = Json(ApiError::new(
            ApiErrorCode::QueryRejectedByPolicy,
            "request headers too large",
            serde_json::json!({"max_header_bytes": state.api.max_header_bytes, "actual": header_bytes}),
            "req-unknown",
        ));
        return (StatusCode::BAD_REQUEST, err).into_response();
    }

    let api_key = normalized_header_value(req.headers(), "x-api-key", 256);
    if state.api.require_api_key && api_key.is_none() {
        record_policy_violation(&state, "api_key_required").await;
        let err = Json(ApiError::new(
            ApiErrorCode::QueryRejectedByPolicy,
            "api key required",
            serde_json::json!({}),
            "req-unknown",
        ));
        return (StatusCode::UNAUTHORIZED, err).into_response();
    }
    if let Some(key) = &api_key {
        if !state.api.allowed_api_keys.is_empty()
            && !state.api.allowed_api_keys.iter().any(|k| k == key)
        {
            record_policy_violation(&state, "api_key_invalid").await;
            let err = Json(ApiError::new(
                ApiErrorCode::QueryRejectedByPolicy,
                "invalid api key",
                serde_json::json!({}),
                "req-unknown",
            ));
            return (StatusCode::UNAUTHORIZED, err).into_response();
        }
    }

    if let Some(secret) = &state.api.hmac_secret {
        let ts = normalized_header_value(req.headers(), "x-bijux-timestamp", 64);
        let sig = normalized_header_value(req.headers(), "x-bijux-signature", 128);
        if state.api.hmac_required && (ts.is_none() || sig.is_none()) {
            record_policy_violation(&state, "hmac_missing_headers").await;
            let err = Json(ApiError::new(
                ApiErrorCode::QueryRejectedByPolicy,
                "missing required HMAC headers",
                serde_json::json!({}),
                "req-unknown",
            ));
            return (StatusCode::UNAUTHORIZED, err).into_response();
        }
        if let (Some(ts_value), Some(sig_value)) = (ts, sig) {
            let now = chrono_like_unix_millis() / 1000;
            let Some(parsed_ts) = ts_value.parse::<u128>().ok() else {
                record_policy_violation(&state, "hmac_invalid_timestamp").await;
                let err = Json(ApiError::new(
                    ApiErrorCode::QueryRejectedByPolicy,
                    "invalid hmac timestamp",
                    serde_json::json!({}),
                    "req-unknown",
                ));
                return (StatusCode::UNAUTHORIZED, err).into_response();
            };
            let skew = now.abs_diff(parsed_ts);
            if skew > state.api.hmac_max_skew_secs as u128 {
                record_policy_violation(&state, "hmac_skew").await;
                let err = Json(ApiError::new(
                    ApiErrorCode::QueryRejectedByPolicy,
                    "hmac timestamp outside allowed skew",
                    serde_json::json!({"max_skew_secs": state.api.hmac_max_skew_secs}),
                    "req-unknown",
                ));
                return (StatusCode::UNAUTHORIZED, err).into_response();
            }
            let method = req.method().as_str();
            let uri = req.uri().path_and_query().map_or("", |pq| pq.as_str());
            if build_hmac_signature(secret, method, uri, &ts_value).as_deref()
                != Some(sig_value.as_str())
            {
                record_policy_violation(&state, "hmac_signature").await;
                let err = Json(ApiError::new(
                    ApiErrorCode::QueryRejectedByPolicy,
                    "invalid hmac signature",
                    serde_json::json!({}),
                    "req-unknown",
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
