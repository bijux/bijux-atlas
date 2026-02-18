# Observability Profiles

- Owner: `bijux-atlas-operations`

## What

Defines durable observability deployment profiles: `local`, `kind`, `cluster`, `airgapped`.

## Why

Profiles express deployment context without temporal or install-step wording.

## Contracts

- `local`: single-node local development footprint.
- `kind`: kind-compatible profile with CRD-aware optional features.
- `cluster`: full cluster profile with ServiceMonitor/PrometheusRule contracts enabled.
- `airgapped`: no external pull dependency during validation workflows.
- Install target: `make ops-obs-up` with `ATLAS_OBS_PROFILE=<profile>`.
- Teardown target: `make ops-obs-down`.
- Validation target: `make ops-observability-validate`.

## Failure modes

- `cluster` profile fails fast if required CRDs are unavailable.
- `airgapped` profile fails when required local images/assets are missing.

## How to verify

```bash
ATLAS_OBS_PROFILE=kind make ops-obs-up
make ops-observability-smoke
make ops-obs-down
```

Expected output: profile install/validate succeeds or fails with deterministic profile-specific diagnostics.

## See also

- [Acceptance Gates](acceptance-gates.md)
- [SLO](slo.md)
