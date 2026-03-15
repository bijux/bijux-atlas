// SPDX-License-Identifier: Apache-2.0

use super::manifest::verify_expected_sha256;
use crate::domain::dataset::{ArtifactManifest, DatasetId};
use crate::errors::ErrorCode;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;

pub use super::backends::local::LocalFsStore;
#[cfg(feature = "backend-s3")]
pub use super::backends::http::HttpReadonlyStore;
#[cfg(feature = "backend-s3")]
pub use super::backends::s3::S3LikeStore;

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
    pub(crate) fn new(lock_path: PathBuf) -> Self {
        Self { lock_path }
    }
}

impl Drop for PublishLockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock_path);
    }
}
