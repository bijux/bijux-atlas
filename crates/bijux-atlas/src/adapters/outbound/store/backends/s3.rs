// SPDX-License-Identifier: Apache-2.0

#[cfg(feature = "backend-s3")]
use super::super::catalog::validate_catalog_strict;
#[cfg(feature = "backend-s3")]
use super::super::manifest::{verify_expected_sha256, ManifestLock};
#[cfg(feature = "backend-s3")]
use super::super::paths::{
    dataset_key_prefix, dataset_manifest_key, dataset_manifest_lock_key, dataset_sqlite_key,
    CATALOG_FILE,
};
#[cfg(feature = "backend-s3")]
use super::super::retry::{BackoffPolicy, RetryPolicy};
#[cfg(feature = "backend-s3")]
use crate::app::ports::store::{
    ArtifactStore, NoopInstrumentation, PublishLockGuard, StoreError, StoreErrorCode,
    StoreInstrumentation,
};
#[cfg(feature = "backend-s3")]
use crate::domain::dataset::{ArtifactManifest, Catalog, DatasetId};
#[cfg(feature = "backend-s3")]
use reqwest::blocking::Client;
#[cfg(feature = "backend-s3")]
use std::fs;
#[cfg(feature = "backend-s3")]
use std::path::PathBuf;
#[cfg(feature = "backend-s3")]
use std::sync::Arc;
#[cfg(feature = "backend-s3")]
use std::thread;
#[cfg(feature = "backend-s3")]
use std::time::Instant;

#[cfg(feature = "backend-s3")]
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

#[cfg(feature = "backend-s3")]
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
        let mut buffer: Vec<u8> = Vec::new();
        loop {
            let started = Instant::now();
            let mut request = self.client.get(self.object_url(key));
            if !buffer.is_empty() {
                request =
                    request.header(reqwest::header::RANGE, format!("bytes={}-", buffer.len()));
            }
            if let Some(token) = &self.bearer_token {
                request = request.bearer_auth(token);
            }
            match request.send() {
                Ok(response) => {
                    if response.status().is_success() || response.status().as_u16() == 206 {
                        let total = response
                            .headers()
                            .get("content-range")
                            .and_then(|v| v.to_str().ok())
                            .and_then(|v| v.split('/').nth(1))
                            .and_then(|v| v.parse::<usize>().ok());
                        let mut part = response
                            .bytes()
                            .map_err(|e| StoreError::new(StoreErrorCode::Network, e.to_string()))?
                            .to_vec();
                        if part.is_empty() {
                            return Ok(buffer);
                        }
                        buffer.append(&mut part);
                        if let Some(total) = total {
                            if buffer.len() < total {
                                attempt += 1;
                                if attempt >= self.retry.max_attempts {
                                    return Err(StoreError::new(
                                        StoreErrorCode::Network,
                                        "partial content did not complete within retry budget",
                                    ));
                                }
                                thread::sleep(self.retry.delay_for_attempt(attempt));
                                continue;
                            }
                        }
                        let bytes = buffer.clone();
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
                    if response.status().as_u16() == 404 {
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
            thread::sleep(self.retry.delay_for_attempt(attempt));
        }
    }

    fn put_bytes(&self, key: &str, bytes: &[u8]) -> Result<(), StoreError> {
        let started = Instant::now();
        let mut request = self.client.put(self.object_url(key)).body(bytes.to_vec());
        if let Some(token) = &self.bearer_token {
            request = request.bearer_auth(token);
        }
        let response = request
            .send()
            .map_err(|e| StoreError::new(StoreErrorCode::Network, e.to_string()))?;
        if !response.status().is_success() {
            return Err(StoreError::new(
                StoreErrorCode::Network,
                format!("s3-like put failed: {}", response.status()),
            ));
        }
        self.instrumentation
            .observe_upload("s3like", bytes.len(), started.elapsed());
        Ok(())
    }
}

#[cfg(feature = "backend-s3")]
impl ArtifactStore for S3LikeStore {
    fn list_datasets(&self) -> Result<Vec<DatasetId>, StoreError> {
        let bytes = self.get_with_retry(CATALOG_FILE)?;
        let catalog: Catalog = serde_json::from_slice(&bytes)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e.to_string()))?;
        validate_catalog_strict(&catalog)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e))?;
        Ok(catalog.datasets.into_iter().map(|x| x.dataset).collect())
    }

    fn get_manifest(&self, dataset: &DatasetId) -> Result<ArtifactManifest, StoreError> {
        let key = dataset_manifest_key(dataset);
        let lock_key = dataset_manifest_lock_key(dataset);
        let bytes = self.get_with_retry(&key)?;
        let lock_bytes = self.get_with_retry(&lock_key)?;
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
        self.get_with_retry(&dataset_sqlite_key(dataset))
    }

    fn put_dataset(
        &self,
        dataset: &DatasetId,
        manifest_bytes: &[u8],
        sqlite_bytes: &[u8],
        expected_manifest_sha256: &str,
        expected_sqlite_sha256: &str,
    ) -> Result<(), StoreError> {
        if self.exists(dataset)? {
            return Err(StoreError::new(
                StoreErrorCode::Conflict,
                "dataset already exists and cannot be overwritten",
            ));
        }

        verify_expected_sha256(manifest_bytes, expected_manifest_sha256)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e))?;
        verify_expected_sha256(sqlite_bytes, expected_sqlite_sha256)
            .map_err(|e| StoreError::new(StoreErrorCode::Validation, e))?;

        let prefix = dataset_key_prefix(dataset);
        self.put_bytes(&format!("{prefix}/manifest.json.tmp"), manifest_bytes)?;
        self.put_bytes(&format!("{prefix}/gene_summary.sqlite.tmp"), sqlite_bytes)?;

        let lock = ManifestLock::from_bytes(manifest_bytes, sqlite_bytes);
        let lock_json = serde_json::to_vec(&lock)
            .map_err(|e| StoreError::new(StoreErrorCode::Internal, e.to_string()))?;
        self.put_bytes(&format!("{prefix}/manifest.lock"), &lock_json)?;

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
