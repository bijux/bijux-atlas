# Store And Registry Contract

- Owner: `platform`
- Stability: `stable`

## What

Runtime contract for artifact store backends and registry/catalog resolution.

## Why

Serving must isolate store failures, keep deterministic catalog behavior, and avoid cache corruption.

## Store Interface (minimal)

- `List(catalog)` via `fetch_catalog(if_none_match)`
- `Get(manifest)` via `fetch_manifest(dataset_id)`
- `Fetch(blob)` via `fetch_sqlite_bytes(dataset_id)` and related artifact fetches

## Backends

- `localfs`: local filesystem read-only artifact root
- `http_s3`: HTTP(S) and S3/MinIO compatible object store pathing
- `federated`: deterministic multi-registry merge over source backends
- `fake`: test fault-injection backend

## Backend Conformance Gates

- Local filesystem backend: `make ci-store-conformance-localfs`
- HTTP/ETag contract path: `make ci-store-conformance-http`
- S3/MinIO runtime integration: `make ci-store-conformance-s3`
- Aggregated gate: `make ci-store-conformance`

All gates are required to keep backend behavior aligned on atomic fetch, checksum validation, and ETag/304 semantics.

## Redis Mode

## Failure Isolation

- Store retries are bounded by retry policy.
- Store circuit breaker opens after configured consecutive failures.
- Cached-only mode serves cached artifacts and rejects uncached artifacts deterministically.
- Corrupt store payloads fail checksum validation and are quarantined by cache policy.

Redis is optional and scoped to runtime protections (rate limiting / response cache acceleration). Core serving correctness must remain valid with Redis disabled.

## Catalog Semantics

- Catalog fetch uses etag + TTL with background refresh.
- Catalog refresh uses lock-based single refresh path (no stampede).
- Federated catalog merge is deterministic with priority + lexical tie-break.
- Maximum federation source count is enforced by `ATLAS_REGISTRY_MAX_SOURCES` (default `8`).

## Health/Readiness

- `/healthz` is liveness and remains `200` while process is running.
- `/readyz` requires app ready and catalog loaded unless cached-only mode allows operating without live catalog.

## Observability

- Store errors are exported with low-cardinality labels: backend + class.
- Store spans include backend tags for `store_catalog_fetch` and dataset download path spans.

## How to verify

```bash
make dev-test-all
```
