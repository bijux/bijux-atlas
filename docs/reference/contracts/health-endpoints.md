# Health Endpoints

- Owner: `bijux-atlas-operations`
- Type: `reference`
- Audience: `operator`
- Stability: `stable`
- Last verified against: `main@2a4d4cd97d7132be969d573ba9461d6bd0a1653e`
- Last changed: `2026-03-03`
- Reason to exist: define the stable semantics of `/healthz` and `/readyz`.

## Endpoint Semantics

- `GET /healthz` reports process liveness. It answers `200` when the server can serve traffic and does not depend on catalog freshness.
- `GET /readyz` reports traffic readiness. It answers `200` only when the server is marked ready and any required catalog dependency is available.
- When `cachedOnlyMode=true`, `/readyz` must not fail only because the catalog is absent.
- When `readinessRequiresCatalog=true` and cached-only mode is disabled, `/readyz` must return `503` until a catalog is available.

## Contract Notes

- `request_id` propagation applies to both endpoints.
- Both endpoints are part of the cheap SLI class and are valid smoke-check targets.
