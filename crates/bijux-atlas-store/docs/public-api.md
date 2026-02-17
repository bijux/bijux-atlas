# PUBLIC API: bijux-atlas-store

Stable exports:
- `CRATE_NAME`
- Trait: `ArtifactStore`
- Backends: `LocalFsStore`, `HttpReadonlyStore`, `S3LikeStore`
- Retry/config: `RetryPolicy`
- Errors: `StoreError`, `StoreErrorCode`
- Instrumentation: `StoreInstrumentation`, `NoopInstrumentation`, `StoreMetrics`
- Locking: `PublishLockGuard`, `ManifestLock`
- Utilities: `enforce_dataset_immutability`, `verify_expected_sha256`
- Catalog utils: `validate_catalog_strict`, `canonical_catalog_json`, `sorted_catalog_entries`
- Path utils: `dataset_artifact_paths`, `manifest_lock_path`, `publish_lock_path`
