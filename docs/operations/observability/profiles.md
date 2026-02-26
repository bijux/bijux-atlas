# Observability Profiles

- Owner: `bijux-atlas-operations`
- Tier: `tier2`
- Audience: `operators`
- Source-of-truth: `ops/CONTRACT.md`, `ops/inventory/**`, `ops/schema/**`

- Owner: `bijux-atlas-operations`

## What

Defines durable observability deployment profiles: `local-compose`, `kind`, `cluster`.

## Why

Profiles express deployment context without temporal or install-step wording.

## Contracts

- `local-compose`: single-node local development footprint via docker compose.
- `kind`: kind-compatible profile with CRD-aware optional features.
- `cluster`: full cluster profile with ServiceMonitor/PrometheusRule contracts enabled.
- Install target: `make ops-observability-mode ATLAS_OBS_PROFILE=<profile>`.
- Teardown target: `make ops-observability-down`.
- Validation target: `make ops-observability-validate`.
- Namespace convention: `atlas-observability`.
- Storage mode default: `ATLAS_OBS_STORAGE_MODE=ephemeral` (optional `persistent`).
- Offline mode: `ATLAS_OBS_OFFLINE=1` requires local mirrored images and pinned digests.

## Failure modes

- `cluster` profile fails fast if required CRDs are unavailable.
- Airgapped usage: mirror images and pin digests in `configs/ops/observability-pack.json` before install.

## How to verify

```bash
make ops-observability-mode ATLAS_OBS_PROFILE=kind
make ops-observability-pack-verify
make ops-observability-pack-health
make ops-observability-pack-smoke
make ops-observability-pack-export
make ops-observability-pack-conformance-report
make ops-observability-down
```

Expected output: profile install/validate succeeds or fails with deterministic profile-specific diagnostics.

## See also

- [Acceptance Gates](acceptance-gates.md)
- [SLO](slo.md)
