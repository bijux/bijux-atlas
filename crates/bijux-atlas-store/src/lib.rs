#![forbid(unsafe_code)]

mod backend;
mod catalog;
mod manifest;
mod paths;

pub use backend::{
    enforce_dataset_immutability, ArtifactStore, HttpReadonlyStore, LocalFsStore,
    NoopInstrumentation, PublishLockGuard, RetryPolicy, S3LikeStore, StoreError, StoreErrorCode,
    StoreInstrumentation, StoreMetrics, StoreMetricsCollector,
};
pub use catalog::{
    canonical_catalog_json, merge_catalogs, sorted_catalog_entries, validate_catalog_strict,
};
pub use manifest::{verify_expected_sha256, ManifestLock};
pub use paths::{dataset_artifact_paths, manifest_lock_path, publish_lock_path};

pub const CRATE_NAME: &str = "bijux-atlas-store";
