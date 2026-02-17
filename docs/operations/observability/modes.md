# Observability Modes

- Owner: `bijux-atlas-operations`

## What

Defines `minimal` and `full` observability deployment modes.

## Why

Allows local clusters without CRDs while preserving a full production profile.

## Contracts

- `minimal` mode: no CRD requirement; Prometheus manifest + OTEL collector only.
- `full` mode: requires `ServiceMonitor` and `PrometheusRule` CRDs.
- Install target: `ops-obs-up` (or `ops-obs-mode` with explicit `ATLAS_OBS_MODE`).
- Smoke target: `ops-observability-smoke`.
- Teardown target: `ops-obs-down`.
- Installer entrypoint: `ops/observability/scripts/install_obs_pack.sh` with `ATLAS_OBS_MODE=minimal|full`.

## Failure Modes

`full` mode fails fast if required CRDs are unavailable.

## How to verify

```bash
$ ATLAS_OBS_MODE=minimal make ops-obs-up
$ make ops-observability-smoke
$ make ops-obs-down
```

Expected output: minimal mode succeeds without CRDs.

## See also

- [CRD Dependency Policy](../k8s/crd-dependency-policy.md)
- [Acceptance Gates](acceptance-gates.md)
- [SLO](slo.md)


$ make ops-obs-mode-minimal
$ make ops-obs-mode-full

Expected output: minimal mode succeeds without CRDs; full mode fails fast when required CRDs are absent.
