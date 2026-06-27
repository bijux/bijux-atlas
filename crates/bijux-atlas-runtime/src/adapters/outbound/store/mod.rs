// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

pub mod registry;
pub mod testing;

pub use bijux_atlas_store::{
    canonical_catalog_json, dataset_artifact_paths, dataset_key_prefix, dataset_manifest_key,
    dataset_manifest_lock_key, dataset_sqlite_key, manifest_lock_path, merge_catalogs,
    publish_lock_path, sorted_catalog_entries, validate_backend_compiled, validate_catalog_strict,
    verify_expected_sha256, ArtifactRef, ArtifactStore, BackendKind, BackoffPolicy, CatalogRef,
    LocalFsStore, ManifestLock, NoopInstrumentation, PublishLockGuard, StoreAdmin, StoreError,
    StoreErrorCode, StoreInstrumentation, StoreMetrics, StoreMetricsCollector, StorePath,
    StoreRead, StoreWrite, CATALOG_FILE, MANIFEST_FILE, MANIFEST_LOCK_FILE, SQLITE_FILE,
};
#[cfg(feature = "backend-s3")]
pub use bijux_atlas_store::{HttpReadonlyStore, S3LikeStore};
pub use registry::backends::{LocalFsBackend, RetryPolicy, S3LikeBackend};
pub use registry::federated::{FederatedBackend, RegistrySource};

pub const CRATE_NAME: &str = "bijux-atlas";
