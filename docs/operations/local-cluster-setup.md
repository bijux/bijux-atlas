# Local Cluster Setup

This setup defines the canonical local deployment workflow:

1. Build datasets in a controlled ingest environment.
2. Publish immutable artifacts to a mirrored store.
3. Serve datasets from Kubernetes with cache+policy controls.

## 1) Mirror Artifacts

Run ingest and publish to a local or object-store-backed mirror:

```sh
make fetch-fixtures
./ops/datasets/fixtures/run-medium-ingest.sh
cargo run -p bijux-atlas-cli --bin bijux-atlas -- atlas dataset publish \
  --source-root artifacts/medium-output \
  --store-root artifacts/server-store \
  --release 110 --species homo_sapiens --assembly GRCh38
cargo run -p bijux-atlas-cli --bin bijux-atlas -- atlas catalog publish \
  --store-root artifacts/server-store \
  --catalog artifacts/server-store/catalog.json
```

## 2) Run in Kubernetes

- Use `ops/k8s/charts/bijux-atlas/`.
- Set cache root to an `emptyDir` volume.
- Set store backend credentials via Secret.
- Configure pinned datasets for warm startup.

Core values:

- `datasetCache.maxDiskBytes`
- `datasetCache.maxDatasetCount`
- `catalog.backoffBaseMs`
- `catalog.breakerFailureThreshold`
- `catalog.breakerOpenMs`

## 3) Operate

Health and readiness:

- `GET /healthz`
- `GET /readyz`
- `GET /metrics`

Dataset discovery:

- `GET /v1/datasets`
- `GET /v1/datasets/{release}/{species}/{assembly}`

Provenance headers on responses:

- `X-Atlas-Dataset-Hash`
- `X-Atlas-Release`

## 4) Demo

Run end-to-end local demo:

```sh
make local-full
```
