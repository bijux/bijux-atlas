#![forbid(unsafe_code)]

use async_trait::async_trait;
use axum::body::Body;
use axum::extract::{DefaultBodyLimit, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use bijux_atlas_api::params::parse_list_genes_params_with_limit;
use bijux_atlas_api::{ApiError, ApiErrorCode};
use bijux_atlas_core::sha256_hex;
use bijux_atlas_model::{artifact_paths, ArtifactManifest, Catalog, DatasetId};
use bijux_atlas_query::{
    classify_query, query_genes, GeneFields, GeneFilter, GeneQueryRequest, QueryClass, QueryLimits,
    RegionFilter,
};
use rusqlite::{Connection, OpenFlags};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, OwnedSemaphorePermit, RwLock, Semaphore};
use tokio::time::timeout;
use tracing::{error, info, info_span, warn};

mod api_config;
mod fake_store;

pub const CRATE_NAME: &str = "bijux-atlas-server";

#[derive(Debug)]
pub struct CacheError(pub String);

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for CacheError {}

fn percentile_ns(values: &[u64], pct: f64) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut v = values.to_vec();
    v.sort_unstable();
    let idx = ((v.len() as f64 - 1.0) * pct).round() as usize;
    v[idx]
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
    pub fail_readiness_on_missing_warmup: bool,
    pub max_connections_per_dataset: usize,
    pub max_total_connections: usize,
    pub dataset_open_timeout: Duration,
    pub breaker_failure_threshold: u32,
    pub breaker_open_duration: Duration,
    pub eviction_check_interval: Duration,
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
            fail_readiness_on_missing_warmup: false,
            max_connections_per_dataset: 8,
            max_total_connections: 64,
            dataset_open_timeout: Duration::from_secs(3),
            breaker_failure_threshold: 3,
            breaker_open_duration: Duration::from_secs(30),
            eviction_check_interval: Duration::from_secs(30),
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
}

#[derive(Default)]
struct RequestMetrics {
    counts: Mutex<HashMap<(String, u16), u64>>,
    latency_ns: Mutex<HashMap<String, Vec<u64>>>,
    sqlite_latency_ns: Mutex<HashMap<String, Vec<u64>>>,
    exemplars: Mutex<HashMap<(String, u16), (String, u128)>>,
}

impl RequestMetrics {
    async fn observe_request(&self, route: &str, status: StatusCode, latency: Duration) {
        self.observe_request_with_trace(route, status, latency, None)
            .await;
    }

    async fn observe_request_with_trace(
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

    async fn observe_sqlite_query(&self, query_type: &str, latency: Duration) {
        let mut q = self.sqlite_latency_ns.lock().await;
        q.entry(query_type.to_string())
            .or_insert_with(Vec::new)
            .push(latency.as_nanos() as u64);
    }
}

fn chrono_like_unix_millis() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis())
}

pub use api_config::{ApiConfig, RateLimitConfig};

pub struct LocalFsBackend {
    root: PathBuf,
}

impl LocalFsBackend {
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

#[async_trait]
impl DatasetStoreBackend for LocalFsBackend {
    async fn fetch_catalog(&self, if_none_match: Option<&str>) -> Result<CatalogFetch, CacheError> {
        let path = self.root.join("catalog.json");
        let bytes = fs::read(&path).map_err(|e| CacheError(format!("catalog read failed: {e}")))?;
        let etag = sha256_hex(&bytes);
        if if_none_match == Some(etag.as_str()) {
            return Ok(CatalogFetch::NotModified);
        }
        let catalog: Catalog = serde_json::from_slice(&bytes)
            .map_err(|e| CacheError(format!("catalog parse failed: {e}")))?;
        Ok(CatalogFetch::Updated { etag, catalog })
    }

    async fn fetch_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, CacheError> {
        let path = artifact_paths(Path::new(&self.root), dataset).manifest;
        let bytes = fs::read(path).map_err(|e| CacheError(format!("manifest read failed: {e}")))?;
        let manifest: ArtifactManifest = serde_json::from_slice(&bytes)
            .map_err(|e| CacheError(format!("manifest parse failed: {e}")))?;
        manifest
            .validate_strict()
            .map_err(|e| CacheError(format!("manifest validation failed: {e}")))?;
        Ok(manifest)
    }

    async fn fetch_sqlite_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, CacheError> {
        let path = artifact_paths(Path::new(&self.root), dataset).sqlite;
        fs::read(path).map_err(|e| CacheError(format!("sqlite read failed: {e}")))
    }
}

#[async_trait]
pub trait DatasetStoreBackend: Send + Sync + 'static {
    async fn fetch_catalog(&self, if_none_match: Option<&str>) -> Result<CatalogFetch, CacheError>;
    async fn fetch_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, CacheError>;
    async fn fetch_sqlite_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, CacheError>;
}

pub enum CatalogFetch {
    NotModified,
    Updated { etag: String, catalog: Catalog },
}

struct DatasetEntry {
    sqlite_path: PathBuf,
    last_access: Instant,
    size_bytes: u64,
    dataset_semaphore: Arc<Semaphore>,
}

#[derive(Default)]
struct CatalogCache {
    etag: Option<String>,
    catalog: Option<Catalog>,
}

#[derive(Default)]
struct BreakerState {
    failure_count: u32,
    open_until: Option<Instant>,
}

#[derive(Debug, Clone)]
struct Bucket {
    tokens: f64,
    last_refill: Instant,
}

#[derive(Default)]
struct RateLimiter {
    buckets: Mutex<HashMap<String, Bucket>>,
}

impl RateLimiter {
    async fn allow(&self, key: &str, cfg: &RateLimitConfig) -> bool {
        let now = Instant::now();
        let mut lock = self.buckets.lock().await;
        let bucket = lock.entry(key.to_string()).or_insert_with(|| Bucket {
            tokens: cfg.capacity,
            last_refill: now,
        });
        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.last_refill = now;
        bucket.tokens = (bucket.tokens + (elapsed * cfg.refill_per_sec)).min(cfg.capacity);
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

pub struct DatasetConnection {
    pub conn: Connection,
    _global_permit: OwnedSemaphorePermit,
    _dataset_permit: OwnedSemaphorePermit,
}

pub struct DatasetCacheManager {
    cfg: DatasetCacheConfig,
    store: Arc<dyn DatasetStoreBackend>,
    entries: Mutex<HashMap<DatasetId, DatasetEntry>>,
    inflight: Mutex<HashMap<DatasetId, Arc<Mutex<()>>>>,
    breakers: Mutex<HashMap<DatasetId, BreakerState>>,
    catalog_cache: Mutex<CatalogCache>,
    global_semaphore: Arc<Semaphore>,
    pub metrics: Arc<CacheMetrics>,
}

impl DatasetCacheManager {
    pub fn new(cfg: DatasetCacheConfig, store: Arc<dyn DatasetStoreBackend>) -> Arc<Self> {
        Arc::new(Self {
            global_semaphore: Arc::new(Semaphore::new(cfg.max_total_connections)),
            cfg,
            store,
            entries: Mutex::new(HashMap::new()),
            inflight: Mutex::new(HashMap::new()),
            breakers: Mutex::new(HashMap::new()),
            catalog_cache: Mutex::new(CatalogCache::default()),
            metrics: Arc::new(CacheMetrics::default()),
        })
    }

    pub async fn startup_warmup(self: &Arc<Self>) -> Result<(), CacheError> {
        std::fs::create_dir_all(&self.cfg.disk_root).map_err(|e| CacheError(e.to_string()))?;
        for ds in &self.cfg.startup_warmup {
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
    }

    pub async fn refresh_catalog(&self) -> Result<(), CacheError> {
        let etag = { self.catalog_cache.lock().await.etag.clone() };
        match self.store.fetch_catalog(etag.as_deref()).await? {
            CatalogFetch::NotModified => Ok(()),
            CatalogFetch::Updated { etag, catalog } => {
                let epoch_hash = sha256_hex(
                    &serde_json::to_vec(&catalog).map_err(|e| CacheError(e.to_string()))?,
                );
                {
                    let mut lock = self.catalog_cache.lock().await;
                    lock.etag = Some(etag);
                    lock.catalog = Some(catalog);
                }
                {
                    let mut e = self.metrics.catalog_epoch_hash.write().await;
                    *e = epoch_hash.clone();
                }
                info!("catalog epoch updated: {epoch_hash}");
                Ok(())
            }
        }
    }

    pub async fn catalog_epoch(&self) -> String {
        self.metrics.catalog_epoch_hash.read().await.clone()
    }

    pub async fn current_catalog(&self) -> Option<Catalog> {
        self.catalog_cache.lock().await.catalog.clone()
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

    pub async fn open_dataset_connection(
        &self,
        dataset: &DatasetId,
    ) -> Result<DatasetConnection, CacheError> {
        info!(dataset = ?dataset, "dataset open start");
        let open_started = Instant::now();
        self.ensure_dataset_cached(dataset).await?;

        self.check_breaker(dataset).await?;

        let (sqlite_path, dataset_sem) = {
            let mut entries = self.entries.lock().await;
            let entry = entries
                .get_mut(dataset)
                .ok_or_else(|| CacheError("dataset cache entry missing".to_string()))?;
            entry.last_access = Instant::now();
            (
                entry.sqlite_path.clone(),
                Arc::clone(&entry.dataset_semaphore),
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

        info!(dataset = ?dataset, "dataset download path");
        let started = Instant::now();
        info!("dataset download start {:?}", dataset);
        let manifest = self.store.fetch_manifest(dataset).await.inspect_err(|_| {
            self.metrics
                .store_download_failures
                .fetch_add(1, Ordering::Relaxed);
        })?;
        let sqlite = self
            .store
            .fetch_sqlite_bytes(dataset)
            .await
            .inspect_err(|_| {
                self.metrics
                    .store_download_failures
                    .fetch_add(1, Ordering::Relaxed);
            })?;
        let sqlite_hash = sha256_hex(&sqlite);
        if sqlite_hash != manifest.checksums.sqlite_sha256 {
            error!("dataset verify failed {:?}", dataset);
            self.metrics
                .store_download_failures
                .fetch_add(1, Ordering::Relaxed);
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

        {
            let mut entries = self.entries.lock().await;
            entries.insert(
                dataset.clone(),
                DatasetEntry {
                    sqlite_path: paths.sqlite,
                    last_access: Instant::now(),
                    size_bytes,
                    dataset_semaphore: Arc::new(Semaphore::new(
                        self.cfg.max_connections_per_dataset,
                    )),
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
            .push(started.elapsed().as_nanos() as u64);
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
            let mut entries = self.entries.lock().await;
            entries
                .entry(dataset.clone())
                .or_insert_with(|| DatasetEntry {
                    sqlite_path: paths.sqlite.clone(),
                    last_access: Instant::now(),
                    size_bytes: std::fs::metadata(&paths.sqlite)
                        .map(|m| m.len())
                        .unwrap_or(0),
                    dataset_semaphore: Arc::new(Semaphore::new(
                        self.cfg.max_connections_per_dataset,
                    )),
                });
            return Ok(true);
        }

        let sqlite_hash =
            sha256_hex(&std::fs::read(&paths.sqlite).map_err(|e| CacheError(e.to_string()))?);
        if sqlite_hash == manifest.checksums.sqlite_sha256 {
            std::fs::write(marker_path, marker_expected.as_bytes())
                .map_err(|e| CacheError(e.to_string()))?;
            let mut entries = self.entries.lock().await;
            entries.insert(
                dataset.clone(),
                DatasetEntry {
                    sqlite_path: paths.sqlite,
                    last_access: Instant::now(),
                    size_bytes: std::fs::metadata(paths.derived_dir.join("gene_summary.sqlite"))
                        .map(|m| m.len())
                        .unwrap_or(0),
                    dataset_semaphore: Arc::new(Semaphore::new(
                        self.cfg.max_connections_per_dataset,
                    )),
                },
            );
            return Ok(true);
        }
        Ok(false)
    }

    async fn evict_background(&self) -> Result<(), CacheError> {
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
            let mut by_lru: Vec<(DatasetId, Instant)> = entries
                .iter()
                .filter(|(id, _)| !self.cfg.pinned_datasets.contains(*id))
                .map(|(id, e)| (id.clone(), e.last_access))
                .collect();
            by_lru.sort_by_key(|x| x.1);
            for (id, _) in by_lru {
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

        Ok(())
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

#[derive(Clone)]
pub struct AppState {
    pub cache: Arc<DatasetCacheManager>,
    pub api: ApiConfig,
    pub limits: QueryLimits,
    pub ready: Arc<AtomicBool>,
    ip_limiter: Arc<RateLimiter>,
    api_key_limiter: Arc<RateLimiter>,
    class_cheap: Arc<Semaphore>,
    class_medium: Arc<Semaphore>,
    class_heavy: Arc<Semaphore>,
    metrics: Arc<RequestMetrics>,
    request_id_seed: Arc<AtomicU64>,
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
        Self {
            cache,
            ready: Arc::new(AtomicBool::new(true)),
            class_cheap: Arc::new(Semaphore::new(api.concurrency_cheap)),
            class_medium: Arc::new(Semaphore::new(api.concurrency_medium)),
            class_heavy: Arc::new(Semaphore::new(api.concurrency_heavy)),
            ip_limiter: Arc::new(RateLimiter::default()),
            api_key_limiter: Arc::new(RateLimiter::default()),
            metrics: Arc::new(RequestMetrics::default()),
            request_id_seed: Arc::new(AtomicU64::new(1)),
            api,
            limits,
        }
    }
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(http_handlers::healthz_handler))
        .route("/readyz", get(http_handlers::readyz_handler))
        .route("/metrics", get(http_handlers::metrics_handler))
        .route("/v1/version", get(http_handlers::version_handler))
        .route("/v1/datasets", get(http_handlers::datasets_handler))
        .route("/v1/genes", get(http_handlers::genes_handler))
        .route("/v1/genes/count", get(http_handlers::genes_count_handler))
        .route(
            "/debug/datasets",
            get(http_handlers::debug_datasets_handler),
        )
        .layer(DefaultBodyLimit::max(state.api.max_body_bytes))
        .with_state(state)
}

pub use fake_store::FakeStore;

mod http_handlers;
mod metrics_endpoint;

#[cfg(test)]
mod cache_manager_tests;
