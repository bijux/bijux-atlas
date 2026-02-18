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

## Failure Isolation

- Store retries are bounded by retry policy.
- Store circuit breaker opens after configured consecutive failures.
- Cached-only mode serves cached artifacts and rejects uncached artifacts deterministically.
- Corrupt store payloads fail checksum validation and are quarantined by cache policy.

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
