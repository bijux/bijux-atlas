# Operations Runbook

## Startup

1. Validate environment and config contracts.
2. Start server with `cargo run -p bijux-atlas-server --bin atlas-server`.
3. Confirm readiness with `GET /readyz` and liveness with `GET /healthz`.

## Required Config Surface

- Bind and storage: `ATLAS_BIND`, `ATLAS_STORE_ROOT`, `ATLAS_CACHE_ROOT`.
- Query safety: `ATLAS_REQUEST_TIMEOUT_MS`, `ATLAS_SQL_TIMEOUT_MS`, `ATLAS_RESPONSE_MAX_BYTES`.
- Cache and downloads: `ATLAS_MAX_DATASET_COUNT`, `ATLAS_MAX_CONCURRENT_DOWNLOADS`.

## Common Failures

- `unknown env vars rejected by contract`: remove unknown `ATLAS_*`/`BIJUX_*` vars or enable local override.
- `dataset manifest unavailable`: upstream store is unavailable or inconsistent.
- `server draining; refusing new requests`: instance is in shutdown drain mode.

## Troubleshooting

- Use `GET /metrics` to inspect request latency and cache/store health counters.
- Use `GET /debug/dataset-health` for dataset cache verification status.
- Use `GET /debug/registry-health` to inspect federated registry source health.

## Safe Rollout Checks

- OpenAPI and endpoint contracts are unchanged.
- `x-request-id` is present on responses for core routes.
- P95 latency for `/v1/genes` and `/v1/diff/genes` remains within SLO envelope.
