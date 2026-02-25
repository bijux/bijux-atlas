# Observability Pack Definition

- Owner: bijux-atlas-operations
- Stability: stable

## Canonical Observe Artifacts

- `ops/observe/alert-catalog.json`
- `ops/observe/slo-definitions.json`
- `ops/observe/telemetry-drills.json`
- `ops/observe/readiness.json`
- `ops/observe/generated/telemetry-index.json`

## Readiness Contract

- Readiness is `ready` only when SLOs, alerts, drills, and dashboard artifacts are present.
- Coverage checks ensure dashboard and alert catalogs are non-empty.
