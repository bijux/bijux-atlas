// SPDX-License-Identifier: Apache-2.0

use crate::catalog::validate_catalog_strict;
use crate::manifest::{verify_expected_sha256, ManifestLock};
use crate::paths::{
    dataset_artifact_paths, manifest_lock_path, publish_lock_path, CATALOG_FILE,
};
#[cfg(feature = "backend-s3")]
use crate::paths::{dataset_key_prefix, dataset_manifest_key, dataset_manifest_lock_key, dataset_sqlite_key};
#[cfg(feature = "backend-s3")]
use crate::retry::{BackoffPolicy, RetryPolicy};
use bijux_atlas_core::ErrorCode;
use bijux_atlas_model::{ArtifactManifest, Catalog, DatasetId};
#[cfg(feature = "backend-s3")]
use reqwest::blocking::{Client, Response};
#[cfg(feature = "backend-s3")]
use reqwest::header::{ETAG, IF_NONE_MATCH};
use std::collections::BTreeMap;
#[cfg(feature = "backend-s3")]
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::{self, OpenOptions};
use std::io::Write;
#[cfg(feature = "backend-s3")]
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
#[cfg(feature = "backend-s3")]
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StoreErrorCode {
    NotFound,
    Validation,
    Conflict,
    Network,
    Io,
    CachedOnly,
    Unsupported,
    Internal,
}

impl StoreErrorCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotFound => "not_found",
            Self::Validation => "validation_error",
            Self::Conflict => "conflict",
            Self::Network => "network_error",
            Self::Io => "io_error",
            Self::CachedOnly => "cached_only_mode",
            Self::Unsupported => "unsupported",
            Self::Internal => "internal_error",
        }
    }

    #[must_use]
    pub const fn as_error_code(self) -> ErrorCode {
        match self {
            Self::NotFound => ErrorCode::QueryRejectedByPolicy,
            Self::Validation => ErrorCode::InvalidQueryParameter,
            Self::Conflict => ErrorCode::QueryRejectedByPolicy,
            Self::Network => ErrorCode::NotReady,
            Self::Io => ErrorCode::Internal,
            Self::CachedOnly => ErrorCode::NotReady,
            Self::Unsupported => ErrorCode::QueryRejectedByPolicy,
            Self::Internal => ErrorCode::Internal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreError {
    pub code: StoreErrorCode,
    pub message: String,
}

impl StoreError {
    #[must_use]
    pub fn new(code: StoreErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl Display for StoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for StoreError {}

#[derive(Debug, Clone, Default)]
pub struct StoreMetrics {
    pub bytes_downloaded: u64,
    pub bytes_uploaded: u64,
    pub request_count: u64,
    pub latency_ms_total: u128,
    pub failures_by_class: BTreeMap<String, u64>,
}

#[derive(Default)]
pub struct StoreMetricsCollector {
    inner: Mutex<StoreMetrics>,
}

impl StoreMetricsCollector {
    #[must_use]
    pub fn snapshot(&self) -> StoreMetrics {
        self.inner.lock().map(|m| m.clone()).unwrap_or_default()
    }
}

pub trait StoreInstrumentation: Send + Sync + 'static {
    fn observe_download(&self, _backend: &str, _bytes: usize, _latency: Duration) {}
    fn observe_upload(&self, _backend: &str, _bytes: usize, _latency: Duration) {}
    fn observe_error(&self, _backend: &str, _code: StoreErrorCode) {}
}

#[derive(Default)]
pub struct NoopInstrumentation;

impl StoreInstrumentation for NoopInstrumentation {}

impl StoreInstrumentation for StoreMetricsCollector {
    fn observe_download(&self, _backend: &str, bytes: usize, latency: Duration) {
        if let Ok(mut m) = self.inner.lock() {
            m.bytes_downloaded = m.bytes_downloaded.saturating_add(bytes as u64);
            m.request_count = m.request_count.saturating_add(1);
            m.latency_ms_total = m.latency_ms_total.saturating_add(latency.as_millis());
        }
    }

    fn observe_upload(&self, _backend: &str, bytes: usize, latency: Duration) {
        if let Ok(mut m) = self.inner.lock() {
            m.bytes_uploaded = m.bytes_uploaded.saturating_add(bytes as u64);
            m.request_count = m.request_count.saturating_add(1);
            m.latency_ms_total = m.latency_ms_total.saturating_add(latency.as_millis());
        }
    }

    fn observe_error(&self, _backend: &str, code: StoreErrorCode) {
        if let Ok(mut m) = self.inner.lock() {
            m.request_count = m.request_count.saturating_add(1);
            *m.failures_by_class.entry(code.as_str().to_string()).or_insert(0) += 1;
        }
    }
}

pub trait ArtifactStore {
    fn list_datasets(&self) -> Result<Vec<DatasetId>, StoreError>;
    fn get_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, StoreError>;
    fn get_sqlite_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, StoreError>;
    fn put_dataset(
        &self,
        dataset: &DatasetId,
        manifest_bytes: &[u8],
        sqlite_bytes: &[u8],
        expected_manifest_sha256: &str,
        expected_sqlite_sha256: &str,
    ) -> Result<(), StoreError>;
    fn exists(&self, dataset: &DatasetId) -> Result<bool, StoreError>;

    fn read_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, StoreError> {
        self.get_manifest(dataset)
    }

    fn get_sqlite_bytes_verified(&self, dataset: &DatasetId) -> Result<Vec<u8>, StoreError> {
        let manifest = self.get_manifest(dataset)?;
        let sqlite_bytes = self.get_sqlite_bytes(dataset)?;
        verify_expected_sha256(&sqlite_bytes, &manifest.checksums.sqlite_sha256)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e))?;
        Ok(sqlite_bytes)
    }

    fn publish_atomic(
        &self,
        dataset: &DatasetId,
        manifest_bytes: &[u8],
        sqlite_bytes: &[u8],
        expected_manifest_sha256: &str,
        expected_sqlite_sha256: &str,
    ) -> Result<(), StoreError> {
        self.put_dataset(
            dataset,
            manifest_bytes,
            sqlite_bytes,
            expected_manifest_sha256,
            expected_sqlite_sha256,
        )
    }

    fn acquire_publish_lock(&self, dataset: &DatasetId) -> Result<PublishLockGuard, StoreError>;
}

pub struct PublishLockGuard {
    lock_path: PathBuf,
}

impl PublishLockGuard {
    fn new(lock_path: PathBuf) -> Self {
        Self { lock_path }
    }
}

impl Drop for PublishLockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock_path);
    }
}

pub struct LocalFsStore {
    pub root: PathBuf,
    instrumentation: Arc<dyn StoreInstrumentation>,
}

impl LocalFsStore {
    #[must_use]
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            instrumentation: Arc::new(NoopInstrumentation),
        }
    }

    #[must_use]
    pub fn with_instrumentation(mut self, instrumentation: Arc<dyn StoreInstrumentation>) -> Self {
        self.instrumentation = instrumentation;
        self
    }
}

impl ArtifactStore for LocalFsStore {
    fn list_datasets(&self) -> Result<Vec<DatasetId>, StoreError> {
        let catalog_path = self.root.join(CATALOG_FILE);
        if !catalog_path.exists() {
            return Ok(Vec::new());
        }
        let raw = fs::read_to_string(catalog_path)
            .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
        let catalog: Catalog = serde_json::from_str(&raw)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
        validate_catalog_strict(&catalog)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e))?;
        Ok(catalog.datasets.into_iter().map(|x| x.dataset).collect())
    }

    fn get_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, StoreError> {
        let paths = dataset_artifact_paths(Path::new(&self.root), dataset);
        let lock_path = manifest_lock_path(Path::new(&self.root), dataset);
        let raw = fs::read(&paths.manifest)
            .map_err(|e| StoreError::new(StoreErrorCode::NotFound, e.to_string()))?;
        let sqlite = fs::read(&paths.sqlite)
            .map_err(|e| StoreError::new(StoreErrorCode::NotFound, e.to_string()))?;

        let lock_raw = fs::read_to_string(&lock_path).map_err(|e| {
            StoreError::new(
                StoreErrorCode::Validation,
                format!("missing manifest.lock: {e}"),
            )
        })?;
        let lock: ManifestLock = serde_json::from_str(&lock_raw)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
        lock.validate(&raw, &sqlite)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e))?;

        let manifest: ArtifactManifest = serde_json::from_slice(&raw)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
        manifest
            .validate_strict()
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
        Ok(manifest)
    }

    fn get_sqlite_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, StoreError> {
        let paths = dataset_artifact_paths(Path::new(&self.root), dataset);
        fs::read(paths.sqlite).map_err(|e| StoreError::new(StoreErrorCode::NotFound, e.to_string()))
    }

    fn put_dataset(
        &self,
        dataset: &DatasetId,
        manifest_bytes: &[u8],
        sqlite_bytes: &[u8],
        expected_manifest_sha256: &str,
        expected_sqlite_sha256: &str,
    ) -> Result<(), StoreError> {
        let started = Instant::now();
        let _guard = self.acquire_publish_lock(dataset)?;
        enforce_dataset_immutability(&self.root, dataset)?;

        verify_expected_sha256(manifest_bytes, expected_manifest_sha256)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e))?;
        verify_expected_sha256(sqlite_bytes, expected_sqlite_sha256)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e))?;

        let paths = dataset_artifact_paths(Path::new(&self.root), dataset);
        fs::create_dir_all(&paths.derived_dir)
            .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;

        let manifest_tmp = paths.derived_dir.join("manifest.json.tmp");
        let sqlite_tmp = paths.derived_dir.join("gene_summary.sqlite.tmp");
        let lock_tmp = paths.derived_dir.join("manifest.lock.tmp");

        write_and_sync(&manifest_tmp, manifest_bytes)?;
        write_and_sync(&sqlite_tmp, sqlite_bytes)?;
        let lock = ManifestLock::from_bytes(manifest_bytes, sqlite_bytes);
        let lock_bytes = serde_json::to_vec(&lock)
            .map_err(|e| StoreError::new(StoreErrorCode::Internal, e.to_string()))?;
        write_and_sync(&lock_tmp, &lock_bytes)?;

        fs::rename(&manifest_tmp, &paths.manifest)
            .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
        fs::rename(&sqlite_tmp, &paths.sqlite)
            .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
        fs::rename(
            &lock_tmp,
            manifest_lock_path(Path::new(&self.root), dataset),
        )
        .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;

        sync_dir(&paths.derived_dir)?;

        self.instrumentation.observe_upload(
            "localfs",
            manifest_bytes.len() + sqlite_bytes.len(),
            started.elapsed(),
        );
        Ok(())
    }

    fn exists(&self, dataset: &DatasetId) -> Result<bool, StoreError> {
        let paths = dataset_artifact_paths(Path::new(&self.root), dataset);
        Ok(paths.manifest.exists() && paths.sqlite.exists())
    }

    fn acquire_publish_lock(&self, dataset: &DatasetId) -> Result<PublishLockGuard, StoreError> {
        let paths = dataset_artifact_paths(Path::new(&self.root), dataset);
        fs::create_dir_all(&paths.derived_dir)
            .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
        let lock_path = publish_lock_path(Path::new(&self.root), dataset);
        match OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&lock_path)
        {
            Ok(_) => Ok(PublishLockGuard::new(lock_path)),
            Err(e) => Err(StoreError::new(
                StoreErrorCode::Conflict,
                format!("failed to acquire publish lock: {e}"),
            )),
        }
    }
}

#[derive(Clone)]
#[cfg(feature = "backend-s3")]
pub struct HttpReadonlyStore {
    pub base_url: String,
    pub cached_only_mode: bool,
    pub cache_root: Option<PathBuf>,
    client: Client,
    etags: Arc<Mutex<HashMap<String, String>>>,
    catalog_state: Arc<Mutex<CatalogCacheState>>,
    instrumentation: Arc<dyn StoreInstrumentation>,
}

#[derive(Debug, Default, Clone)]
#[cfg(feature = "backend-s3")]
struct CatalogCacheState {
    last_fetch: Option<Instant>,
    backoff_until: Option<Instant>,
    consecutive_errors: u32,
}

#[cfg(feature = "backend-s3")]
impl HttpReadonlyStore {
    #[must_use]
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            cached_only_mode: false,
            cache_root: None,
            client: Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .unwrap_or_else(|_| Client::new()),
            etags: Arc::new(Mutex::new(HashMap::new())),
            catalog_state: Arc::new(Mutex::new(CatalogCacheState::default())),
            instrumentation: Arc::new(NoopInstrumentation),
        }
    }

    #[must_use]
    pub fn with_cache(mut self, cache_root: PathBuf, cached_only_mode: bool) -> Self {
        self.cache_root = Some(cache_root);
        self.cached_only_mode = cached_only_mode;
        self
    }

    #[must_use]
    pub fn with_instrumentation(mut self, instrumentation: Arc<dyn StoreInstrumentation>) -> Self {
        self.instrumentation = instrumentation;
        self
    }

    fn url_for(&self, dataset: &DatasetId, file: &str) -> String {
        let id = dataset_key_prefix(dataset);
        format!("{}/{}/{}", self.base_url.trim_end_matches('/'), id, file)
    }

    fn fetch_bytes(&self, key: &str, url: &str) -> Result<Vec<u8>, StoreError> {
        if let Some(root) = &self.cache_root {
            let cached = root.join(key.replace('/', "__"));
            if cached.exists() {
                return fs::read(&cached)
                    .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()));
            }
            if self.cached_only_mode {
                return Err(StoreError::new(
                    StoreErrorCode::CachedOnly,
                    "cached-only mode enabled and key not present in cache",
                ));
            }
        } else if self.cached_only_mode {
            return Err(StoreError::new(
                StoreErrorCode::CachedOnly,
                "cached-only mode enabled without cache root",
            ));
        }
        Self::validate_url(url)?;

        let started = Instant::now();
        let mut req = self.client.get(url);
        if let Ok(map) = self.etags.lock() {
            if let Some(etag) = map.get(key) {
                req = req.header(IF_NONE_MATCH, etag);
            }
        }

        let resp = req
            .send()
            .map_err(|e| StoreError::new(StoreErrorCode::Network, e.to_string()))?;

        let bytes = handle_etag_response(resp, key, &self.etags, &self.cache_root)?;
        self.instrumentation
            .observe_download("http", bytes.len(), started.elapsed());
        Ok(bytes)
    }

    fn validate_url(url: &str) -> Result<(), StoreError> {
        let parsed = reqwest::Url::parse(url)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
        let host = parsed
            .host_str()
            .ok_or_else(|| StoreError::new(StoreErrorCode::Validation, "missing host"))?
            .to_ascii_lowercase();
        if host == "localhost" || host.ends_with(".localhost") {
            return Err(StoreError::new(
                StoreErrorCode::Validation,
                "blocked localhost host",
            ));
        }
        if let Ok(ip) = host.parse::<IpAddr>() {
            let blocked = match ip {
                IpAddr::V4(v4) => {
                    v4.is_private() || v4.is_loopback() || v4.is_link_local() || v4.is_broadcast()
                }
                IpAddr::V6(v6) => v6.is_loopback() || v6.is_unspecified() || v6.is_unique_local(),
            };
            if blocked {
                return Err(StoreError::new(
                    StoreErrorCode::Validation,
                    "blocked private host",
                ));
            }
        }
        Ok(())
    }
}

#[cfg(feature = "backend-s3")]
impl ArtifactStore for HttpReadonlyStore {
    fn list_datasets(&self) -> Result<Vec<DatasetId>, StoreError> {
        if let Ok(state) = self.catalog_state.lock() {
            if let Some(until) = state.backoff_until {
                if Instant::now() < until {
                    return Err(StoreError::new(
                        StoreErrorCode::Network,
                        "catalog backoff active after recent errors",
                    ));
                }
            }
            if let Some(last) = state.last_fetch {
                if Instant::now().saturating_duration_since(last) < Duration::from_millis(250) {
                    return Err(StoreError::new(
                        StoreErrorCode::Network,
                        "catalog fetch throttled to avoid hot loop",
                    ));
                }
            }
        }
        let bytes = self.fetch_bytes(
            "catalog.json",
            &format!("{}/catalog.json", self.base_url.trim_end_matches('/')),
        );
        match bytes {
            Ok(bytes) => {
                if let Ok(mut state) = self.catalog_state.lock() {
                    state.last_fetch = Some(Instant::now());
                    state.consecutive_errors = 0;
                    state.backoff_until = None;
                }
                let catalog: Catalog = serde_json::from_slice(&bytes)
                    .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
                validate_catalog_strict(&catalog)
                    .map_err(|e| StoreError::new(StoreErrorCode::Validation, e))?;
                Ok(catalog.datasets.into_iter().map(|x| x.dataset).collect())
            }
            Err(err) => {
                if let Ok(mut state) = self.catalog_state.lock() {
                    state.last_fetch = Some(Instant::now());
                    state.consecutive_errors = state.consecutive_errors.saturating_add(1);
                    let backoff_ms = 250_u64
                        .saturating_mul(state.consecutive_errors as u64)
                        .min(5_000);
                    state.backoff_until = Some(Instant::now() + Duration::from_millis(backoff_ms));
                }
                Err(err)
            }
        }
    }

    fn get_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, StoreError> {
        let key = dataset_manifest_key(dataset);
        let lock_key = dataset_manifest_lock_key(dataset);
        let bytes = self.fetch_bytes(&key, &self.url_for(dataset, "manifest.json"))?;
        let lock_bytes = self.fetch_bytes(&lock_key, &self.url_for(dataset, "manifest.lock"))?;
        let lock: ManifestLock = serde_json::from_slice(&lock_bytes)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
        lock.validate_manifest_only(&bytes)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e))?;
        let manifest: ArtifactManifest = serde_json::from_slice(&bytes)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
        manifest
            .validate_strict()
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
        Ok(manifest)
    }

    fn get_sqlite_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, StoreError> {
        let key = dataset_sqlite_key(dataset);
        self.fetch_bytes(&key, &self.url_for(dataset, "gene_summary.sqlite"))
    }

    fn put_dataset(
        &self,
        _dataset: &DatasetId,
        _manifest_bytes: &[u8],
        _sqlite_bytes: &[u8],
        _expected_manifest_sha256: &str,
        _expected_sqlite_sha256: &str,
    ) -> Result<(), StoreError> {
        Err(StoreError::new(
            StoreErrorCode::Unsupported,
            "http readonly backend cannot publish",
        ))
    }

    fn exists(&self, dataset: &DatasetId) -> Result<bool, StoreError> {
        match self.get_manifest(dataset) {
            Ok(_) => Ok(true),
            Err(err) if err.code == StoreErrorCode::NotFound => Ok(false),
            Err(err) => Err(err),
        }
    }

    fn acquire_publish_lock(&self, _dataset: &DatasetId) -> Result<PublishLockGuard, StoreError> {
        Err(StoreError::new(
            StoreErrorCode::Unsupported,
            "http readonly backend cannot lock",
        ))
    }
}
