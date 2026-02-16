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
        .route("/healthz", get(healthz_handler))
        .route("/readyz", get(readyz_handler))
        .route("/metrics", get(metrics_handler))
        .route("/v1/datasets", get(datasets_handler))
        .route("/v1/genes", get(genes_handler))
        .route("/v1/genes/count", get(genes_count_handler))
        .route("/debug/datasets", get(debug_datasets_handler))
        .layer(DefaultBodyLimit::max(state.api.max_body_bytes))
        .with_state(state)
}

fn api_error_response(status: StatusCode, err: ApiError) -> Response {
    let body = Json(json!({"error": err}));
    (status, body).into_response()
}

fn error_json(code: ApiErrorCode, message: &str, details: Value) -> ApiError {
    ApiError {
        code,
        message: message.to_string(),
        details,
    }
}

fn normalize_query(params: &HashMap<String, String>) -> String {
    let mut kv: Vec<(&String, &String)> = params.iter().collect();
    kv.sort_by(|a, b| a.0.cmp(b.0).then_with(|| a.1.cmp(b.1)));
    kv.into_iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&")
}

fn if_none_match(headers: &HeaderMap) -> Option<String> {
    headers
        .get("if-none-match")
        .and_then(|v| v.to_str().ok())
        .map(std::string::ToString::to_string)
}

fn put_cache_headers(headers: &mut HeaderMap, ttl: Duration, etag: &str) {
    if let Ok(value) = HeaderValue::from_str(&format!("public, max-age={}", ttl.as_secs())) {
        headers.insert("cache-control", value);
    }
    if let Ok(value) = HeaderValue::from_str(etag) {
        headers.insert("etag", value);
    }
}

fn wants_pretty(params: &HashMap<String, String>) -> bool {
    params
        .get("pretty")
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

fn wants_text(headers: &HeaderMap) -> bool {
    headers
        .get("accept")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.contains("text/plain"))
}

fn make_request_id(state: &AppState) -> String {
    let id = state.request_id_seed.fetch_add(1, Ordering::Relaxed);
    format!("req-{id:016x}")
}

fn with_request_id(mut response: Response, request_id: &str) -> Response {
    if let Ok(v) = HeaderValue::from_str(request_id) {
        response.headers_mut().insert("x-request-id", v);
    }
    response
}

async fn healthz_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let resp = (StatusCode::OK, "ok").into_response();
    state
        .metrics
        .observe_request_with_trace(
            "/healthz",
            StatusCode::OK,
            started.elapsed(),
            Some(&request_id),
        )
        .await;
    with_request_id(resp, &request_id)
}

async fn readyz_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let catalog_ready = if state.api.readiness_requires_catalog {
        state.cache.current_catalog().await.is_some()
    } else {
        true
    };
    if state.ready.load(Ordering::Relaxed) && catalog_ready {
        let resp = (StatusCode::OK, "ready").into_response();
        state
            .metrics
            .observe_request_with_trace(
                "/readyz",
                StatusCode::OK,
                started.elapsed(),
                Some(&request_id),
            )
            .await;
        with_request_id(resp, &request_id)
    } else {
        let resp = (StatusCode::SERVICE_UNAVAILABLE, "not-ready").into_response();
        state
            .metrics
            .observe_request_with_trace(
                "/readyz",
                StatusCode::SERVICE_UNAVAILABLE,
                started.elapsed(),
                Some(&request_id),
            )
            .await;
        with_request_id(resp, &request_id)
    }
}

async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let request_id = make_request_id(&state);
    let started = Instant::now();
    let mut body = format!(
        "atlas_dataset_hits {}\natlas_dataset_misses {}\natlas_dataset_count {}\natlas_dataset_disk_usage_bytes {}\n",
        state.cache.metrics.dataset_hits.load(Ordering::Relaxed),
        state.cache.metrics.dataset_misses.load(Ordering::Relaxed),
        state.cache.metrics.dataset_count.load(Ordering::Relaxed),
        state.cache.metrics.disk_usage_bytes.load(Ordering::Relaxed),
    );
    let open_lat = state
        .cache
        .metrics
        .store_open_latency_ns
        .lock()
        .await
        .clone();
    let download_lat = state
        .cache
        .metrics
        .store_download_latency_ns
        .lock()
        .await
        .clone();
    body.push_str(&format!(
        "atlas_store_open_failure_total {}\natlas_store_download_failure_total {}\n",
        state
            .cache
            .metrics
            .store_open_failures
            .load(Ordering::Relaxed),
        state
            .cache
            .metrics
            .store_download_failures
            .load(Ordering::Relaxed),
    ));
    body.push_str(&format!(
        "atlas_store_open_p95_seconds {:.6}\natlas_store_download_p95_seconds {:.6}\n",
        percentile_ns(&open_lat, 0.95) as f64 / 1_000_000_000.0,
        percentile_ns(&download_lat, 0.95) as f64 / 1_000_000_000.0
    ));

    let req_counts = state.metrics.counts.lock().await.clone();
    let req_exemplars = state.metrics.exemplars.lock().await.clone();
    for ((route, status), count) in req_counts {
        if state.api.enable_exemplars {
            if let Some((trace_id, ts_ms)) = req_exemplars.get(&(route.clone(), status)) {
                body.push_str(&format!(
                    "atlas_http_requests_total{{route=\"{}\",status=\"{}\"}} {} # {{trace_id=\"{}\"}} {}\n",
                    route, status, count, trace_id, ts_ms
                ));
            } else {
                body.push_str(&format!(
                    "atlas_http_requests_total{{route=\"{}\",status=\"{}\"}} {}\n",
                    route, status, count
                ));
            }
        } else {
            body.push_str(&format!(
                "atlas_http_requests_total{{route=\"{}\",status=\"{}\"}} {}\n",
                route, status, count
            ));
        }
    }
    let req_lat = state.metrics.latency_ns.lock().await.clone();
    for (route, vals) in req_lat {
        body.push_str(&format!(
            "atlas_http_request_latency_p95_seconds{{route=\"{}\"}} {:.6}\n",
            route,
            percentile_ns(&vals, 0.95) as f64 / 1_000_000_000.0
        ));
    }
    let sql_lat = state.metrics.sqlite_latency_ns.lock().await.clone();
    for (query_type, vals) in sql_lat {
        body.push_str(&format!(
            "atlas_sqlite_query_latency_p95_seconds{{query_type=\"{}\"}} {:.6}\n",
            query_type,
            percentile_ns(&vals, 0.95) as f64 / 1_000_000_000.0
        ));
    }
    let resp = (StatusCode::OK, body).into_response();
    state
        .metrics
        .observe_request_with_trace(
            "/metrics",
            StatusCode::OK,
            started.elapsed(),
            Some(&request_id),
        )
        .await;
    with_request_id(resp, &request_id)
}

async fn datasets_handler(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let started = Instant::now();
    let request_id = make_request_id(&state);
    info!(request_id = %request_id, route = "/v1/datasets", "request start");
    let _ = state.cache.refresh_catalog().await;
    let catalog = state
        .cache
        .current_catalog()
        .await
        .unwrap_or(Catalog { datasets: vec![] });
    let payload = json!({"datasets": catalog.datasets});
    let etag = format!(
        "\"{}\"",
        sha256_hex(&serde_json::to_vec(&payload).unwrap_or_default())
    );
    if if_none_match(&headers).as_deref() == Some(etag.as_str()) {
        let mut resp = StatusCode::NOT_MODIFIED.into_response();
        put_cache_headers(resp.headers_mut(), state.api.discovery_ttl, &etag);
        state
            .metrics
            .observe_request("/v1/datasets", StatusCode::NOT_MODIFIED, started.elapsed())
            .await;
        return with_request_id(resp, &request_id);
    }
    let mut response = Json(payload).into_response();
    put_cache_headers(response.headers_mut(), state.api.discovery_ttl, &etag);
    state
        .metrics
        .observe_request("/v1/datasets", StatusCode::OK, started.elapsed())
        .await;
    with_request_id(response, &request_id)
}

async fn debug_datasets_handler(State(state): State<AppState>) -> impl IntoResponse {
    let started = Instant::now();
    let request_id = make_request_id(&state);
    if !state.api.enable_debug_datasets {
        let resp = api_error_response(
            StatusCode::NOT_FOUND,
            error_json(
                ApiErrorCode::InvalidQueryParameter,
                "debug endpoint disabled",
                json!({}),
            ),
        );
        state
            .metrics
            .observe_request("/debug/datasets", StatusCode::NOT_FOUND, started.elapsed())
            .await;
        return with_request_id(resp, &request_id);
    }
    let items = state.cache.cached_datasets_debug().await;
    let resp = Json(json!({"datasets": items, "catalog_epoch": state.cache.catalog_epoch().await}))
        .into_response();
    state
        .metrics
        .observe_request("/debug/datasets", StatusCode::OK, started.elapsed())
        .await;
    with_request_id(resp, &request_id)
}

fn parse_fields(fields: Option<Vec<String>>) -> GeneFields {
    let mut out = GeneFields {
        gene_id: false,
        name: false,
        coords: false,
        biotype: false,
        transcript_count: false,
        sequence_length: false,
    };
    if let Some(list) = fields {
        for field in list {
            match field.as_str() {
                "gene_id" => out.gene_id = true,
                "name" => out.name = true,
                "coords" => out.coords = true,
                "biotype" => out.biotype = true,
                "transcript_count" => out.transcript_count = true,
                "sequence_length" => out.sequence_length = true,
                _ => {}
            }
        }
        out
    } else {
        GeneFields::default()
    }
}

fn parse_region(raw: Option<String>) -> Result<Option<RegionFilter>, ApiError> {
    if let Some(value) = raw {
        let (seqid, span) = value
            .split_once(':')
            .ok_or_else(|| ApiError::invalid_param("region", &value))?;
        let (start, end) = span
            .split_once('-')
            .ok_or_else(|| ApiError::invalid_param("region", &value))?;
        let start = start
            .parse::<u64>()
            .map_err(|_| ApiError::invalid_param("region", &value))?;
        let end = end
            .parse::<u64>()
            .map_err(|_| ApiError::invalid_param("region", &value))?;
        return Ok(Some(RegionFilter {
            seqid: seqid.to_string(),
            start,
            end,
        }));
    }
    Ok(None)
}

async fn acquire_class_permit(
    state: &AppState,
    class: QueryClass,
) -> Result<tokio::sync::OwnedSemaphorePermit, ApiError> {
    let sem = match class {
        QueryClass::Cheap => state.class_cheap.clone(),
        QueryClass::Medium => state.class_medium.clone(),
        QueryClass::Heavy => state.class_heavy.clone(),
    };
    sem.try_acquire_owned().map_err(|_| {
        error_json(
            ApiErrorCode::QueryRejectedByPolicy,
            "concurrency limit reached",
            json!({"class": format!("{class:?}")}),
        )
    })
}

async fn genes_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    let started = Instant::now();
    let request_id = make_request_id(&state);
    info!(request_id = %request_id, "request start");
    if let Some(ip) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        if !state
            .ip_limiter
            .allow(ip, &state.api.rate_limit_per_ip)
            .await
        {
            let resp = api_error_response(
                StatusCode::TOO_MANY_REQUESTS,
                error_json(
                    ApiErrorCode::RateLimited,
                    "rate limit exceeded",
                    json!({"scope":"ip"}),
                ),
            );
            state
                .metrics
                .observe_request(
                    "/v1/genes",
                    StatusCode::TOO_MANY_REQUESTS,
                    started.elapsed(),
                )
                .await;
            return with_request_id(resp, &request_id);
        }
    }
    if state.api.enable_api_key_rate_limit {
        if let Some(key) = headers.get("x-api-key").and_then(|v| v.to_str().ok()) {
            if !state
                .api_key_limiter
                .allow(key, &state.api.rate_limit_per_api_key)
                .await
            {
                let resp = api_error_response(
                    StatusCode::TOO_MANY_REQUESTS,
                    error_json(
                        ApiErrorCode::RateLimited,
                        "rate limit exceeded",
                        json!({"scope":"api_key"}),
                    ),
                );
                state
                    .metrics
                    .observe_request(
                        "/v1/genes",
                        StatusCode::TOO_MANY_REQUESTS,
                        started.elapsed(),
                    )
                    .await;
                return with_request_id(resp, &request_id);
            }
        }
    }

    let parse_map: std::collections::BTreeMap<String, String> =
        params.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    let parsed = match parse_list_genes_params_with_limit(&parse_map, 100, state.limits.max_limit) {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(StatusCode::BAD_REQUEST, e);
            state
                .metrics
                .observe_request("/v1/genes", StatusCode::BAD_REQUEST, started.elapsed())
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    let dataset = match DatasetId::new(&parsed.release, &parsed.species, &parsed.assembly) {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(
                StatusCode::BAD_REQUEST,
                ApiError::invalid_param("dataset", &e.to_string()),
            );
            state
                .metrics
                .observe_request("/v1/genes", StatusCode::BAD_REQUEST, started.elapsed())
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    let req = match parse_region(parsed.region) {
        Ok(region) => GeneQueryRequest {
            fields: parse_fields(parsed.fields),
            filter: GeneFilter {
                gene_id: parsed.gene_id,
                name: parsed.name,
                name_prefix: parsed.name_prefix,
                biotype: parsed.biotype,
                region,
            },
            limit: parsed.limit,
            cursor: parsed.cursor,
            allow_full_scan: false,
        },
        Err(e) => {
            let resp = api_error_response(StatusCode::BAD_REQUEST, e);
            state
                .metrics
                .observe_request("/v1/genes", StatusCode::BAD_REQUEST, started.elapsed())
                .await;
            return with_request_id(resp, &request_id);
        }
    };
    let class = classify_query(&req);
    let _class_permit = match acquire_class_permit(&state, class).await {
        Ok(v) => v,
        Err(e) => {
            let resp = api_error_response(StatusCode::TOO_MANY_REQUESTS, e);
            state
                .metrics
                .observe_request(
                    "/v1/genes",
                    StatusCode::TOO_MANY_REQUESTS,
                    started.elapsed(),
                )
                .await;
            return with_request_id(resp, &request_id);
        }
    };

    let normalized = normalize_query(&params);
    let work = async {
        info!(request_id = %request_id, dataset = ?dataset, "dataset resolve");
        let c = state.cache.open_dataset_connection(&dataset).await?;
        let deadline = Instant::now() + state.api.sql_timeout;
        c.conn
            .progress_handler(1_000, Some(move || Instant::now() > deadline));
        let query_started = Instant::now();
        let query_span = info_span!("sqlite_query", class = %format!("{class:?}").to_lowercase());
        let result = query_span.in_scope(|| {
            query_genes(&c.conn, &req, &state.limits, b"atlas-server-cursor-secret")
                .map_err(|e| CacheError(e.to_string()))
        })?;
        let query_elapsed = query_started.elapsed();
        if query_elapsed > state.api.slow_query_threshold {
            warn!(
                request_id = %request_id,
                dataset = %format!("{}/{}/{}", dataset.release, dataset.species, dataset.assembly),
                class = %format!("{class:?}").to_lowercase(),
                normalized_query = %normalize_query(&params),
                "slow query detected"
            );
        }
        c.conn.progress_handler(1_000, None::<fn() -> bool>);
        Ok::<_, CacheError>((result, query_elapsed))
    };

    let result = timeout(state.api.request_timeout, work).await;
    let payload = match result {
        Ok(Ok((resp, query_elapsed))) => {
            state
                .metrics
                .observe_sqlite_query(&format!("{class:?}").to_lowercase(), query_elapsed)
                .await;
            json!({"dataset": dataset, "class": format!("{class:?}").to_lowercase(), "response": resp})
        }
        Ok(Err(err)) => {
            let msg = err.to_string();
            if msg.contains("limit") || msg.contains("span") || msg.contains("scan") {
                let resp = api_error_response(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    error_json(
                        ApiErrorCode::QueryRejectedByPolicy,
                        "query rejected",
                        json!({"message": msg}),
                    ),
                );
                state
                    .metrics
                    .observe_request(
                        "/v1/genes",
                        StatusCode::UNPROCESSABLE_ENTITY,
                        started.elapsed(),
                    )
                    .await;
                return with_request_id(resp, &request_id);
            }
            if req.cursor.is_some() {
                let resp = api_error_response(
                    StatusCode::BAD_REQUEST,
                    error_json(
                        ApiErrorCode::InvalidCursor,
                        "invalid cursor",
                        json!({"message": msg}),
                    ),
                );
                state
                    .metrics
                    .observe_request("/v1/genes", StatusCode::BAD_REQUEST, started.elapsed())
                    .await;
                return with_request_id(resp, &request_id);
            }
            let resp = api_error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                error_json(
                    ApiErrorCode::Internal,
                    "query failed",
                    json!({"message": msg}),
                ),
            );
            state
                .metrics
                .observe_request(
                    "/v1/genes",
                    StatusCode::SERVICE_UNAVAILABLE,
                    started.elapsed(),
                )
                .await;
            return with_request_id(resp, &request_id);
        }
        Err(_) => {
            let resp = api_error_response(
                StatusCode::GATEWAY_TIMEOUT,
                error_json(ApiErrorCode::Timeout, "request timed out", json!({})),
            );
            state
                .metrics
                .observe_request("/v1/genes", StatusCode::GATEWAY_TIMEOUT, started.elapsed())
                .await;
            return with_request_id(resp, &request_id);
        }
    };

    let bytes = info_span!("serialize_response").in_scope(|| {
        if wants_pretty(&params) {
            serde_json::to_vec_pretty(&payload).unwrap_or_default()
        } else {
            serde_json::to_vec(&payload).unwrap_or_default()
        }
    });
    if bytes.len() > state.api.response_max_bytes {
        let resp = api_error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            error_json(
                ApiErrorCode::ResponseTooLarge,
                "response exceeds configured size guard",
                json!({"bytes": bytes.len(), "max": state.api.response_max_bytes}),
            ),
        );
        state
            .metrics
            .observe_request(
                "/v1/genes",
                StatusCode::PAYLOAD_TOO_LARGE,
                started.elapsed(),
            )
            .await;
        return with_request_id(resp, &request_id);
    }

    let etag = format!(
        "\"{}\"",
        sha256_hex(format!("{normalized}|{}", String::from_utf8_lossy(&bytes)).as_bytes())
    );
    if if_none_match(&headers).as_deref() == Some(etag.as_str()) {
        let mut resp = StatusCode::NOT_MODIFIED.into_response();
        put_cache_headers(resp.headers_mut(), state.api.immutable_gene_ttl, &etag);
        state
            .metrics
            .observe_request("/v1/genes", StatusCode::NOT_MODIFIED, started.elapsed())
            .await;
        return with_request_id(resp, &request_id);
    }

    if wants_text(&headers) {
        let text = String::from_utf8_lossy(&bytes).to_string();
        let mut resp = (StatusCode::OK, text).into_response();
        put_cache_headers(resp.headers_mut(), state.api.immutable_gene_ttl, &etag);
        state
            .metrics
            .observe_request("/v1/genes", StatusCode::OK, started.elapsed())
            .await;
        return with_request_id(resp, &request_id);
    }
    let mut resp = Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(bytes))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response());
    resp.headers_mut()
        .insert("content-type", HeaderValue::from_static("application/json"));
    put_cache_headers(resp.headers_mut(), state.api.immutable_gene_ttl, &etag);
    state
        .metrics
        .observe_request("/v1/genes", StatusCode::OK, started.elapsed())
        .await;
    info!(request_id = %request_id, status = 200_u16, "request complete");
    with_request_id(resp, &request_id)
}

async fn genes_count_handler(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HashMap<String, String>>,
) -> Response {
    let started = Instant::now();
    let request_id = make_request_id(&state);
    let release = params.get("release").cloned().unwrap_or_default();
    let species = params.get("species").cloned().unwrap_or_default();
    let assembly = params.get("assembly").cloned().unwrap_or_default();
    let dataset = match DatasetId::new(&release, &species, &assembly) {
        Ok(v) => v,
        Err(e) => {
            let resp = (
                axum::http::StatusCode::BAD_REQUEST,
                Json(json!({"error": e.to_string()})),
            )
                .into_response();
            state
                .metrics
                .observe_request(
                    "/v1/genes/count",
                    StatusCode::BAD_REQUEST,
                    started.elapsed(),
                )
                .await;
            return with_request_id(resp, &request_id);
        }
    };

    match state.cache.open_dataset_connection(&dataset).await {
        Ok(c) => {
            let count: Result<i64, _> =
                c.conn
                    .query_row("SELECT COUNT(*) FROM gene_summary", [], |r| r.get(0));
            match count {
                Ok(v) => {
                    let epoch = state.cache.catalog_epoch().await;
                    let resp = Json(json!({
                        "dataset": format!("{}/{}/{}", release, species, assembly),
                        "gene_count": v,
                        "catalog_epoch": epoch
                    }))
                    .into_response();
                    state
                        .metrics
                        .observe_request("/v1/genes/count", StatusCode::OK, started.elapsed())
                        .await;
                    with_request_id(resp, &request_id)
                }
                Err(e) => {
                    let resp = api_error_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        error_json(
                            ApiErrorCode::Internal,
                            "query failed",
                            json!({"message": e.to_string()}),
                        ),
                    );
                    state
                        .metrics
                        .observe_request(
                            "/v1/genes/count",
                            StatusCode::INTERNAL_SERVER_ERROR,
                            started.elapsed(),
                        )
                        .await;
                    with_request_id(resp, &request_id)
                }
            }
        }
        Err(e) => {
            let resp = api_error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                error_json(
                    ApiErrorCode::NotReady,
                    "dataset unavailable",
                    json!({"message": e.to_string()}),
                ),
            );
            state
                .metrics
                .observe_request(
                    "/v1/genes/count",
                    StatusCode::SERVICE_UNAVAILABLE,
                    started.elapsed(),
                )
                .await;
            with_request_id(resp, &request_id)
        }
    }
}

pub struct FakeStore {
    pub catalog: Mutex<Catalog>,
    pub manifest: Mutex<HashMap<DatasetId, ArtifactManifest>>,
    pub sqlite: Mutex<HashMap<DatasetId, Vec<u8>>>,
    pub fetch_calls: std::sync::atomic::AtomicU64,
    pub etag: Mutex<String>,
    pub slow_read: bool,
}

impl Default for FakeStore {
    fn default() -> Self {
        Self {
            catalog: Mutex::new(Catalog {
                datasets: Vec::new(),
            }),
            manifest: Mutex::new(HashMap::new()),
            sqlite: Mutex::new(HashMap::new()),
            fetch_calls: std::sync::atomic::AtomicU64::new(0),
            etag: Mutex::new(String::new()),
            slow_read: false,
        }
    }
}

#[async_trait]
impl DatasetStoreBackend for FakeStore {
    async fn fetch_catalog(&self, if_none_match: Option<&str>) -> Result<CatalogFetch, CacheError> {
        let etag = self.etag.lock().await.clone();
        if if_none_match == Some(etag.as_str()) {
            return Ok(CatalogFetch::NotModified);
        }
        Ok(CatalogFetch::Updated {
            etag,
            catalog: self.catalog.lock().await.clone(),
        })
    }

    async fn fetch_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, CacheError> {
        self.fetch_calls
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if self.slow_read {
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        self.manifest
            .lock()
            .await
            .get(dataset)
            .cloned()
            .ok_or_else(|| CacheError("manifest missing".to_string()))
    }

    async fn fetch_sqlite_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, CacheError> {
        if self.slow_read {
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        self.sqlite
            .lock()
            .await
            .get(dataset)
            .cloned()
            .ok_or_else(|| CacheError("sqlite missing".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn fixture_sqlite() -> Vec<u8> {
        let dir = tempdir().expect("tempdir");
        let db = dir.path().join("x.sqlite");
        let conn = Connection::open(&db).expect("open sqlite");
        conn.execute_batch(
            "CREATE TABLE gene_summary(id INTEGER PRIMARY KEY, gene_id TEXT, name TEXT, biotype TEXT, seqid TEXT, start INT, end INT, transcript_count INT, sequence_length INT);
             INSERT INTO gene_summary(id,gene_id,name,biotype,seqid,start,end,transcript_count,sequence_length) VALUES (1,'g1','G1','pc','chr1',1,10,1,10);",
        )
        .expect("seed sqlite");
        std::fs::read(db).expect("read sqlite bytes")
    }

    fn mk_dataset() -> (DatasetId, ArtifactManifest, Vec<u8>) {
        let ds = DatasetId::new("110", "homo_sapiens", "GRCh38").expect("dataset id");
        let sqlite = fixture_sqlite();
        let sqlite_sha = sha256_hex(&sqlite);
        let manifest = ArtifactManifest {
            manifest_version: "1".to_string(),
            db_schema_version: "1".to_string(),
            dataset: ds.clone(),
            checksums: bijux_atlas_model::ArtifactChecksums {
                gff3_sha256: "a".repeat(64),
                fasta_sha256: "b".repeat(64),
                fai_sha256: "c".repeat(64),
                sqlite_sha256: sqlite_sha,
            },
            stats: bijux_atlas_model::ManifestStats {
                gene_count: 1,
                transcript_count: 1,
                contig_count: 1,
            },
        };
        (ds, manifest, sqlite)
    }

    #[tokio::test]
    async fn single_flight_download_shared_by_concurrent_calls() {
        let (ds, manifest, sqlite) = mk_dataset();
        let store = Arc::new(FakeStore::default());
        store.manifest.lock().await.insert(ds.clone(), manifest);
        store.sqlite.lock().await.insert(ds.clone(), sqlite);
        *store.etag.lock().await = "v1".to_string();

        let tmp = tempdir().expect("tempdir");
        let cfg = DatasetCacheConfig {
            disk_root: tmp.path().to_path_buf(),
            ..Default::default()
        };
        let mgr = DatasetCacheManager::new(cfg, store.clone());

        let mut joins = Vec::new();
        for _ in 0..8 {
            let m = Arc::clone(&mgr);
            let d = ds.clone();
            joins.push(tokio::spawn(
                async move { m.open_dataset_connection(&d).await },
            ));
        }
        for j in joins {
            j.await.expect("join handle").expect("open connection");
        }

        let calls = store.fetch_calls.load(std::sync::atomic::Ordering::Relaxed);
        assert_eq!(calls, 1, "single-flight should perform one manifest fetch");
    }

    #[tokio::test]
    #[ignore]
    async fn chaos_mode_slow_store_reads_graceful_errors() {
        let (ds, manifest, sqlite) = mk_dataset();
        let store = Arc::new(FakeStore {
            slow_read: true,
            ..Default::default()
        });
        store.manifest.lock().await.insert(ds.clone(), manifest);
        store.sqlite.lock().await.insert(ds.clone(), sqlite);

        let tmp = tempdir().expect("tempdir");
        let cfg = DatasetCacheConfig {
            disk_root: tmp.path().to_path_buf(),
            dataset_open_timeout: Duration::from_millis(1),
            ..Default::default()
        };
        let mgr = DatasetCacheManager::new(cfg, store);
        let result = mgr.open_dataset_connection(&ds).await;
        match result {
            Ok(_) => {}
            Err(err) => {
                assert!(err.to_string().contains("timeout") || err.to_string().contains("missing"))
            }
        }
    }
}
