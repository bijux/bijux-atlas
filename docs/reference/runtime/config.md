# Runtime Config

- Owner: `bijux-atlas-server`
- Type: `reference`
- Audience: `user`
- Stability: `stable`
- Last updated for release: `v1`
- Reason to exist: define the canonical runtime environment surface consumed by `atlas-server`.

## Canonical sources

- Runtime loader: `crates/bijux-atlas-server/src/config/mod.rs`
- Env allowlist contract: `configs/contracts/env.schema.json`
- Contract reference: [Config Keys](../contracts/config-keys.md)

## Canonical runtime env keys

- Store and endpoint settings: `ATLAS_STORE_S3_ENABLED`, `ATLAS_STORE_S3_BASE_URL`, `ATLAS_STORE_S3_PRESIGNED_BASE_URL`
- Runtime paths: `ATLAS_BIND`, `ATLAS_STORE_ROOT`, `ATLAS_CACHE_ROOT`
- Cache behavior: `ATLAS_MAX_DATASET_COUNT`, `ATLAS_MAX_DISK_BYTES`, `ATLAS_PINNED_DATASETS`, `ATLAS_MAX_CONCURRENT_DOWNLOADS`
- Request behavior: `ATLAS_REQUEST_TIMEOUT_MS`, `ATLAS_SQL_TIMEOUT_MS`, `ATLAS_RESPONSE_MAX_BYTES`, `ATLAS_MAX_BODY_BYTES`
- Readiness and mode: `ATLAS_CACHED_ONLY_MODE`, `ATLAS_READINESS_REQUIRES_CATALOG`, `ATLAS_READ_ONLY_FS_MODE`

## Semantics

- `ATLAS_STORE_S3_BASE_URL` is required when `ATLAS_STORE_S3_ENABLED=true`.
- `ATLAS_CACHED_ONLY_MODE=true` requires `ATLAS_READINESS_REQUIRES_CATALOG=false`.
- Unknown `ATLAS_*` and `BIJUX_*` env keys fail startup unless `ATLAS_DEV_ALLOW_UNKNOWN_ENV=1` is set explicitly for local development.
- Startup logs emit the effective runtime config, but the server redacts known secret fields before printing.

## Source-of-truth rule

- `configs/contracts/env.schema.json` is the allowlist source of truth for accepted prefixed env keys.
- This page documents the canonical runtime-facing names that the server actively consumes.
- If an env name is renamed, update the runtime loader first, then the allowlist contract, then this page.

## Reproduce locally

- `cargo test -p bijux-atlas-server --test runtime_env_contract_startup`
- `cargo test -p bijux-atlas-server config::tests::runtime_config_contract_snapshot_points_to_the_allowlist_source --lib -- --exact`

## See also

- [Reference Index](../index.md)
- [Config Keys](../contracts/config-keys.md)
