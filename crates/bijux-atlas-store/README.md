# bijux-atlas-store

Storage contracts and backend adapters for atlas artifacts.

## Stable Store Traits

- `StoreRead`
- `StoreWrite`
- `StoreAdmin`

`ArtifactStore` remains available as a compatibility facade over these ports.

## Stable Contract Types

- `StorePath`
- `ArtifactRef`
- `CatalogRef`

## Backends

- local filesystem: `backends::local::LocalFsStore`
- HTTP read-only: `backends::http::HttpReadonlyStore`
- S3-compatible: `backends::s3::S3LikeStore` (feature `backend-s3`)

## Stable API Guidance

Considered stable:
- store port traits
- contract path/key helpers
- store error codes and retry policy abstraction

Internal implementation details in backend modules may evolve.
