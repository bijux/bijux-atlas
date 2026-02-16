use crate::catalog::validate_catalog_strict;
use crate::manifest::{verify_expected_sha256, ManifestLock};
use crate::paths::{dataset_artifact_paths, manifest_lock_path, publish_lock_path};
use bijux_atlas_model::{ArtifactManifest, Catalog, DatasetId};
use reqwest::blocking::{Client, Response};
use reqwest::header::{ETAG, IF_NONE_MATCH};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
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
}

pub trait StoreInstrumentation: Send + Sync + 'static {
    fn observe_download(&self, _backend: &str, _bytes: usize, _latency: Duration) {}
    fn observe_upload(&self, _backend: &str, _bytes: usize, _latency: Duration) {}
    fn observe_error(&self, _backend: &str, _code: StoreErrorCode) {}
}

#[derive(Default)]
pub struct NoopInstrumentation;

impl StoreInstrumentation for NoopInstrumentation {}

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
        let catalog_path = self.root.join("catalog.json");
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
struct CatalogCacheState {
    last_fetch: Option<Instant>,
    backoff_until: Option<Instant>,
    consecutive_errors: u32,
}

impl HttpReadonlyStore {
    #[must_use]
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            cached_only_mode: false,
            cache_root: None,
            client: Client::new(),
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
        let id = dataset.canonical_string();
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
}

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
        let key = format!("{}/manifest.json", dataset.canonical_string());
        let bytes = self.fetch_bytes(&key, &self.url_for(dataset, "manifest.json"))?;
        let manifest: ArtifactManifest = serde_json::from_slice(&bytes)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
        manifest
            .validate_strict()
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
        Ok(manifest)
    }

    fn get_sqlite_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, StoreError> {
        let key = format!("{}/gene_summary.sqlite", dataset.canonical_string());
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

#[derive(Clone)]
pub struct RetryPolicy {
    pub max_attempts: usize,
    pub base_backoff_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 4,
            base_backoff_ms: 120,
        }
    }
}

pub struct S3LikeStore {
    pub endpoint: String,
    pub presigned_endpoint: Option<String>,
    pub bucket: String,
    pub bearer_token: Option<String>,
    pub retry: RetryPolicy,
    pub cached_only_mode: bool,
    pub cache_root: Option<PathBuf>,
    client: Client,
    instrumentation: Arc<dyn StoreInstrumentation>,
}

impl S3LikeStore {
    #[must_use]
    pub fn new(endpoint: String, bucket: String) -> Self {
        Self {
            endpoint,
            presigned_endpoint: None,
            bucket,
            bearer_token: None,
            retry: RetryPolicy::default(),
            cached_only_mode: false,
            cache_root: None,
            client: Client::new(),
            instrumentation: Arc::new(NoopInstrumentation),
        }
    }

    #[must_use]
    pub fn with_bearer_token(mut self, token: Option<String>) -> Self {
        self.bearer_token = token;
        self
    }

    #[must_use]
    pub fn with_presigned_endpoint(mut self, endpoint: Option<String>) -> Self {
        self.presigned_endpoint = endpoint
            .map(|x| x.trim_end_matches('/').to_string())
            .filter(|x| !x.is_empty());
        self
    }

    #[must_use]
    pub fn with_retry(mut self, retry: RetryPolicy) -> Self {
        self.retry = retry;
        self
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

    fn object_url(&self, key: &str) -> String {
        let base = self.presigned_endpoint.as_deref().unwrap_or(&self.endpoint);
        format!(
            "{}/{}/{}",
            base.trim_end_matches('/'),
            self.bucket,
            key.trim_start_matches('/')
        )
    }

    fn get_with_retry(&self, key: &str) -> Result<Vec<u8>, StoreError> {
        if let Some(cache_root) = &self.cache_root {
            let cached = cache_root.join(key.replace('/', "__"));
            if cached.exists() {
                return fs::read(&cached)
                    .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()));
            }
            if self.cached_only_mode {
                return Err(StoreError::new(
                    StoreErrorCode::CachedOnly,
                    "cached-only mode enabled and object missing from cache",
                ));
            }
        } else if self.cached_only_mode {
            return Err(StoreError::new(
                StoreErrorCode::CachedOnly,
                "cached-only mode enabled without cache root",
            ));
        }

        let mut attempt = 0usize;
        let mut buf: Vec<u8> = Vec::new();
        loop {
            let started = Instant::now();
            let mut req = self.client.get(self.object_url(key));
            if !buf.is_empty() {
                req = req.header(reqwest::header::RANGE, format!("bytes={}-", buf.len()));
            }
            if let Some(token) = &self.bearer_token {
                req = req.bearer_auth(token);
            }
            match req.send() {
                Ok(resp) => {
                    if resp.status().is_success() || resp.status().as_u16() == 206 {
                        let total = resp
                            .headers()
                            .get("content-range")
                            .and_then(|v| v.to_str().ok())
                            .and_then(|v| v.split('/').nth(1))
                            .and_then(|v| v.parse::<usize>().ok());
                        let mut part = resp
                            .bytes()
                            .map_err(|e| StoreError::new(StoreErrorCode::Network, e.to_string()))?
                            .to_vec();
                        if part.is_empty() {
                            return Ok(buf);
                        }
                        buf.append(&mut part);
                        if let Some(total) = total {
                            if buf.len() < total {
                                attempt += 1;
                                if attempt >= self.retry.max_attempts {
                                    return Err(StoreError::new(
                                        StoreErrorCode::Network,
                                        "partial content did not complete within retry budget",
                                    ));
                                }
                                thread::sleep(Duration::from_millis(
                                    self.retry.base_backoff_ms.saturating_mul(attempt as u64),
                                ));
                                continue;
                            }
                        }
                        let bytes = buf.clone();
                        if let Some(root) = &self.cache_root {
                            fs::create_dir_all(root)
                                .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
                            let target = root.join(key.replace('/', "__"));
                            fs::write(target, &bytes)
                                .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
                        }
                        self.instrumentation.observe_download(
                            "s3like",
                            bytes.len(),
                            started.elapsed(),
                        );
                        return Ok(bytes);
                    }
                    if resp.status().as_u16() == 404 {
                        return Err(StoreError::new(
                            StoreErrorCode::NotFound,
                            "object not found",
                        ));
                    }
                }
                Err(err) => {
                    self.instrumentation
                        .observe_error("s3like", StoreErrorCode::Network);
                    if attempt + 1 >= self.retry.max_attempts {
                        return Err(StoreError::new(StoreErrorCode::Network, err.to_string()));
                    }
                }
            }
            attempt += 1;
            let backoff = self.retry.base_backoff_ms.saturating_mul(attempt as u64);
            thread::sleep(Duration::from_millis(backoff));
        }
    }

    fn put_bytes(&self, key: &str, bytes: &[u8]) -> Result<(), StoreError> {
        let started = Instant::now();
        let mut req = self.client.put(self.object_url(key)).body(bytes.to_vec());
        if let Some(token) = &self.bearer_token {
            req = req.bearer_auth(token);
        }
        let resp = req
            .send()
            .map_err(|e| StoreError::new(StoreErrorCode::Network, e.to_string()))?;
        if !resp.status().is_success() {
            return Err(StoreError::new(
                StoreErrorCode::Network,
                format!("s3-like put failed: {}", resp.status()),
            ));
        }
        self.instrumentation
            .observe_upload("s3like", bytes.len(), started.elapsed());
        Ok(())
    }
}

impl ArtifactStore for S3LikeStore {
    fn list_datasets(&self) -> Result<Vec<DatasetId>, StoreError> {
        let bytes = self.get_with_retry("catalog.json")?;
        let catalog: Catalog = serde_json::from_slice(&bytes)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
        validate_catalog_strict(&catalog)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e))?;
        Ok(catalog.datasets.into_iter().map(|x| x.dataset).collect())
    }

    fn get_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, StoreError> {
        let key = format!("{}/manifest.json", dataset.canonical_string());
        let bytes = self.get_with_retry(&key)?;
        let manifest: ArtifactManifest = serde_json::from_slice(&bytes)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
        manifest
            .validate_strict()
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
        Ok(manifest)
    }

    fn get_sqlite_bytes(&self, dataset: &DatasetId) -> Result<Vec<u8>, StoreError> {
        self.get_with_retry(&format!(
            "{}/gene_summary.sqlite",
            dataset.canonical_string()
        ))
    }

    fn put_dataset(
        &self,
        dataset: &DatasetId,
        manifest_bytes: &[u8],
        sqlite_bytes: &[u8],
        expected_manifest_sha256: &str,
        expected_sqlite_sha256: &str,
    ) -> Result<(), StoreError> {
        verify_expected_sha256(manifest_bytes, expected_manifest_sha256)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e))?;
        verify_expected_sha256(sqlite_bytes, expected_sqlite_sha256)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e))?;

        let prefix = dataset.canonical_string();
        self.put_bytes(&format!("{prefix}/manifest.json.tmp"), manifest_bytes)?;
        self.put_bytes(&format!("{prefix}/gene_summary.sqlite.tmp"), sqlite_bytes)?;

        let lock = ManifestLock::from_bytes(manifest_bytes, sqlite_bytes);
        let lock_json = serde_json::to_vec(&lock)
            .map_err(|e| StoreError::new(StoreErrorCode::Internal, e.to_string()))?;
        self.put_bytes(&format!("{prefix}/manifest.lock"), &lock_json)?;

        // S3 has no native rename; publish by writing final keys after temp upload verification.
        self.put_bytes(&format!("{prefix}/manifest.json"), manifest_bytes)?;
        self.put_bytes(&format!("{prefix}/gene_summary.sqlite"), sqlite_bytes)?;
        Ok(())
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
            "s3-like backend does not support local publish lock guard",
        ))
    }
}

pub fn enforce_dataset_immutability(root: &Path, dataset: &DatasetId) -> Result<(), StoreError> {
    let paths = dataset_artifact_paths(root, dataset);
    if paths.manifest.exists() || paths.sqlite.exists() {
        return Err(StoreError::new(
            StoreErrorCode::Conflict,
            "dataset already published; immutable artifacts must not be overwritten",
        ));
    }
    Ok(())
}

fn write_and_sync(path: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    let mut file =
        File::create(path).map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
    file.write_all(bytes)
        .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
    file.sync_all()
        .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))
}

fn sync_dir(dir: &Path) -> Result<(), StoreError> {
    let f = File::open(dir).map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
    f.sync_all()
        .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))
}

fn handle_etag_response(
    response: Response,
    key: &str,
    etags: &Arc<Mutex<HashMap<String, String>>>,
    cache_root: &Option<PathBuf>,
) -> Result<Vec<u8>, StoreError> {
    if response.status().as_u16() == 304 {
        if let Some(root) = cache_root {
            let target = root.join(key.replace('/', "__"));
            return fs::read(target)
                .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()));
        }
        return Err(StoreError::new(
            StoreErrorCode::Internal,
            "received 304 without cache root",
        ));
    }

    if response.status().as_u16() == 404 {
        return Err(StoreError::new(
            StoreErrorCode::NotFound,
            "resource not found",
        ));
    }

    if !response.status().is_success() {
        return Err(StoreError::new(
            StoreErrorCode::Network,
            format!("http fetch failed: {}", response.status()),
        ));
    }

    let etag = response
        .headers()
        .get(ETAG)
        .and_then(|h| h.to_str().ok())
        .map(ToString::to_string);

    let bytes = response
        .bytes()
        .map_err(|e| StoreError::new(StoreErrorCode::Network, e.to_string()))?
        .to_vec();

    if let Some(root) = cache_root {
        fs::create_dir_all(root).map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
        let target = root.join(key.replace('/', "__"));
        fs::write(target, &bytes)
            .map_err(|e| StoreError::new(StoreErrorCode::Io, e.to_string()))?;
    }

    if let Some(tag) = etag {
        if let Ok(mut map) = etags.lock() {
            map.insert(key.to_string(), tag);
        }
    }

    Ok(bytes)
}
