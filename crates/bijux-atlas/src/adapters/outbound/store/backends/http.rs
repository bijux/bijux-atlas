// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "backend-s3")]
use crate::app::ports::store::{
    ArtifactStore, NoopInstrumentation, PublishLockGuard, StoreError, StoreErrorCode,
    StoreInstrumentation,
};
#[cfg(feature = "backend-s3")]
use super::super::catalog::validate_catalog_strict;
#[cfg(feature = "backend-s3")]
use super::super::manifest::ManifestLock;
#[cfg(feature = "backend-s3")]
use super::super::paths::{
    dataset_key_prefix, dataset_manifest_key, dataset_manifest_lock_key, dataset_sqlite_key,
};
#[cfg(feature = "backend-s3")]
use crate::domain::dataset::{ArtifactManifest, Catalog, DatasetId};
#[cfg(feature = "backend-s3")]
use reqwest::blocking::{Client, Response};
#[cfg(feature = "backend-s3")]
use reqwest::header::{ETAG, IF_NONE_MATCH};
#[cfg(feature = "backend-s3")]
use std::collections::HashMap;
#[cfg(feature = "backend-s3")]
use std::fs;
#[cfg(feature = "backend-s3")]
use std::net::IpAddr;
#[cfg(feature = "backend-s3")]
use std::path::PathBuf;
#[cfg(feature = "backend-s3")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "backend-s3")]
use std::time::{Duration, Instant};

#[cfg(feature = "backend-s3")]
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

#[cfg(feature = "backend-s3")]
#[derive(Debug, Default, Clone)]
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

        let response = req
            .send()
            .map_err(|e| StoreError::new(StoreErrorCode::Network, e.to_string()))?;

        let bytes = handle_etag_response(response, key, &self.etags, &self.cache_root)?;
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

#[cfg(feature = "backend-s3")]
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
        .and_then(|header| header.to_str().ok())
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
