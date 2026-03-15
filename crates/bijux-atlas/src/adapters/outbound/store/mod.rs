// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod backend_capabilities;
/// Backend-specific store adapters.
pub mod backends;
mod catalog;
mod manifest;
mod paths;
pub mod registry;
mod retry;

#[cfg(feature = "backend-s3")]
pub use backends::http::HttpReadonlyStore;
pub use backends::local::LocalFsStore;
#[cfg(feature = "backend-s3")]
pub use backends::s3::S3LikeStore;
pub use backend_capabilities::{validate_backend_compiled, BackendKind};
pub use catalog::{
    canonical_catalog_json, merge_catalogs, sorted_catalog_entries, validate_catalog_strict,
};
pub use crate::ports::store::{
    ArtifactRef, ArtifactStore, CatalogRef, NoopInstrumentation, PublishLockGuard, StoreAdmin,
    StoreError, StoreErrorCode, StoreInstrumentation, StoreMetrics, StoreMetricsCollector,
    StorePath, StoreRead, StoreWrite,
};
pub use manifest::{verify_expected_sha256, ManifestLock};
pub use paths::{
    dataset_artifact_paths, dataset_key_prefix, dataset_manifest_key, dataset_manifest_lock_key,
    dataset_sqlite_key, manifest_lock_path, publish_lock_path, CATALOG_FILE, MANIFEST_FILE,
    MANIFEST_LOCK_FILE, SQLITE_FILE,
};
pub use retry::{BackoffPolicy, RetryPolicy};

pub const CRATE_NAME: &str = "bijux-atlas";
