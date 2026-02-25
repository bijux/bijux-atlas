# OpenTelemetry Outage Behavior

- Owner: `bijux-atlas-operations`

## Invariant

Server request handling remains available when OTEL collector is unavailable.

## Expected degradation

- Traces may be dropped or absent.
- Metrics and logs continue to function.
- Readiness for serving endpoints remains based on application health, not collector availability.

## Drill command

- `make ops-drill-otel-outage`
- `make ops-observability-validate`
- `make observability-pack-drills`

## Evidence

- `artifacts/ops/observe/traces.snapshot.log`
- `artifacts/ops/observe/metrics.prom`
- `artifacts/observability/pack-conformance-report.json`
