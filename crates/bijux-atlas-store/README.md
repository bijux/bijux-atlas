# bijux-atlas-store

![Version](https://img.shields.io/badge/version-0.1.0-informational.svg) ![License: Apache-2.0](https://img.shields.io/badge/license-Apache%202.0-blue.svg) ![Docs](https://img.shields.io/badge/docs-contract-stable-brightgreen.svg)

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

- local filesystem: `backends::local::LocalFsStore` (feature `backend-local`, default)
- HTTP read-only: `backends::http::HttpReadonlyStore` (feature `backend-s3`)
- S3-compatible: `backends::s3::S3LikeStore` (feature `backend-s3`)

## Stable API Guidance

Considered stable:
- store port traits
- contract path/key helpers
- store error codes and retry policy abstraction

Internal implementation details in backend modules may evolve.

## References

- Backend feature matrix: `docs/backend-feature-matrix.md`
- Attack surface budget: `docs/attack-surface-budget.md`

## Purpose
- Describe the crate responsibility and stable boundaries.

## How to use
- Read `docs/index.md` for workflows and examples.
- Use the crate through its documented public API only.

## Where docs live
- Crate docs index: `docs/index.md`
- Contract: `CONTRACT.md`
