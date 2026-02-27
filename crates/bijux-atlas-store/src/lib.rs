// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod backend;
mod backend_capabilities;
pub mod backends;
mod catalog;
mod contracts;
mod manifest;
mod paths;
mod retry;

#[cfg(feature = "backend-s3")]
pub use backend::S3LikeStore;
#[cfg(feature = "backend-s3")]
pub use backend::HttpReadonlyStore;
pub use backend::{
    enforce_dataset_immutability, ArtifactStore, LocalFsStore, NoopInstrumentation,
    PublishLockGuard, StoreError, StoreErrorCode, StoreInstrumentation, StoreMetrics,
    StoreMetricsCollector,
};
pub use backend_capabilities::{validate_backend_compiled, BackendKind};
pub use catalog::{
    canonical_catalog_json, merge_catalogs, sorted_catalog_entries, validate_catalog_strict,
};
pub use contracts::{ArtifactRef, CatalogRef, StoreAdmin, StorePath, StoreRead, StoreWrite};
pub use manifest::{verify_expected_sha256, ManifestLock};
pub use paths::{
    dataset_artifact_paths, dataset_key_prefix, dataset_manifest_key, dataset_manifest_lock_key,
    dataset_sqlite_key, manifest_lock_path, publish_lock_path, CATALOG_FILE, MANIFEST_FILE,
    MANIFEST_LOCK_FILE, SQLITE_FILE,
};
pub use retry::{BackoffPolicy, RetryPolicy};

pub const CRATE_NAME: &str = "bijux-atlas-store";
