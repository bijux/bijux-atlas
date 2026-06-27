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

pub use backend::{
    ArtifactStore, NoopInstrumentation, PublishLockGuard, StoreError, StoreErrorCode,
    StoreInstrumentation, StoreMetrics, StoreMetricsCollector,
};
pub use backend_capabilities::{validate_backend_compiled, BackendKind};
#[cfg(feature = "backend-s3")]
pub use backends::http::HttpReadonlyStore;
pub use backends::local::LocalFsStore;
#[cfg(feature = "backend-s3")]
pub use backends::s3::S3LikeStore;
pub use catalog::{
    canonical_catalog_json, merge_catalogs, sorted_catalog_entries, validate_catalog_strict,
};
pub use contracts::{ArtifactRef, CatalogRef, StoreAdmin, StorePath, StoreRead, StoreWrite};
pub use manifest::{verify_expected_sha256, ManifestLock};
pub use paths::{
    dataset_artifact_paths, dataset_key_prefix, dataset_manifest_key, dataset_manifest_lock_key,
    dataset_sqlite_key, immutability_marker_path, lifecycle_state_path, lifecycle_transitions_path,
    manifest_lock_path, publish_lock_path, CATALOG_FILE, IMMUTABILITY_MARKER_FILE,
    LIFECYCLE_STATE_FILE, LIFECYCLE_TRANSITIONS_FILE, MANIFEST_FILE, MANIFEST_LOCK_FILE,
    PUBLISH_LOCK_FILE, SQLITE_FILE,
};
pub use retry::{BackoffPolicy, RetryPolicy};

pub const CRATE_NAME: &str = "bijux-atlas-store";
