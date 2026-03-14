# Backends And Guarantees

## Supported Backends

- `LocalFsStore` (`backends::local`, feature `backend-local`, default)
- `HttpReadonlyStore` (`backends::http`, feature `backend-s3`)
- `S3LikeStore` (`backends::s3`, feature `backend-s3`)

## Guarantees

### Atomicity

- Local filesystem publish writes temporary files and renames into final paths.
- S3-like publish is best-effort staged write (no native atomic rename).
- HTTP backend is read-only.

### Idempotency

- Reads are idempotent across all backends.
- Dataset publish is immutable: existing dataset keys must not be overwritten.

### Integrity

- Manifest and SQLite checksums are verified during publish and read verification paths.
- `ManifestLock` binds manifest/sqlite digests.
