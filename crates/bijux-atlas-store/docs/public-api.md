# PUBLIC API: bijux-atlas-store

Stability reference: [Stability Levels](../../../docs/_internal/governance/style/stability-levels.md)

Stable exports:
- `CRATE_NAME`
- Traits: `StoreRead`, `StoreWrite`, `StoreAdmin`, `ArtifactStore`
- Contract types: `StorePath`, `ArtifactRef`, `CatalogRef`
- Trait: `ArtifactStore`
- Backends: `LocalFsStore` (`backend-local`, default), `HttpReadonlyStore` (`backend-s3`), `S3LikeStore` (`backend-s3`)
- Retry/config: `RetryPolicy`
- Errors: `StoreError`, `StoreErrorCode`
- Instrumentation: `StoreInstrumentation`, `NoopInstrumentation`, `StoreMetrics`
- Locking: `PublishLockGuard`, `ManifestLock`
- Utilities: `enforce_dataset_immutability`, `verify_expected_sha256`
- Catalog utils: `validate_catalog_strict`, `canonical_catalog_json`, `sorted_catalog_entries`
- Path utils: `dataset_artifact_paths`, `dataset_key_prefix`, `dataset_manifest_key`, `dataset_manifest_lock_key`, `dataset_sqlite_key`, `manifest_lock_path`, `publish_lock_path`
